use crate::prelude::*;

use nom::{
    IResult, Parser,
    branch::alt,
    bytes::complete::tag,
    character::complete::{digit1, space0, space1},
    combinator::{all_consuming, map_res},
    multi,
    sequence::preceded,
};

#[derive(Debug, PartialEq, Clone)]
/// Represents the type of a MOSFET (Metal-Oxide-Semiconductor Field-Effect Transistor).
/// A MOSFET can be either an N-Channel or a P-Channel.
pub enum MosfetType {
    /// N-Channel MOSFET.
    NChannel,
    /// P-Channel MOSFET.
    PChannel,
}

#[derive(Debug, Clone)]
/// Represents a MOSFET (Metal-Oxide-Semiconductor Field-Effect Transistor) in a circuit.
/// SPICE format: M<name> <drain> <gate> <source> <bulk/substrate> <model> [parameters...]
pub struct MOSFET {
    /// Name of the MOSFET (numeric part).
    pub name: u32,
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
    /// Type of the MOSFET (inferred from name, e.g., MN or MP).
    pub mosfet_type: MosfetType,
    /// Multiplicity factor. Simulates "m" parallel devices
    pub multiplicity: Option<usize>,
    /// Width of the MOSFET.
    pub width: Option<f64>,
    /// Length of the MOSFET.
    pub length: Option<f64>,
}

impl MOSFET {
    fn threshold_voltage(&self) -> f64 {
        // Placeholder: In a real implementation, this would look up the model parameters.
        0.0
    }

    fn beta_parameter(&self) -> f64 {
        // Placeholder: In a real implementation, this would look up the model parameters.
        0.1
    }

    fn lambda_parameter(&self) -> f64 {
        // Placeholder: In a real implementation, this would look up the model parameters.
        0.0
    }
}

impl Identifiable for MOSFET {
    /// Returns the identifier of the MOSFET in the format `M{name}`.
    fn identifier(&self) -> String {
        format!("M{}", self.name)
    }
}

impl Stampable for MOSFET {
    fn add_conductance_matrix_dc_stamp(
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
        let v_th = self.threshold_voltage();
        let beta = self.beta_parameter();
        let lambda = self.lambda_parameter();

        let mut g_ds = 0.0;
        let mut g_m = 0.0;

        let mut triplets = Vec::new();

        if v_gs <= v_th {
            // Cut-off region: No conduction
            g_ds = 0.0;
            g_m = 0.0;
        } else if v_ds >= 0.0 && v_ds <= (v_gs - v_th) {
            // Linear region
            g_ds = beta * (v_gs - v_th - v_ds);
            g_m = beta * v_ds;
        } else if v_ds >= (v_gs - v_th) && v_ds >= 0.0 {
            // Saturation region
            g_ds = (beta / 2.0) * lambda * (v_gs - v_th).powi(2);
            g_m = beta * (v_gs - v_th) * (1.0 + lambda * v_ds);
        }

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

    fn add_excitation_vector_dc_stamp(
        &self,
        _index_map: &HashMap<String, usize>,
        _solution_map: &HashMap<String, f64>,
    ) -> Vec<Triplet<usize, usize, f64>> {
        todo!()
    }

    fn add_excitation_vector_ac_stamp(
        &self,
        _index_map: &HashMap<String, usize>,
        _solution_map: &HashMap<String, f64>,
        _frequency: f64,
    ) -> Vec<Triplet<usize, usize, faer::c64>> {
        vec![]
    }

    fn add_conductance_matrix_ac_stamp(
        &self,
        _index_map: &HashMap<String, usize>,
        _solution_map: &HashMap<String, f64>,
        _frequency: f64,
    ) -> Vec<Triplet<usize, usize, faer::c64>> {
        todo!()
    }
}

// Nom parser for MOSFET (updated for bulk node)
fn parse_mosfet(input: &str) -> IResult<&str, MOSFET> {
    // Parse the initial 'M' (case-insensitive)
    let (input, _) = alt((tag("M"), tag("m"))).parse(input)?;

    // Parse the type character (N or P, case-insensitive) - kept for structuring data
    let (input, type_char) = alt((tag("N"), tag("n"), tag("P"), tag("p"))).parse(input)?;
    let mosfet_type = match type_char.to_ascii_uppercase().as_str() {
        "N" => MosfetType::NChannel,
        "P" => MosfetType::PChannel,
        _ => unreachable!(),
    };

    // Parse the numeric name part
    let (input, name) = map_res(digit1, |s: &str| s.parse::<u32>()).parse(input)?;

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

    let mosfet = MOSFET {
        name,
        drain: drain.to_string(),
        gate: gate.to_string(),
        source: source.to_string(),
        bulk: bulk.to_string(), // Added bulk field
        model_name: model_name.to_string(),
        mosfet_type,
        multiplicity,
        width,
        length,
    };

    Ok((input, mosfet))
}

impl FromStr for MOSFET {
    type Err = crate::prelude::Error;

