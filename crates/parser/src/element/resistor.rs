use crate::{prelude::*, utils::parse_value};
use std::str::FromStr;

pub struct Resistor {
    pub name: String,
    pub value: f64,
    pub node1: String,
    pub node2: String,
}

impl FromStr for Resistor {
    type Err = crate::prelude::Error;

    fn from_str(s: &str) -> Result<Self> {
        let parts: Vec<&str> = s.split_whitespace().collect();

        if parts.len() != 4 {
            return Err(Error::InvalidFormat("Invalid resistor format".to_string()));
        }

        if parts[0].len() <= 1 {
            return Err(Error::InvalidFormat(
                "Resistor name is too short".to_string(),
            ));
        }

        let name = parts[0].chars().skip(1).collect();
        let node1 = parts[1].to_string();
        let node2 = parts[2].to_string();
        let value = parse_value(parts[3])?;
        Ok(Resistor {
            name,
            value,
            node1,
            node2,
        })
    }
}
