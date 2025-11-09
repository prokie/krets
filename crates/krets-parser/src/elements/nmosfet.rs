use crate::{models::nmosfet::NMosfetModel, prelude::*};

use nom::{
    IResult, Parser,
    bytes::complete::tag_no_case,
    character::complete::{space0, space1},
    combinator::all_consuming,
    multi,
    sequence::preceded,
};

#[derive(Debug, Clone)]
/// Represents a MOSFET (Metal-Oxide-Semiconductor Field-Effect Transistor) in a circuit.
/// SPICE format: M<name> <drain> <gate> <source> <bulk/substrate> <model> [parameters...]
pub struct NMOSFET {
    /// Name of the MOSFET.
    pub name: String,
    /// Drain node of the MOSFET.
    pub drain: String,
    /// Gate node of the MOSFET.
    pub gate: String,
    /// Source node of the MOSFET.
    pub source: String,
    /// Bulk (or Substrate) node of the MOSFET.
    pub bulk: String, // Added bulk node
    /// Model name associated with the MOSFET (required).
    pub model_name: String,
    /// The model associated with the MOSFET.
    pub model: NMosfetModel,
    /// Multiplicity factor. Simulates "m" parallel devices
    pub multiplicity: Option<usize>,
    /// Width of the MOSFET.
    pub width: Option<f64>,
    /// Length of the MOSFET.
    pub length: Option<f64>,
}

impl NMOSFET {
    fn threshold_voltage(&self) -> f64 {
        self.model.voltage_threshold
    }

    fn beta(&self) -> f64 {
        self.model.beta()
    }

    fn lambda(&self) -> f64 {
        self.model.channel_length_modulation
    }

    fn g_m(&self, v_gs: f64, v_ds: f64) -> f64 {
        let v_th = self.threshold_voltage();
        let beta = self.beta();
        let lambda = self.lambda();
        if v_gs <= v_th {
            0.0
        } else if v_ds >= 0.0 && v_ds <= (v_gs - v_th) {
            // Linear region
            beta * v_ds
        } else if v_ds >= (v_gs - v_th) && v_ds >= 0.0 {
            // Saturation region
            beta * (v_gs - v_th) * (1.0 + lambda * v_ds)
        } else {
            0.0
        }
    }

    fn g_ds(&self, v_gs: f64, v_ds: f64) -> f64 {
        let v_th = self.threshold_voltage();
        let beta = self.beta();
        let lambda = self.lambda();

        if v_gs <= v_th {
            0.0
        } else if v_ds >= 0.0 && v_ds <= (v_gs - v_th) {
            // Linear region
            beta * (v_gs - v_th - v_ds)
        } else if v_ds >= (v_gs - v_th) && v_ds >= 0.0 {
            // Saturation region
            (beta / 2.0) * lambda * (v_gs - v_th).powi(2)
        } else {
            0.0
        }
    }

    fn i_d(&self, v_gs: f64, v_ds: f64) -> f64 {
        let v_th = self.threshold_voltage();
        let beta = self.beta();
        let lambda = self.lambda();

        if v_gs <= v_th {
            0.0
        } else if v_ds >= 0.0 && v_ds <= (v_gs - v_th) {
            // Linear region
            beta * ((v_gs - v_th) * v_ds - (v_ds.powi(2) / 2.0))
        } else if v_ds >= (v_gs - v_th) && v_ds >= 0.0 {
            // Saturation region
            (beta / 2.0) * (v_gs - v_th).powi(2) * (1.0 + lambda * v_ds)
        } else {
            0.0
        }
    }
}

impl Identifiable for NMOSFET {
    /// Returns the identifier of the MOSFET in the format `M{name}`.
    fn identifier(&self) -> String {
        format!("M{}", self.name)
    }
}

