use krets_matrix::mna_matrix::MnaMatrix;

use crate::prelude::*;
use std::fmt;
use std::str::FromStr;

use super::{Identifiable, Stampable};

#[derive(Debug, Clone)]
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

impl Identifiable for VoltageSource {
    fn identifier(&self) -> String {
        format!("V{}", self.name)
    }
}

impl Stampable for VoltageSource {
    fn add_stamp(&self, mna_matrix: &mut MnaMatrix) {
        let index_plus = mna_matrix.index_map.get(&format!("V({})", self.plus));
        let index_minus = mna_matrix.index_map.get(&format!("V({})", self.minus));
        let index_current = mna_matrix
            .index_map
            .get(&format!("I({})", self.identifier()));

        if let (Some(&index_plus), Some(&index_current)) = (index_plus, index_current) {
            mna_matrix.conductance_matrix[(index_plus, index_current)] = 1.0;
            mna_matrix.conductance_matrix[(index_current, index_plus)] = 1.0;
        }

        if let (Some(&index_minus), Some(&index_current)) = (index_minus, index_current) {
            mna_matrix.conductance_matrix[(index_minus, index_current)] = -1.0;
            mna_matrix.conductance_matrix[(index_current, index_minus)] = -1.0;
        }

        if let Some(&index_current) = index_current {
            mna_matrix.excitation_vector[(index_current, 0)] = self.value;
        }
    }
}

impl fmt::Display for VoltageSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "V{} {} {} {}",
            self.name, self.plus, self.minus, self.value,
        )
    }
}

impl FromStr for VoltageSource {
    type Err = crate::prelude::Error;

    fn from_str(s: &str) -> Result<Self> {
        let mut parts: Vec<&str> = s.split_whitespace().collect();

        if parts.iter().any(|&x| x == "%") {
            let index = parts.iter().position(|&x| x == "%").unwrap();
            parts.truncate(index);
        }

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_voltage_source() {
        let voltage_source_str = "V1 1 0 5";
        let voltage_source = voltage_source_str.parse::<VoltageSource>().unwrap();

        assert_eq!(voltage_source.name, 1);
        assert_eq!(voltage_source.plus, "1");
        assert_eq!(voltage_source.minus, "0");
        assert_eq!(voltage_source.value, 5.0);
    }

    #[test]
    fn test_parse_voltage_source_with_comment() {
        let voltage_source_str = "V1 1 0 5 % This is a comment";
        let voltage_source = voltage_source_str.parse::<VoltageSource>().unwrap();

        assert_eq!(voltage_source.name, 1);
        assert_eq!(voltage_source.plus, "1");
        assert_eq!(voltage_source.minus, "0");
        assert_eq!(voltage_source.value, 5.0);
    }

    #[test]
    fn test_invalid_voltage_source_format() {
        let voltage_source_str = "V1 1 0";
        let result = voltage_source_str.parse::<VoltageSource>();
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_voltage_source_name() {
        let voltage_source_str = "V 1 0 5";
        let result = voltage_source_str.parse::<VoltageSource>();
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_voltage_source_value() {
        let voltage_source_str = "V1 1 0 abc";
        let result = voltage_source_str.parse::<VoltageSource>();
        assert!(result.is_err());
    }
}
