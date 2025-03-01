use crate::prelude::*;
use std::str::FromStr;

#[derive(Debug)]
/// Represents a voltage source in a circuit.
pub struct VoltageSource {
    /// Name of the voltage source.
    pub name: u32,
    /// Value of the voltage source.
    pub value: f64,
    /// Positive node of the voltage source.
    pub plus: String,
    /// Negative node of the voltage source.
    pub minus: String,
}

impl FromStr for VoltageSource {
    type Err = crate::prelude::Error;

    fn from_str(s: &str) -> Result<Self> {
        let parts: Vec<&str> = s.split_whitespace().collect();

        if parts.len() != 4 {
            return Err(Error::InvalidFormat(
                "Invalid voltage source format".to_string(),
            ));
        }

        if parts[0].len() <= 1 {
            return Err(Error::InvalidFormat(
                "Voltage source name is too short".to_string(),
            ));
        }

        let name = parts[0][1..]
            .parse::<u32>()
            .map_err(|_| Error::InvalidNodeName("Invalid voltage source name".to_string()))?;
        let plus = parts[1].to_string();
        let minus = parts[2].to_string();
        let value = parts[3]
            .parse::<f64>()
            .map_err(|_| Error::InvalidFloatValue("Invalid voltage source value".to_string()))?;
        Ok(VoltageSource {
            name,
            value,
            plus,
            minus,
        })
    }
}
