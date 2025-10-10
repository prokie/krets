use crate::prelude::*;
use std::str::FromStr;

use super::Identifiable;

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
pub struct MOSFET {
    /// Name of the MOSFET.
    pub name: u32,
    /// Drain node of the MOSFET.
    pub drain: String,
    /// Gate node of the MOSFET.
    pub gate: String,
    /// Source node of the MOSFET.
    pub source: String,
    /// Value of the MOSFET.
    pub value: Option<f64>,
    /// Type of the MOSFET.
    pub mosfet_type: MosfetType,
}

impl Identifiable for MOSFET {
    /// Returns the identifier of the MOSFET in the format `M{name}`.
    fn identifier(&self) -> String {
        format!("M{}", self.name)
    }
}

impl FromStr for MOSFET {
    type Err = crate::prelude::Error;

    fn from_str(s: &str) -> Result<Self> {
        let s_without_comment = s.split('%').next().unwrap_or("").trim();
        let parts: Vec<&str> = s_without_comment.split_whitespace().collect();

        if parts.len() != 4 && parts.len() != 5 {
            return Err(Error::InvalidFormat(format!(
                "Invalid MOSFET format: Expected 4 or 5 parts, found {}, in '{s}'",
                parts.len()
            )));
        }

        let identifier = parts[0];

        if !identifier.starts_with(['M', 'm']) {
            return Err(Error::InvalidFormat(format!(
                "Invalid MOSFET identifier: must start with 'M', in '{s}'"
            )));
        }

        if identifier.len() < 3 {
            return Err(Error::InvalidFormat(format!(
                "MOSFET name is too short: expected format M<type><name>, in '{s}'"
            )));
        }

        let mosfet_type = match identifier.chars().nth(1) {
            Some('N') | Some('n') => MosfetType::NChannel,
            Some('P') | Some('p') => MosfetType::PChannel,
            _ => {
                return Err(Error::InvalidFormat(format!(
                    "Invalid MOSFET type: expected 'N' or 'P' as the second character, in '{s}'"
                )));
            }
        };

        let name = identifier[2..]
            .parse::<u32>()
            .map_err(|_| Error::InvalidNodeName(format!("Invalid MOSFET name: '{s}'")))?;
        let drain = parts[1].to_string();
        let gate = parts[2].to_string();
        let source = parts[3].to_string();

        let value =
            if parts.len() == 5 {
                Some(parts[4].parse::<f64>().map_err(|_| {
                    Error::InvalidFloatValue(format!("Invalid MOSFET value: '{s}'"))
                })?)
            } else {
                None
            };

        Ok(MOSFET {
            name,
            drain,
            gate,
            source,
            value,
            mosfet_type,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_nchannel_mosfet() {
        let mosfet_str = "MN1 1 2 3 1.5";
        let mosfet = mosfet_str.parse::<MOSFET>().unwrap();

        assert_eq!(mosfet.name, 1);
        assert_eq!(mosfet.drain, "1");
        assert_eq!(mosfet.gate, "2");
        assert_eq!(mosfet.source, "3");
        assert_eq!(mosfet.value, Some(1.5));
        assert_eq!(mosfet.mosfet_type, MosfetType::NChannel);
    }

    #[test]
    fn test_parse_pchannel_mosfet_case_insensitive() {
        let mosfet_str = "mP1 4 5 6 2.5";
        let mosfet = mosfet_str.parse::<MOSFET>().unwrap();

        assert_eq!(mosfet.name, 1);
        assert_eq!(mosfet.drain, "4");
        assert_eq!(mosfet.gate, "5");
        assert_eq!(mosfet.source, "6");
        assert_eq!(mosfet.value, Some(2.5));
        assert_eq!(mosfet.mosfet_type, MosfetType::PChannel);
    }

    #[test]
    fn test_parse_mosfet_without_value() {
        let mosfet_str = "MN2 7 8 9";
        let mosfet = mosfet_str.parse::<MOSFET>().unwrap();

        assert_eq!(mosfet.name, 2);
        assert_eq!(mosfet.drain, "7");
        assert_eq!(mosfet.gate, "8");
        assert_eq!(mosfet.source, "9");
        assert_eq!(mosfet.value, None);
        assert_eq!(mosfet.mosfet_type, MosfetType::NChannel);
    }

    #[test]
    fn test_parse_with_comment() {
        let s = "MP3 1 2 3 % My P-Channel FET";
        let mosfet = s.parse::<MOSFET>().unwrap();
        assert_eq!(mosfet.name, 3);
        assert_eq!(mosfet.mosfet_type, MosfetType::PChannel);
    }

    #[test]
    fn test_invalid_mosfet_format() {
        let mosfet_str = "MN1 1 2";
        let result = mosfet_str.parse::<MOSFET>();
        assert!(result.is_err());
    }

    #[test]
    fn test_error_on_short_name() {
        assert!("M1 1 2 3".parse::<MOSFET>().is_err());
        assert!("M 1 2 3".parse::<MOSFET>().is_err());
    }

    #[test]
    fn test_invalid_type_char() {
        let s = "MX1 1 2 3";
        let result = s.parse::<MOSFET>();
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_prefix() {
        let s = "R1 1 2 3";
        let result = s.parse::<MOSFET>();
        assert!(result.is_err());
    }
}
