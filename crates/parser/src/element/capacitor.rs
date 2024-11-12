use crate::{prelude::*, utils::parse_value};
use std::str::FromStr;

pub struct Capacitor {
    pub name: String,
    pub value: f64,
    pub node1: String,
    pub node2: String,
}

impl FromStr for Capacitor {
    type Err = crate::prelude::Error;

    fn from_str(s: &str) -> Result<Self> {
        let parts: Vec<&str> = s.split_whitespace().collect();

        if parts.len() != 4 {
            return Err(Error::InvalidFormat("Invalid capacitor format".to_string()));
        }

        let name = parts[0].to_string();
        let node1 = parts[1].to_string();
        let node2 = parts[2].to_string();
        let value = parse_value(parts[3])?;
        Ok(Capacitor {
            name,
            value,
            node1,
            node2,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capacitor_from_str_a() {
        let capacitor = "C1 node_a node_b 1".parse::<Capacitor>().unwrap();
        assert_eq!(capacitor.name, "C1");
        assert!((capacitor.value - 1.0).abs() < f64::EPSILON);
        assert_eq!(capacitor.node1, "node_a");
        assert_eq!(capacitor.node2, "node_b");
    }
    #[test]
    fn test_capacitor_from_str_b() {
        let capacitor = "C1 node_a node_b 1T".parse::<Capacitor>().unwrap();
        assert!((capacitor.value - 1.0 * 1e12).abs() < f64::EPSILON);
    }
}