impl Stampable for NMOSFET {
    fn stamp_conductance_matrix_dc(
        &self,
        index_map: &HashMap<String, usize>,
        solution_map: &HashMap<String, f64>,
    ) -> Vec<Triplet<usize, usize, f64>> {
        let v_g = solution_map
            .get(&format!("V({})", self.gate))
            .unwrap_or(&0.0);
        let v_s = solution_map
            .get(&format!("V({})", self.source))
            .unwrap_or(&0.0);
        let v_d = solution_map
            .get(&format!("V({})", self.drain))
            .unwrap_or(&0.0);

        let v_gs = v_g - v_s;
        let v_ds = v_d - v_s;

        let mut triplets = Vec::new();
        let g_m = self.g_m(v_gs, v_ds);
        let g_ds = self.g_ds(v_gs, v_ds);

        let index_d = index_map.get(&self.drain);
        let index_s = index_map.get(&self.source);
        let index_g = index_map.get(&self.gate);

        if let Some(&id) = index_d {
            triplets.push(Triplet::new(id, id, g_ds));
        }

        if let Some(&is) = index_s {
            triplets.push(Triplet::new(is, is, g_ds + g_m));
        }

        if let (Some(&id), Some(&is)) = (index_d, index_s) {
            triplets.push(Triplet::new(id, is, -(g_ds + g_m)));
            triplets.push(Triplet::new(is, id, g_ds + g_m));
        }

        if let (Some(&is), Some(&ig)) = (index_s, index_g) {
            triplets.push(Triplet::new(is, ig, g_m));
        }

        if let (Some(&id), Some(&ig)) = (index_d, index_g) {
            triplets.push(Triplet::new(id, ig, g_m));
        }

        triplets
    }

    fn stamp_excitation_vector_dc(
        &self,
        index_map: &HashMap<String, usize>,
        solution_map: &HashMap<String, f64>,
    ) -> Vec<Triplet<usize, usize, f64>> {
        let v_g = solution_map
            .get(&format!("V({})", self.gate))
            .unwrap_or(&0.0);
        let v_s = solution_map
            .get(&format!("V({})", self.source))
            .unwrap_or(&0.0);
        let v_d = solution_map
            .get(&format!("V({})", self.drain))
            .unwrap_or(&0.0);

        let v_gs = v_g - v_s;
        let v_ds = v_d - v_s;
        let g_ds = self.g_ds(v_gs, v_ds);
        let g_m = self.g_m(v_gs, v_ds);
        let i_d = self.i_d(v_gs, v_ds);

        let i_eq = i_d - g_ds * v_ds - g_m * v_gs;

        let mut triplets = Vec::new();

        if let Some(&is) = index_map.get(&self.source) {
            triplets.push(Triplet::new(is, 0, i_eq));
        }

        if let Some(&id) = index_map.get(&self.drain) {
            triplets.push(Triplet::new(id, 0, -i_eq));
        }
        triplets
    }

    fn stamp_excitation_vector_ac(
        &self,
        _index_map: &HashMap<String, usize>,
        _solution_map: &HashMap<String, f64>,
        _frequency: f64,
    ) -> Vec<Triplet<usize, usize, faer::c64>> {
        vec![]
    }

    fn stamp_conductance_matrix_ac(
        &self,
        _index_map: &HashMap<String, usize>,
        _solution_map: &HashMap<String, f64>,
        _frequency: f64,
    ) -> Vec<Triplet<usize, usize, faer::c64>> {
        todo!()
    }
}

// Nom parser for NMOSFET
pub fn parse_nmosfet(input: &str) -> IResult<&str, NMOSFET> {
    // Parse the initial 'MN' (case-insensitive)
    let (input, _) = tag_no_case("MN").parse(input)?;

    // Parse the numeric name part
    let (input, name) = alphanumeric_or_underscore1(input)?;

    // Parse nodes: drain, gate, source, bulk
    let (input, drain) = preceded(space1, alphanumeric_or_underscore1).parse(input)?;
    let (input, gate) = preceded(space1, alphanumeric_or_underscore1).parse(input)?;
    let (input, source) = preceded(space1, alphanumeric_or_underscore1).parse(input)?;
    let (input, bulk) = preceded(space1, alphanumeric_or_underscore1).parse(input)?; // Added bulk parser

    // Parse the required model name
    let (input, model_name) = preceded(space1, alphanumeric_or_underscore1).parse(input)?;

    // Each parameter is expected to be separated by at least one space from the previous token.
    let (input, params) = multi::many0(preceded(space1, parse_key_value)).parse(input)?;

    // consume any trailing whitespace
    let (input, _) = space0.parse(input)?;

    let mut multiplicity: Option<usize> = None;
    let mut width: Option<f64> = None;
    let mut length: Option<f64> = None;
    for (k, v) in params {
        if k.eq_ignore_ascii_case("m") {
            multiplicity = Some(v as usize);
        }

        if k.eq_ignore_ascii_case("w") {
            width = Some(v);
        }
        if k.eq_ignore_ascii_case("l") {
            length = Some(v);
        }
    }

    let mosfet = NMOSFET {
        name: name.to_string(),
        drain: drain.to_string(),
        gate: gate.to_string(),
        source: source.to_string(),
        bulk: bulk.to_string(), // Added bulk field
        model_name: model_name.to_string(),
        model: NMosfetModel::default(),
        multiplicity,
        width,
        length,
    };

    Ok((input, mosfet))
}

