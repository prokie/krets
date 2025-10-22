use crate::{constants::THERMAL_VOLTAGE, prelude::*};

use nom::{
    IResult,
    Parser,
    branch::alt,
    bytes::complete::tag,
    character::complete::space1,
    combinator::{all_consuming, map_res, opt}, // Added map_res
    sequence::preceded,
};

#[derive(Debug, Clone)]
/// Represents a diode in a circuit.
pub struct Diode {
    /// Name of the diode.
    pub name: u32,
    /// The name of the diode model to use.
    pub model_name: String,
    /// Model parameters for the diode.
    pub options: DiodeOptions,
    /// Positive node of the diode.
    pub plus: String,
    /// Negative node of the diode.
    pub minus: String,
}

impl Identifiable for Diode {
    fn identifier(&self) -> String {
        format!("D{}", self.name)
    }
}

impl Diode {
    pub fn apply_model_parameter(&mut self, parameter: (&String, &f64)) -> Result<()> {
        match parameter.0.to_lowercase().as_str() {
            "is" => {
                self.options.saturation_current = *parameter.1;
                Ok(())
            }
            "rs" => {
                self.options.parasitic_resistance = *parameter.1;
                Ok(())
            }
            "n" => {
                self.options.emission_coefficient = *parameter.1;
                Ok(())
            }
            _ => Err(Error::InvalidModelParameter(format!(
                "Invalid parameter '{}' for Diode model",
                parameter.0
            ))),
        }
    }
}

#[derive(Debug, Clone)]
/// Options for the diode, including saturation current, parasitic resistance, and emission coefficient.
pub struct DiodeOptions {
    /// The Saturation current (Is).
    pub saturation_current: f64,
    /// The Parasitic resistance (Rs).
    pub parasitic_resistance: f64,
    /// The Emission coefficient (N).
    pub emission_coefficient: f64,
}

impl Default for DiodeOptions {
    fn default() -> Self {
        DiodeOptions {
            saturation_current: 1e-12,
            parasitic_resistance: 0.0,
            emission_coefficient: 1.0,
        }
    }
}

impl Stampable for Diode {
    fn add_conductance_matrix_dc_stamp(
        &self,
        index_map: &HashMap<String, usize>,
        solution_map: &HashMap<String, f64>,
    ) -> Vec<Triplet<usize, usize, f64>> {
        let index_plus = index_map.get(&format!("V({})", self.plus));
        let index_minus = index_map.get(&format!("V({})", self.minus));

        // The linearized conductance of the diode for the current iteration.
        let conductance = self.conductance(solution_map);

        let mut triplets = Vec::with_capacity(4);

        if let Some(&index_plus) = index_plus {
            triplets.push(Triplet::new(index_plus, index_plus, conductance));
        }
        if let Some(&index_minus) = index_minus {
            triplets.push(Triplet::new(index_minus, index_minus, conductance));
        }
        if let (Some(&index_plus), Some(&index_minus)) = (index_plus, index_minus) {
            triplets.push(Triplet::new(index_plus, index_minus, -conductance));
            triplets.push(Triplet::new(index_minus, index_plus, -conductance));
        }
        triplets
    }

    fn add_conductance_matrix_ac_stamp(
        &self,
        index_map: &HashMap<String, usize>,
        solution_map: &HashMap<String, f64>,
        _frequency: f64,
    ) -> Vec<Triplet<usize, usize, faer::c64>> {
        // FIX: Implemented AC stamp. For a diode, the small-signal AC conductance
        // at a given DC bias point is the same as its linearized DC conductance.
        let conductance = self.conductance(solution_map);
        let conductance_complex = c64::new(conductance, 0.0);

        let index_plus = index_map.get(&format!("V({})", self.plus));
        let index_minus = index_map.get(&format!("V({})", self.minus));

        let mut triplets = Vec::with_capacity(4);

        if let Some(&index_plus) = index_plus {
            triplets.push(Triplet::new(index_plus, index_plus, conductance_complex));
        }
        if let Some(&index_minus) = index_minus {
            triplets.push(Triplet::new(index_minus, index_minus, conductance_complex));
        }
        if let (Some(&index_plus), Some(&index_minus)) = (index_plus, index_minus) {
            triplets.push(Triplet::new(index_plus, index_minus, -conductance_complex));
            triplets.push(Triplet::new(index_minus, index_plus, -conductance_complex));
        }
        triplets
    }

