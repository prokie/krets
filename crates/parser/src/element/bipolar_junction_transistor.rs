use super::Nodes;
use crate::{prelude::*, utils::parse_value};
use std::str::FromStr;
#[derive(Clone)]
pub struct BipolarJunctionTransistor {
    pub name: String,
    pub node1: String,
    pub node2: String,
    pub node3: String,
    pub value: f64,
}

impl FromStr for BipolarJunctionTransistor {
    type Err = crate::prelude::Error;

    fn from_str(s: &str) -> Result<Self> {
        let parts: Vec<&str> = s.split_whitespace().collect();

        if parts.len() != 4 {
            return Err(Error::InvalidFormat("Invalid BJT format".to_string()));
        }

        if parts[0].len() <= 1 {
            return Err(Error::InvalidFormat("BJT name is too short".to_string()));
        }

        let name = parts[0].chars().skip(1).collect();
        let node1 = parts[1].to_string();
        let node2 = parts[2].to_string();
        let node3 = parts[3].to_string();
        let value = parse_value(parts[4])?;
        Ok(BipolarJunctionTransistor {
            name,
            node1,
            node2,
            node3,
            value,
        })
    }
}

impl Nodes for BipolarJunctionTransistor {
    fn nodes(&self) -> Vec<String> {
        vec![self.node1.clone(), self.node2.clone(), self.node3.clone()]
    }
}
