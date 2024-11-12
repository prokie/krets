use crate::{prelude::*, utils::parse_value};
use std::str::FromStr;

#[derive(PartialEq, Debug)]
pub enum ChannelType {
    N,
    P,
}

impl FromStr for ChannelType {
    type Err = crate::prelude::Error;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_uppercase().as_str() {
            "N" => Ok(ChannelType::N),
            "P" => Ok(ChannelType::P),
            _ => Err(Error::InvalidFormat("Invalid channel type".to_string())),
        }
    }
}

pub struct Mosfet {
    pub name: String,
    value: f64,
    source: String,
    drain: String,
    gate: String,
    channel_type: ChannelType,
}

impl FromStr for Mosfet {
    type Err = crate::prelude::Error;

    fn from_str(s: &str) -> Result<Self> {
        let parts: Vec<&str> = s.split_whitespace().collect();

        if parts.len() != 5 {
            return Err(Error::InvalidFormat("Invalid mosfet format".to_string()));
        }

        let name = parts[0].to_string();
        let source = parts[1].to_string();
        let drain = parts[2].to_string();
        let gate = parts[3].to_string();
        let value = parse_value(parts[4])?;

        let channel_type_str = name
            .chars()
            .nth(1)
            .ok_or_else(|| Error::InvalidFormat("Invalid mosfet format".to_string()))?
            .to_string();
        let channel_type = ChannelType::from_str(&channel_type_str)?;

        Ok(Mosfet {
            name,
            value,
            source,
            drain,
            gate,
            channel_type,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mosfet_from_str_a() {
        let mosfet = "MN source drain gate 1".parse::<Mosfet>().unwrap();
        assert_eq!(mosfet.name, "MN");
        assert!((mosfet.value - 1.0).abs() < f64::EPSILON);
        assert_eq!(mosfet.source, "source");
        assert_eq!(mosfet.drain, "drain");
        assert_eq!(mosfet.gate, "gate");
        assert_eq!(mosfet.channel_type, ChannelType::N);
    }
}