    fn add_excitation_vector_dc_stamp(
        &self,
        index_map: &HashMap<String, usize>,
        solution_map: &HashMap<String, f64>,
    ) -> Vec<Triplet<usize, usize, f64>> {
        let index_plus = index_map.get(&format!("V({})", self.plus));
        let index_minus = index_map.get(&format!("V({})", self.minus));

        let equivalent_current = self.equivalent_current(solution_map);

        let mut triplets = Vec::with_capacity(2);

        if let Some(&index_plus) = index_plus {
            triplets.push(Triplet::new(index_plus, 0, -equivalent_current));
        }
        if let Some(&index_minus) = index_minus {
            triplets.push(Triplet::new(index_minus, 0, equivalent_current));
        }
        triplets
    }

    fn add_excitation_vector_ac_stamp(
        &self,
        _index_map: &HashMap<String, usize>,
        _solution_map: &HashMap<String, f64>,
        _frequency: f64,
    ) -> Vec<Triplet<usize, usize, faer::c64>> {
        // A diode is a passive component and does not
        // contribute to the excitation vector in small-signal AC analysis.
        vec![]
    }
}

impl Diode {
    // NOTE: This initial guess helps convergence but isn't a robust solution for all circuits.
    pub fn v_plus(&self, solution_map: &HashMap<String, f64>) -> f64 {
        *solution_map
            .get(&format!("V({})", self.plus))
            .unwrap_or(&0.5) // Consider replacing unwrap_or for robustness
    }

    pub fn v_minus(&self, solution_map: &HashMap<String, f64>) -> f64 {
        *solution_map
            .get(&format!("V({})", self.minus))
            .unwrap_or(&0.0) // Consider replacing unwrap_or for robustness
    }

    pub fn v_d(&self, solution_map: &HashMap<String, f64>) -> f64 {
        self.v_plus(solution_map) - self.v_minus(solution_map)
    }

    fn conductance(&self, solution_map: &HashMap<String, f64>) -> f64 {
        let diode_voltage = self.limit_diode_voltage(self.v_d(solution_map));
        let n = self.options.emission_coefficient;
        let is = self.options.saturation_current;

        (is / (n * THERMAL_VOLTAGE)) * f64::exp(diode_voltage / (n * THERMAL_VOLTAGE))
    }

    fn current(&self, solution_map: &HashMap<String, f64>) -> f64 {
        let diode_voltage = self.limit_diode_voltage(self.v_d(solution_map));
        let n = self.options.emission_coefficient;
        let is = self.options.saturation_current;

        is * (f64::exp(diode_voltage / (n * THERMAL_VOLTAGE)) - 1.0)
    }

    fn equivalent_current(&self, solution_map: &HashMap<String, f64>) -> f64 {
        let diode_voltage = self.v_d(solution_map);
        self.current(solution_map) - self.conductance(solution_map) * diode_voltage
    }

    // Voltage limiting function to prevent floating-point overflows
    // in the exponential function, which is a common issue in circuit simulators.
    fn limit_diode_voltage(&self, vd: f64) -> f64 {
        let n = self.options.emission_coefficient;
        let is = self.options.saturation_current;
        let v_critical = n * THERMAL_VOLTAGE * f64::ln(f64::MAX * n * THERMAL_VOLTAGE / is);
        vd.clamp(-v_critical, v_critical)
    }
}

// Updated nom parser function
fn parse_diode(input: &str) -> IResult<&str, Diode> {
    let (input, _) = alt((tag("D"), tag("d"))).parse(input)?;
    // Use map_res to parse the name directly into u32 and handle potential errors
    let (input, name) =
        map_res(alphanumeric_or_underscore1, |s: &str| s.parse::<u32>()).parse(input)?;
    let (input, plus) = preceded(space1, alphanumeric_or_underscore1).parse(input)?;
    let (input, minus) = preceded(space1, alphanumeric_or_underscore1).parse(input)?;
    let (input, model_name) = opt(preceded(space1, alphanumeric_or_underscore1)).parse(input)?;

    let diode = Diode {
        name, // Directly use the parsed u32 name
        plus: plus.to_string(),
        minus: minus.to_string(),
        model_name: model_name.unwrap_or("default").to_string(),
        options: DiodeOptions::default(),
    };

    Ok((input, diode))
}

