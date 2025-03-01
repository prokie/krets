use crate::prelude::*;
use std::str::FromStr;

#[derive(Debug)]
/// Represents a current source in a circuit.
pub struct CurrentSource {
    /// The name of the current source.
    pub name: u32,
    /// The value of the current source.
    pub value: f64,
    /// The positive node of the current source.
    pub plus: String,
    /// The negative node of the current source.
    pub minus: String,
    // The transconductance value for the current source.
    pub g2: Option<f64>,
}

impl FromStr for CurrentSource {
    type Err = crate::prelude::Error;

    fn from_str(s: &str) -> Result<Self> {
        let parts: Vec<&str> = s.split_whitespace().collect();

        if parts.len() != 4 && parts.len() != 5 {
            return Err(Error::InvalidFormat(
                "Invalid current source format".to_string(),
            ));
        }

        if parts[0].len() <= 1 {
            return Err(Error::InvalidFormat(
                "Current source name is too short".to_string(),
            ));
        }

        let name = parts[0][1..]
            .parse::<u32>()
            .map_err(|_| Error::InvalidNodeName("Invalid current source name".to_string()))?;
        let plus = parts[1].to_string();
        let minus = parts[2].to_string();
        let value = parts[3]
            .parse::<f64>()
            .map_err(|_| Error::InvalidFloatValue("Invalid current source value".to_string()))?;

        let g2 = if parts.len() == 5 {
            Some(parts[4].parse::<f64>().map_err(|_| {
                Error::InvalidFloatValue("Invalid transconductance value".to_string())
            })?)
        } else {
            None
        };

        Ok(CurrentSource {
            name,
            value,
            plus,
            minus,
            g2,
        })
    }
}
