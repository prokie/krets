use crate::{prelude::*, utils::parse_value};
use std::str::FromStr;

pub struct BipolarJunctionTransistor {
    pub name: String,
    pub value: f64,
    pub collector: String,
    pub base: String,
    pub emitter: String,
}

impl FromStr for BipolarJunctionTransistor {
    type Err = crate::prelude::Error;

    fn from_str(s: &str) -> Result<Self> {
        let parts: Vec<&str> = s.split_whitespace().collect();

        if parts.len() != 4 {
            return Err(Error::InvalidFormat(
                "Invalid bipolar junction transistor format".to_string(),
            ));
        }

        let name = parts[0].to_string();
        let collector = parts[1].to_string();
        let base = parts[2].to_string();
        let emitter = parts[2].to_string();
        let value = parse_value(parts[3])?;
        Ok(BipolarJunctionTransistor {
            name,
            value,
            collector,
            base,
            emitter,
        })
    }
}
