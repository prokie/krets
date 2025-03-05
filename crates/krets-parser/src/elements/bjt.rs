use crate::prelude::*;
use std::str::FromStr;

#[derive(Debug, PartialEq)]
/// Represents the type of a BJT (Bipolar Junction Transistor).
pub enum BjtType {
    /// NPN BJT.
    NPN,
    /// PNP BJT.
    PNP,
}

#[derive(Debug)]
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

impl FromStr for BJT {
    type Err = crate::prelude::Error;

    fn from_str(s: &str) -> Result<Self> {
        let parts: Vec<&str> = s.split_whitespace().collect();

        if parts.len() != 4 && parts.len() != 5 {
            return Err(Error::InvalidFormat("Invalid BJT format".to_string()));
        }

        if parts[0].len() <= 2 {
            return Err(Error::InvalidFormat("BJT name is too short".to_string()));
        }

        let name = parts[0][2..]
            .parse::<u32>()
            .map_err(|_| Error::InvalidNodeName("Invalid BJT name".to_string()))?;
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

        let bjt_type = match parts[0].chars().nth(1).unwrap() {
            'N' => BjtType::NPN,
            'n' => BjtType::NPN,
            'P' => BjtType::PNP,
            'p' => BjtType::PNP,
            _ => return Err(Error::InvalidFormat("Invalid MOSFET format".to_string())),
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
    }

    #[test]
    fn test_invalid_bjt_format() {
        let bjt_str = "QN1 1 2";
        let result = bjt_str.parse::<BJT>();
        assert!(result.is_err());
    }
}
