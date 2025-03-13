use krets_matrix::mna_matrix::MnaMatrix;

use crate::prelude::*;
use std::str::FromStr;

use super::{Identifiable, Stampable};

#[derive(Debug, Clone)]
/// Represents a capacitor in a circuit.
pub struct Inductor {
    /// Name of the capacitor.
    pub name: u32,
    /// Value of the capacitor.
    pub value: f64,
    /// Positive node of the capacitor.
    pub plus: String,
    /// Negative node of the capacitor.
    pub minus: String,
}

impl Identifiable for Inductor {
    fn identifier(&self) -> String {
        format!("I{}", self.name)
    }
}

impl Stampable for Inductor {
    fn add_dc_stamp(&self, mna_matrix: &mut MnaMatrix) {
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
            mna_matrix.excitation_vector[(index_current, 0)] = 0.0;
        }
    }

    fn add_ac_stamp(&self, mna_matrix: &mut MnaMatrix, _frequency: f64) {
        self.add_dc_stamp(mna_matrix);
    }
}

impl FromStr for Inductor {
    type Err = crate::prelude::Error;

    fn from_str(s: &str) -> Result<Self> {
        let mut parts: Vec<&str> = s.split_whitespace().collect();

        // if % is found ignore everything after it
        if parts.iter().any(|&x| x == "%") {
            let index = parts.iter().position(|&x| x == "%").unwrap();
            parts.truncate(index);
        }

        if parts.len() != 4 {
            return Err(Error::InvalidFormat(format!(
                "Invalid inductor format: '{}'",
                s
            )));
        }

        if parts[0].len() <= 1 {
            return Err(Error::InvalidFormat(format!(
                "Inductor name is too short: '{}'",
                s
            )));
        }

        let name = parts[0][1..]
            .parse::<u32>()
            .map_err(|_| Error::InvalidNodeName(format!("Invalid inductor name: '{}'", s)))?;
        let plus = parts[1].to_string();
        let minus = parts[2].to_string();
        let value = parts[3]
            .parse::<f64>()
            .map_err(|_| Error::InvalidFloatValue(format!("Invalid inductor value: '{}'", s)))?;

        Ok(Inductor {
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
    fn test_parse_inductor() {
        let inductor_str = "L1 1 0 0.001";
        let inductor = inductor_str.parse::<Inductor>().unwrap();

        assert_eq!(inductor.name, 1);
        assert_eq!(inductor.plus, "1");
        assert_eq!(inductor.minus, "0");
        assert_eq!(inductor.value, 0.001);
    }

    #[test]
    fn test_parse_inductor_with_comment() {
        let inductor_str = "L1 1 0 0.001 % This is a comment";
        let inductor = inductor_str.parse::<Inductor>().unwrap();

        assert_eq!(inductor.name, 1);
        assert_eq!(inductor.plus, "1");
        assert_eq!(inductor.minus, "0");
        assert_eq!(inductor.value, 0.001);
    }

    #[test]
    fn test_invalid_inductor_format() {
        let inductor_str = "L1 1 0";
        let result = inductor_str.parse::<Inductor>();
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_inductor_name() {
        let inductor_str = "L 1 0 0.001";
        let result = inductor_str.parse::<Inductor>();
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_inductor_value() {
        let inductor_str = "L1 1 0 abc";
        let result = inductor_str.parse::<Inductor>();
        assert!(result.is_err());
    }
}
