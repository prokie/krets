use crate::{prelude::*, utils::parse_value};
use std::str::FromStr;
pub struct Inductor {
    pub name: String,
    value: f64,
    node1: String,
    node2: String,
}

impl FromStr for Inductor {
    type Err = crate::prelude::Error;

    fn from_str(s: &str) -> Result<Self> {
        let parts: Vec<&str> = s.split_whitespace().collect();

        if parts.len() != 4 {
            return Err(Error::InvalidFormat("Invalid inductor format".to_string()));
        }

        let name = parts[0].to_string();
        let node1 = parts[1].to_string();
        let node2 = parts[2].to_string();
        let value = parse_value(parts[3])?;
        Ok(Inductor {
            name,
            value,
            node1,
            node2,
        })
    }
}