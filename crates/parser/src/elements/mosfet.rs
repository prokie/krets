use crate::prelude::*;
use std::str::FromStr;

#[derive(Debug, PartialEq)]
/// Represents the type of a MOSFET (Metal-Oxide-Semiconductor Field-Effect Transistor).
/// A MOSFET can be either an N-Channel or a P-Channel.
pub enum MosfetType {
    /// N-Channel MOSFET.
    NChannel,
    /// P-Channel MOSFET.
    PChannel,
}

#[derive(Debug)]
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

impl FromStr for MOSFET {
    type Err = crate::prelude::Error;

    fn from_str(s: &str) -> Result<Self> {
        let parts: Vec<&str> = s.split_whitespace().collect();

        if parts.len() != 4 && parts.len() != 5 {
            return Err(Error::InvalidFormat("Invalid MOSFET format".to_string()));
        }

        if parts[0].len() <= 2 {
            return Err(Error::InvalidFormat("MOSFET name is too short".to_string()));
        }

        let name = parts[0][2..]
            .parse::<u32>()
            .map_err(|_| Error::InvalidNodeName("Invalid MOSFET name".to_string()))?;
        let drain = parts[1].to_string();
        let gate = parts[2].to_string();
        let source = parts[3].to_string();
        let value = if parts.len() == 5 {
            Some(
                parts[4]
                    .parse::<f64>()
                    .map_err(|_| Error::InvalidFloatValue("Invalid MOSFET value".to_string()))?,
            )
        } else {
            None
        };

        let mosfet_type = match parts[0].chars().nth(1).unwrap() {
            'N' => MosfetType::NChannel,
            'n' => MosfetType::NChannel,
            'P' => MosfetType::PChannel,
            'p' => MosfetType::PChannel,
            _ => return Err(Error::InvalidFormat("Invalid MOSFET format".to_string())),
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
    fn test_parse_pchannel_mosfet() {
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
    fn test_invalid_mosfet_format() {
        let mosfet_str = "MN1 1 2";
        let result = mosfet_str.parse::<MOSFET>();
        assert!(result.is_err());
    }
}
