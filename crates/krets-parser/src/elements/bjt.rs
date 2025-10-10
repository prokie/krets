use crate::prelude::*;
use std::str::FromStr;

use super::Identifiable;

#[derive(Debug, PartialEq, Clone)]
/// Represents the type of a BJT (Bipolar Junction Transistor).
pub enum BjtType {
    /// NPN BJT.
    NPN,
    /// PNP BJT.
    PNP,
}

#[derive(Debug, Clone)]
/// Represents a BJT (Bipolar Junction Transistor) in a circuit.
pub struct BJT {
    /// Name of the BJT.
    pub name: u32,
    /// Collector node of the BJT.
    pub collector: String,
    /// Base node of the BJT.
    pub base: String,
    /// Emitter node of the BJT.
    pub emitter: String,
    /// Value of the BJT.
    pub value: Option<f64>,
    /// Type of the BJT.
    pub bjt_type: BjtType,
}

impl Identifiable for BJT {
    /// Returns the identifier of the BJT in the format `Q{name}`.
    fn identifier(&self) -> String {
        format!("Q{}", self.name)
    }
}

impl FromStr for BJT {
    type Err = crate::prelude::Error;

    fn from_str(s: &str) -> Result<Self> {
        let parts: Vec<&str> = s.split_whitespace().collect();

        if parts.len() < 4 || parts.len() > 5 {
            return Err(Error::InvalidFormat(format!(
                "Invalid BJT format: Expected 4 or 5 parts, but found {}",
                parts.len()
            )));
        }

        let identifier = parts[0];
        if !identifier.starts_with(['Q', 'q']) {
            return Err(Error::InvalidFormat(
                "Invalid BJT identifier: Must start with 'Q'.".to_string(),
            ));
        }

        let bjt_type = match identifier.chars().nth(1) {
            Some('N') | Some('n') => BjtType::NPN,
            Some('P') | Some('p') => BjtType::PNP,
            _ => {
                // FIX: Corrected the error message from "MOSFET" to "BJT".
                return Err(Error::InvalidFormat(
                    "Invalid BJT type in identifier. Expected 'N' or 'P' after 'Q'.".to_string(),
                ));
            }
        };

        // This check is now safer because we've already confirmed the first two chars
        if identifier.len() < 3 {
            return Err(Error::InvalidFormat(
                "Invalid BJT identifier: Missing number after type.".to_string(),
            ));
        }

        let name = identifier[2..]
            .parse::<u32>()
            .map_err(|_| Error::InvalidNodeName("Invalid BJT name number".to_string()))?;

        let collector = parts[1].to_string();
        let base = parts[2].to_string();
        let emitter = parts[3].to_string();

        let value = if parts.len() == 5 {
            Some(
                parts[4]
                    .parse::<f64>()
                    .map_err(|_| Error::InvalidFloatValue("Invalid BJT value".to_string()))?,
            )
        } else {
            None
        };

        Ok(BJT {
            name,
            collector,
            base,
            emitter,
            value,
            bjt_type,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_npn_bjt() {
        let bjt_str = "QN1 1 2 3 0.7";
        let bjt = bjt_str.parse::<BJT>().unwrap();

        assert_eq!(bjt.name, 1);
        assert_eq!(bjt.collector, "1");
        assert_eq!(bjt.base, "2");
        assert_eq!(bjt.emitter, "3");
        assert_eq!(bjt.value, Some(0.7));
        assert_eq!(bjt.bjt_type, BjtType::NPN);
        assert_eq!(bjt.identifier(), "Q1");
    }

    #[test]
    fn test_parse_pnp_bjt() {
        let bjt_str = "QP1 4 5 6 0.8";
        let bjt = bjt_str.parse::<BJT>().unwrap();

        assert_eq!(bjt.name, 1);
        assert_eq!(bjt.collector, "4");
        assert_eq!(bjt.base, "5");
        assert_eq!(bjt.emitter, "6");
        assert_eq!(bjt.value, Some(0.8));
        assert_eq!(bjt.bjt_type, BjtType::PNP);
        assert_eq!(bjt.identifier(), "Q1");
    }

    #[test]
    fn test_parse_bjt_without_value() {
        let bjt_str = "QN2 7 8 9";
        let bjt = bjt_str.parse::<BJT>().unwrap();

        assert_eq!(bjt.name, 2);
        assert_eq!(bjt.collector, "7");
        assert_eq!(bjt.base, "8");
        assert_eq!(bjt.emitter, "9");
        assert_eq!(bjt.value, None);
        assert_eq!(bjt.bjt_type, BjtType::NPN);
        assert_eq!(bjt.identifier(), "Q2");
    }

    #[test]
    fn test_invalid_bjt_format_parts() {
        let bjt_str = "QN1 1 2";
        let result = bjt_str.parse::<BJT>();
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_bjt_type() {
        let bjt_str = "QX1 1 2 3";
        let result = bjt_str.parse::<BJT>();
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_bjt_name() {
        let bjt_str = "QNa 1 2 3";
        let result = bjt_str.parse::<BJT>();
        assert!(result.is_err());
    }

    #[test]
    fn test_malformed_identifier() {
        let bjt_str = "Q 1 2 3";
        let result = bjt_str.parse::<BJT>();
        assert!(result.is_err());
    }
}