// Updated FromStr implementation using the nom parser
impl FromStr for Diode {
    type Err = crate::prelude::Error;

    fn from_str(s: &str) -> Result<Self> {
        // Remove comments first
        let s_without_comment = s.split(['%', '*']).next().unwrap_or("").trim();

        // Use the nom parser with all_consuming to ensure the whole line is parsed
        match all_consuming(parse_diode).parse(s_without_comment) {
            Ok((_, diode)) => {
                // Additional validation (like checking if name is > 0) could go here if needed
                if diode.name == 0 {
                    return Err(Error::InvalidFormat(format!(
                        "Diode name cannot be zero: '{s}'"
                    )));
                }
                Ok(diode)
            }
            Err(nom::Err::Error(e) | nom::Err::Failure(e)) => Err(Error::InvalidFormat(format!(
                "Failed to parse diode line '{}': {:?}",
                s_without_comment, e.code
            ))),
            Err(nom::Err::Incomplete(_)) => Err(Error::InvalidFormat(format!(
                "Incomplete parse for diode line: '{}'",
                s_without_comment
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_diode() {
        let diode_str = "D1 1 0";
        let diode = diode_str.parse::<Diode>().unwrap();

        assert_eq!(diode.name, 1);
        assert_eq!(diode.plus, "1");
        assert_eq!(diode.minus, "0");
        assert_eq!(diode.model_name, "default");
    }

    #[test]
    fn test_parse_diode_with_model() {
        let diode_str = "D1 1 0 1N4148";
        let diode = diode_str.parse::<Diode>().unwrap();

        assert_eq!(diode.name, 1);
        assert_eq!(diode.plus, "1");
        assert_eq!(diode.minus, "0");
        assert_eq!(diode.model_name, "1N4148");
    }

    #[test]
    fn test_parse_diode_lowercase() {
        let diode_str = "d5 nodeA nodeB MyModel";
        let diode = diode_str.parse::<Diode>().unwrap();

        assert_eq!(diode.name, 5);
        assert_eq!(diode.plus, "nodeA");
        assert_eq!(diode.minus, "nodeB");
        assert_eq!(diode.model_name, "MyModel");
    }

    #[test]
    fn test_parse_diode_with_comment() {
        let diode_str = "D1 1 0 % This is a comment";
        let diode = diode_str.parse::<Diode>().unwrap();

        assert_eq!(diode.name, 1);
        assert_eq!(diode.plus, "1");
        assert_eq!(diode.minus, "0");
        assert_eq!(diode.model_name, "default"); // Model name becomes default if none before comment
    }

    #[test]
    fn test_parse_diode_with_model_and_comment() {
        let diode_str = "D1 1 0 DMOD % This is a comment";
        let diode = diode_str.parse::<Diode>().unwrap();

        assert_eq!(diode.name, 1);
        assert_eq!(diode.plus, "1");
        assert_eq!(diode.minus, "0");
        assert_eq!(diode.model_name, "DMOD");
    }

    #[test]
    fn test_parse_diode_with_star_comment() {
        let diode_str = "D2 out 0 Special * With star comment";
        let diode = diode_str.parse::<Diode>().unwrap();
        assert_eq!(diode.name, 2);
        assert_eq!(diode.model_name, "Special");
    }

    #[test]
    fn test_invalid_diode_format_missing_node() {
        let diode_str = "D1 1";
        let result = diode_str.parse::<Diode>();
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_diode_format_extra_token() {
        let diode_str = "D1 1 0 MyModel Extra";
        let result = diode_str.parse::<Diode>();
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_diode_name_char() {
        // The nom parser alphanumeric_or_underscore1 handles this
        let diode_str = "D! 1 0";
        let result = diode_str.parse::<Diode>();
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_diode_name_zero() {
        // Added explicit check for name == 0 in FromStr
        let diode_str = "D0 1 0";
        let result = diode_str.parse::<Diode>();
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_prefix() {
        let s = "R1 1 0 1N4148";
        let result = s.parse::<Diode>();
        assert!(result.is_err());
    }
}