    fn from_str(s: &str) -> Result<Self> {
        let s_without_comment = s.split(['%', '*']).next().unwrap_or("").trim();
        if s_without_comment.is_empty() {
            return Err(Error::InvalidFormat(
                "Empty line after comment removal".to_string(),
            ));
        }

        // Expected format: M<name> <drain> <gate> <source> <bulk> <model>
        match all_consuming(parse_mosfet).parse(s_without_comment) {
            Ok((_, mosfet)) => {
                if mosfet.name == 0 {
                    return Err(Error::InvalidFormat(format!(
                        "MOSFET name cannot be zero: '{s}'"
                    )));
                }
                Ok(mosfet)
            }
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
        let mosfet_str = "MN1 D G S B MyNmosModel"; // Added bulk node 'B'
        let mosfet = mosfet_str.parse::<MOSFET>().unwrap();

        assert_eq!(mosfet.name, 1);
        assert_eq!(mosfet.drain, "D");
        assert_eq!(mosfet.gate, "G");
        assert_eq!(mosfet.source, "S");
        assert_eq!(mosfet.bulk, "B"); // Check bulk node
        assert_eq!(mosfet.model_name, "MyNmosModel");
        assert_eq!(mosfet.mosfet_type, MosfetType::NChannel);
    }

    #[test]
    fn test_parse_pchannel_mosfet_case_insensitive() {
        let mosfet_str = "mp5 4 5 6 0 PModel"; // Lowercase 'm' and 'p', bulk tied to ground (0)
        let mosfet = mosfet_str.parse::<MOSFET>().unwrap();

        assert_eq!(mosfet.name, 5);
        assert_eq!(mosfet.drain, "4");
        assert_eq!(mosfet.gate, "5");
        assert_eq!(mosfet.source, "6");
        assert_eq!(mosfet.bulk, "0"); // Check bulk node
        assert_eq!(mosfet.model_name, "PModel");
        assert_eq!(mosfet.mosfet_type, MosfetType::PChannel);
    }

    #[test]
    fn test_parse_with_comment() {
        let s = "MP3 1 2 3 0 P_FET % My P-Channel FET"; // Added bulk node 0
        let mosfet = s.parse::<MOSFET>().unwrap();
        assert_eq!(mosfet.name, 3);
        assert_eq!(mosfet.mosfet_type, MosfetType::PChannel);
        assert_eq!(mosfet.model_name, "P_FET");
        assert_eq!(mosfet.bulk, "0");
        assert_eq!(mosfet.multiplicity, None);
    }

    #[test]
    fn test_invalid_mosfet_format_missing_bulk() {
        let mosfet_str = "MN1 1 2 3 MyModel"; // Missing bulk node
        let result = mosfet_str.parse::<MOSFET>();
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_mosfet_format_missing_model() {
        let mosfet_str = "MN1 1 2 3 0"; // Missing model name
        let result = mosfet_str.parse::<MOSFET>();
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_mosfet_format_too_few_nodes() {
        let mosfet_str = "MN1 1 2 MyModel";
        let result = mosfet_str.parse::<MOSFET>();
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_type_char() {
        let s = "MX1 1 2 3 0 MyModel";
        let result = s.parse::<MOSFET>();
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_name_not_numeric() {
        let s = "MNA 1 2 3 0 MyModel";
        let result = s.parse::<MOSFET>();
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_prefix() {
        let s = "R1 1 2 3 0 MyModel";
        let result = s.parse::<MOSFET>();
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_mosfet_name_zero() {
        let mosfet_str = "MN0 1 2 3 0 MyModel";
        let result = mosfet_str.parse::<MOSFET>();
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_mosfet_with_optional_value_removed() {
        // This format is no longer supported by the parser
        let mosfet_str = "MN2 7 8 9 0 N_Model 1.5";
        let result = mosfet_str.parse::<MOSFET>();
        assert!(result.is_err()); // Should fail because "1.5" is an extra part
    }

    #[test]
    fn test_parse_mosfet_with_multiplicity() {
        let mosfet_str = "MN2 7 8 9 0 N_Model         m=3    ";
        let mosfet = mosfet_str.parse::<MOSFET>().unwrap();
        assert_eq!(mosfet.multiplicity, Some(3))
    }
}