impl FromStr for NMOSFET {
    type Err = crate::prelude::Error;

    fn from_str(s: &str) -> Result<Self> {
        let s_without_comment = s.split(['%', '*']).next().unwrap_or("").trim();
        if s_without_comment.is_empty() {
            return Err(Error::InvalidFormat(
                "Empty line after comment removal".to_string(),
            ));
        }

        // Expected format: M<name> <drain> <gate> <source> <bulk> <model>
        match all_consuming(parse_nmosfet).parse(s_without_comment) {
            Ok((_, mosfet)) => Ok(mosfet),
            Err(nom::Err::Error(e) | nom::Err::Failure(e)) => Err(Error::InvalidFormat(format!(
                "Failed to parse MOSFET line '{}': {:?}. Expected format: M<name> D G S B <model>", // Updated error message
                s_without_comment, e.code
            ))),
            Err(nom::Err::Incomplete(_)) => Err(Error::InvalidFormat(format!(
                "Incomplete parse for MOSFET line: '{}'. Expected format: M<name> D G S B <model>", // Updated error message
                s_without_comment
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_nchannel_mosfet() {
        // Standard SPICE format: M<name> <drain> <gate> <source> <bulk> <model>
        let mosfet_str = "MN1 D G S B MyNmosModel % bla";
        let mosfet = mosfet_str.parse::<NMOSFET>().unwrap();

        assert_eq!(mosfet.name, "1");
        assert_eq!(mosfet.drain, "D");
        assert_eq!(mosfet.gate, "G");
        assert_eq!(mosfet.source, "S");
        assert_eq!(mosfet.bulk, "B"); // Check bulk node
        assert_eq!(mosfet.model_name, "MyNmosModel");
    }

    #[test]
    fn test_invalid_mosfet_format_missing_bulk() {
        let mosfet_str = "MN1 1 2 3 MyModel"; // Missing bulk node
        let result = mosfet_str.parse::<NMOSFET>();
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_mosfet_format_missing_model() {
        let mosfet_str = "MN1 1 2 3 0"; // Missing model name
        let result = mosfet_str.parse::<NMOSFET>();
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_mosfet_format_too_few_nodes() {
        let mosfet_str = "MN1 1 2 MyModel";
        let result = mosfet_str.parse::<NMOSFET>();
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_type_char() {
        let s = "MX1 1 2 3 0 MyModel";
        let result = s.parse::<NMOSFET>();
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_prefix() {
        let s = "R1 1 2 3 0 MyModel";
        let result = s.parse::<NMOSFET>();
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_mosfet_with_optional_value_removed() {
        // This format is no longer supported by the parser
        let mosfet_str = "MN2 7 8 9 0 N_Model 1.5";
        let result = mosfet_str.parse::<NMOSFET>();
        assert!(result.is_err()); // Should fail because "1.5" is an extra part
    }

    #[test]
    fn test_parse_mosfet_with_multiplicity() {
        let mosfet_str = "MN2 7 8 9 0 N_Model         m=3    ";
        let mosfet = mosfet_str.parse::<NMOSFET>().unwrap();
        assert_eq!(mosfet.multiplicity, Some(3))
    }
}
