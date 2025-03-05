use crate::prelude::*;
use std::str::FromStr;

#[derive(Debug)]
/// Represents a capacitor in a circuit.
pub struct Diode {
    /// Name of the capacitor.
    pub name: u32,
    /// Value of the capacitor.
    pub value: Option<f64>,
    /// Positive node of the capacitor.
    pub plus: String,
    /// Negative node of the capacitor.
    pub minus: String,
}

impl FromStr for Diode {
    type Err = crate::prelude::Error;

    fn from_str(s: &str) -> Result<Self> {
        let mut parts: Vec<&str> = s.split_whitespace().collect();

        if parts.iter().any(|&x| x == "%") {
            let index = parts.iter().position(|&x| x == "%").unwrap();
            parts.truncate(index);
        }

        if parts.len() != 3 && parts.len() != 4 {
            return Err(Error::InvalidFormat(format!(
                "Invalid diode format: '{}'",
                s
            )));
        }

        if parts[0].len() <= 1 {
            return Err(Error::InvalidFormat(format!(
                "Diode name is too short: '{}'",
                s
            )));
        }

        let name = parts[0][1..]
            .parse::<u32>()
            .map_err(|_| Error::InvalidNodeName(format!("Invalid diode name: '{}'", s)))?;
        let plus = parts[1].to_string();
        let minus = parts[2].to_string();
        let value =
            if parts.len() == 4 {
                Some(parts[3].parse::<f64>().map_err(|_| {
                    Error::InvalidFloatValue(format!("Invalid diode value: '{}'", s))
                })?)
            } else {
                None
            };

        Ok(Diode {
            name,
            value,
            plus,
            minus,
        })
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
        assert_eq!(diode.value, None);
    }

    #[test]
    fn test_parse_diode_with_value() {
        let diode_str = "D1 1 0 0.7";
        let diode = diode_str.parse::<Diode>().unwrap();

        assert_eq!(diode.name, 1);
        assert_eq!(diode.plus, "1");
        assert_eq!(diode.minus, "0");
        assert_eq!(diode.value, Some(0.7));
    }

    #[test]
    fn test_parse_diode_with_comment() {
        let diode_str = "D1 1 0 % This is a comment";
        let diode = diode_str.parse::<Diode>().unwrap();

        assert_eq!(diode.name, 1);
        assert_eq!(diode.plus, "1");
        assert_eq!(diode.minus, "0");
        assert_eq!(diode.value, None);
    }

    #[test]
    fn test_invalid_diode_format() {
        let diode_str = "D1 1";
        let result = diode_str.parse::<Diode>();
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_diode_name() {
        let diode_str = "D 1 0";
        let result = diode_str.parse::<Diode>();
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_diode_value() {
        let diode_str = "D1 1 0 abc";
        let result = diode_str.parse::<Diode>();
        assert!(result.is_err());
    }
}
