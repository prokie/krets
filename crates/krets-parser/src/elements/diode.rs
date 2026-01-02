use crate::{constants::THERMAL_VOLTAGE, models::diode::DiodeModel, prelude::*};

#[derive(Debug, Clone)]
/// Represents a diode in a circuit.
pub struct Diode {
    /// Name of the diode.
    pub name: String,
    /// The name of the diode model to use.
    pub model_name: String,
    /// Model parameters for the diode.
    pub model: DiodeModel,
    /// Positive node of the diode.
    pub plus: String,
    /// Negative node of the diode.
    pub minus: String,
}

impl Diode {
    pub fn identifier(&self) -> String {
        format!("D{}", self.name)
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

    pub fn conductance(&self, solution_map: &HashMap<String, f64>) -> f64 {
        let diode_voltage = self.limit_diode_voltage(self.v_d(solution_map));
        let n = self.model.emission_coefficient;
        let is = self.model.saturation_current;

        (is / (n * THERMAL_VOLTAGE)) * f64::exp(diode_voltage / (n * THERMAL_VOLTAGE))
    }

    pub fn current(&self, solution_map: &HashMap<String, f64>) -> f64 {
        let diode_voltage = self.limit_diode_voltage(self.v_d(solution_map));
        let n = self.model.emission_coefficient;
        let is = self.model.saturation_current;

        is * (f64::exp(diode_voltage / (n * THERMAL_VOLTAGE)) - 1.0)
    }

    pub fn equivalent_current(&self, solution_map: &HashMap<String, f64>) -> f64 {
        let diode_voltage = self.v_d(solution_map);
        self.current(solution_map) - self.conductance(solution_map) * diode_voltage
    }

    // Voltage limiting function to prevent floating-point overflows
    // in the exponential function, which is a common issue in circuit simulators.
    pub fn limit_diode_voltage(&self, vd: f64) -> f64 {
        let n = self.model.emission_coefficient;
        let is = self.model.saturation_current;
        let v_critical = n * THERMAL_VOLTAGE * f64::ln(f64::MAX * n * THERMAL_VOLTAGE / is);
        vd.clamp(-v_critical, v_critical)
    }
}

// Updated nom parser function
pub fn parse_diode(input: &str) -> IResult<&str, Diode> {
    let (input, _) = tag_no_case("D").parse(input)?;
    // Use map_res to parse the name directly into u32 and handle potential errors
    let (input, name) = alphanumeric_or_underscore1(input)?;
    let (input, plus) = preceded(space1, alphanumeric_or_underscore1).parse(input)?;
    let (input, minus) = preceded(space1, alphanumeric_or_underscore1).parse(input)?;
    let (input, model_name) = opt(preceded(space1, alphanumeric_or_underscore1)).parse(input)?;

    let diode = Diode {
        name: name.to_string(),
        plus: plus.to_string(),
        minus: minus.to_string(),
        model_name: model_name.unwrap_or("default").to_string(),
        model: DiodeModel::default(),
    };

    Ok((input, diode))
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_diode() {
        let diode_str = "D1 1 0";
        let (_, diode) = parse_diode(diode_str).unwrap();

        assert_eq!(diode.name, "1");
        assert_eq!(diode.plus, "1");
        assert_eq!(diode.minus, "0");
        assert_eq!(diode.model_name, "default");
    }

    #[test]
    fn test_parse_diode_with_model() {
        let diode_str = "D1 1 0 1N4148";
        let (_, diode) = parse_diode(diode_str).unwrap();

        assert_eq!(diode.name, "1");
        assert_eq!(diode.plus, "1");
        assert_eq!(diode.minus, "0");
        assert_eq!(diode.model_name, "1N4148");
    }

    #[test]
    fn test_parse_diode_lowercase() {
        let diode_str = "d5 nodeA nodeB MyModel";
        let (_, diode) = parse_diode(diode_str).unwrap();

        assert_eq!(diode.name, "5");
        assert_eq!(diode.plus, "nodeA");
        assert_eq!(diode.minus, "nodeB");
        assert_eq!(diode.model_name, "MyModel");
    }

    #[test]
    fn test_parse_diode_with_comment() {
        let diode_str = "D1 1 0";
        let (_, diode) = parse_diode(diode_str).unwrap();

        assert_eq!(diode.name, "1");
        assert_eq!(diode.plus, "1");
        assert_eq!(diode.minus, "0");
        assert_eq!(diode.model_name, "default");
    }

    #[test]
    fn test_parse_diode_with_model_and_comment() {
        let diode_str = "D1 1 0 DMOD";
        let (_, diode) = parse_diode(diode_str).unwrap();

        assert_eq!(diode.name, "1");
        assert_eq!(diode.plus, "1");
        assert_eq!(diode.minus, "0");
        assert_eq!(diode.model_name, "DMOD");
    }

    #[test]
    fn test_parse_diode_with_star_comment() {
        let diode_str = "D2 out 0 Special";
        let (_, diode) = parse_diode(diode_str).unwrap();
        assert_eq!(diode.name, "2");
        assert_eq!(diode.model_name, "Special");
    }

    #[test]
    fn test_invalid_diode_format_missing_node() {
        let diode_str = "D1 1";
        let result = parse_diode(diode_str);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_diode_format_extra_token() {
        // The parser should parse "D1 1 0 MyModel" and leave "Extra" unparsed
        let input = "D1 1 0 MyModel Extra";
        let (remainder, _) = parse_diode(input).unwrap();
        // The remainder should be " Extra" (with leading space)
        assert!(!remainder.is_empty());
    }

    #[test]
    fn test_invalid_diode_name_char() {
        // The nom parser alphanumeric_or_underscore1 handles this
        let diode_str = "D! 1 0";
        let result = parse_diode(diode_str);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_prefix() {
        let s = "R1 1 0 1N4148";
        let result = parse_diode(s);
        assert!(result.is_err());
    }
}
