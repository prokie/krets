use crate::prelude::*;
use std::str::FromStr;

#[derive(Debug)]
/// Represents a resistor in a circuit.
pub struct Resistor {
    /// Name of the resistor.
    pub name: u32,
    /// Value of the resistor.
    pub value: f64,
    /// Positive node of the resistor.
    pub plus: String,
    /// Negative node of the resistor.
    pub minus: String,
    /// Group of the resistor.
    pub g2: Option<u32>,
}

impl FromStr for Resistor {
    type Err = crate::prelude::Error;

    fn from_str(s: &str) -> Result<Self> {
        let mut parts: Vec<&str> = s.split_whitespace().collect();

        if parts.iter().any(|&x| x == "%") {
            let index = parts.iter().position(|&x| x == "%").unwrap();
            parts.truncate(index);
        }

        if parts.len() != 4 && parts.len() != 5 {
            return Err(Error::InvalidFormat(format!(
                "Invalid resistor format: '{}'",
                s
            )));
        }

        if parts[0].len() <= 1 {
            return Err(Error::InvalidFormat(format!(
                "Resistor name is too short: '{}'",
                s
            )));
        }

        let name = parts[0][1..]
            .parse::<u32>()
            .map_err(|_| Error::InvalidNodeName(format!("Invalid resistor name: '{}'", s)))?;
        let plus = parts[1].to_string();
        let minus = parts[2].to_string();
        let value = parts[3]
            .parse::<f64>()
            .map_err(|_| Error::InvalidFloatValue(format!("Invalid resistor value: '{}'", s)))?;
        let g2 = if parts.len() == 5 {
            Some(
                parts[4][1..]
                    .parse::<u32>()
                    .map_err(|_| Error::InvalidFloatValue("Invalid group value".to_string()))?,
            )
        } else {
            None
        };

        Ok(Resistor {
            name,
            value,
            plus,
            minus,
            g2,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_resistor() {
        let resistor_str = "R1 1 0 1000";
        let resistor = resistor_str.parse::<Resistor>().unwrap();

        assert_eq!(resistor.name, 1);
        assert_eq!(resistor.plus, "1");
        assert_eq!(resistor.minus, "0");
        assert_eq!(resistor.value, 1000.0);
        assert_eq!(resistor.g2, None);
    }

    #[test]
    fn test_parse_resistor_with_group() {
        let resistor_str = "R1 1 0 1000 G2";
        let resistor = resistor_str.parse::<Resistor>().unwrap();

        assert_eq!(resistor.name, 1);
        assert_eq!(resistor.plus, "1");
        assert_eq!(resistor.minus, "0");
        assert_eq!(resistor.value, 1000.0);
        assert_eq!(resistor.g2, Some(2));
    }

    #[test]
    fn test_parse_resistor_with_comment() {
        let resistor_str = "R1 1 0 1000 % This is a comment";
        let resistor = resistor_str.parse::<Resistor>().unwrap();

        assert_eq!(resistor.name, 1);
        assert_eq!(resistor.plus, "1");
        assert_eq!(resistor.minus, "0");
        assert_eq!(resistor.value, 1000.0);
        assert_eq!(resistor.g2, None);
    }

    #[test]
    fn test_invalid_resistor_format() {
        let resistor_str = "R1 1 0";
        let result = resistor_str.parse::<Resistor>();
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_resistor_name() {
        let resistor_str = "R 1 0 1000";
        let result = resistor_str.parse::<Resistor>();
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_resistor_value() {
        let resistor_str = "R1 1 0 abc";
        let result = resistor_str.parse::<Resistor>();
        assert!(result.is_err());
    }
}
