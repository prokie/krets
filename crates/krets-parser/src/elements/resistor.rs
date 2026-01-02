use crate::prelude::*;
use std::fmt;

#[derive(Debug, Clone)]
/// Represents a resistor in a circuit.
pub struct Resistor {
    /// Name of the resistor.
    pub name: String,
    /// Value of the resistor in Ohms.
    pub value: f64,
    /// Positive node of the resistor.
    pub plus: String,
    /// Negative node of the resistor.
    pub minus: String,
    /// g2
    pub g2: bool,
}

impl Resistor {
    /// Returns the identifier of the resistor in the format `R{name}`.
    pub fn identifier(&self) -> String {
        format!("R{}", self.name)
    }
}

impl fmt::Display for Resistor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "R{} {} {} {}",
            self.name, self.plus, self.minus, self.value,
        )
    }
}
pub fn parse_resistor(input: &str) -> IResult<&str, Resistor> {
    let (input, _) = tag_no_case("R").parse(input)?;
    let (input, name) = alphanumeric_or_underscore1(input)?;
    let (input, plus) = preceded(space1, alphanumeric_or_underscore1).parse(input)?;
    let (input, minus) = preceded(space1, alphanumeric_or_underscore1).parse(input)?;
    let (input, value) = preceded(space1, value_parser).parse(input)?;

    let resistor = Resistor {
        name: name.to_string(),
        plus: plus.to_string(),
        minus: minus.to_string(),
        value,
        g2: false,
    };

    Ok((input, resistor))
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_resistor() {
        let resistor_str = "R1 1 0 1000";
        let (_, resistor) = parse_resistor(resistor_str).unwrap();

        assert_eq!(resistor.name, "1");
        assert_eq!(resistor.plus, "1");
        assert_eq!(resistor.minus, "0");
        assert_eq!(resistor.value, 1000.0);
    }

    #[test]
    fn test_parse_resistor_with_comment() {
        let resistor_str = "R1 1 0 1000";
        let (_, resistor) = parse_resistor(resistor_str).unwrap();
        assert_eq!(resistor.value, 1000.0);
    }

    #[test]
    fn test_parse_lowercase() {
        let s = "r5 2 3 1500";
        let (_, resistor) = parse_resistor(s).unwrap();
        assert_eq!(resistor.name, "5");
        assert_eq!(resistor.value, 1500.0);
    }

    #[test]
    fn test_invalid_resistor_format() {
        let resistor_str = "R1 1 0";
        let result = parse_resistor(resistor_str);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_prefix() {
        let s = "C1 1 0 1000";
        assert!(parse_resistor(s).is_err());
    }

    #[test]
    fn test_invalid_resistor_name() {
        let resistor_str = "R 1 0 1000";
        let result = parse_resistor(resistor_str);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_resistor_value() {
        let resistor_str = "R1 1 0 abc";
        let result = parse_resistor(resistor_str);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_resistor_name_long() {
        let resistor_str = "Rin 1 0 1000";
        let (_, resistor) = parse_resistor(resistor_str).unwrap();
        assert_eq!(resistor.name, "in");
    }
}
