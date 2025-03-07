use krets_matrix::mna_matrix::MnaMatrix;

use crate::prelude::*;
use std::fmt;
use std::str::FromStr;

use super::{Identifiable, Stampable};

#[derive(Debug, Clone)]
/// Represents a resistor in a circuit.
pub struct Resistor {
    /// Name of the resistor.
    pub name: u32,
    /// Value of the resistor.
    pub value: f64,
    /// Positive node of the resistor.
    pub plus: String,
    /// Negative node of the resistor.
    pub minus: String,
    /// If the resistor is G2.
    pub g2: bool,
}

impl Stampable for Resistor {
    /// Adds the resistor's stamp to the given matrix.
    ///
    /// # Parameters
    /// - `conductance_matrix`: The conducatance matrix to update.
    /// - `node_map`: A map from node names to matrix indices.
    fn add_dc_stamp(&self, mna_matrix: &mut MnaMatrix) {
        let index_plus = mna_matrix.index_map.get(&format!("V({})", self.plus));
        let index_minus = mna_matrix.index_map.get(&format!("V({})", self.minus));

        if let Some(&index_plus) = index_plus {
            mna_matrix.conductance_matrix[(index_plus, index_plus)] += 1. / self.value;
        }

        if let Some(&index_minus) = index_minus {
            mna_matrix.conductance_matrix[(index_minus, index_minus)] += 1. / self.value;
        }

        if let (Some(&index_plus), Some(&index_minus)) = (index_plus, index_minus) {
            mna_matrix.conductance_matrix[(index_plus, index_minus)] -= 1. / self.value;
            mna_matrix.conductance_matrix[(index_minus, index_plus)] -= 1. / self.value;
        }
    }
}

impl Identifiable for Resistor {
    /// Returns the identifier of the resistor in the format `R{name}`.
    fn identifier(&self) -> String {
        format!("R{}", self.name)
    }
}

impl fmt::Display for Resistor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "R{} {} {} {}{}",
            self.name,
            self.plus,
            self.minus,
            self.value,
            if self.g2 { " G2" } else { "" }
        )
    }
}

impl FromStr for Resistor {
    type Err = crate::prelude::Error;

    fn from_str(s: &str) -> Result<Self> {
        let mut parts: Vec<&str> = s.split_whitespace().collect();

        if parts.iter().any(|&x| x == "%") {
            let index = parts.iter().position(|&x| x == "%").unwrap();
            parts.truncate(index);
        }

        if parts.len() != 4 && parts.len() != 5 {
            return Err(Error::InvalidFormat(format!(
                "Invalid resistor format: '{}'",
                s
            )));
        }

        if parts[0].len() <= 1 {
            return Err(Error::InvalidFormat(format!(
                "Resistor name is too short: '{}'",
                s
            )));
        }

        let name = parts[0][1..]
            .parse::<u32>()
            .map_err(|_| Error::InvalidNodeName(format!("Invalid resistor name: '{}'", s)))?;
        let plus = parts[1].to_string();
        let minus = parts[2].to_string();
        let value = parts[3]
            .parse::<f64>()
            .map_err(|_| Error::InvalidFloatValue(format!("Invalid resistor value: '{}'", s)))?;
        let g2 = parts.len() == 5 && parts[4] == "G2";

        Ok(Resistor {
            name,
            value,
            plus,
            minus,
            g2,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_resistor() {
        let resistor_str = "R1 1 0 1000";
        let resistor = resistor_str.parse::<Resistor>().unwrap();

        assert_eq!(resistor.name, 1);
        assert_eq!(resistor.plus, "1");
        assert_eq!(resistor.minus, "0");
        assert_eq!(resistor.value, 1000.0);
        assert!(!resistor.g2);
    }

    #[test]
    fn test_parse_resistor_with_group() {
        let resistor_str = "R1 1 0 1000 G2";
        let resistor = resistor_str.parse::<Resistor>().unwrap();

        assert_eq!(resistor.name, 1);
        assert_eq!(resistor.plus, "1");
        assert_eq!(resistor.minus, "0");
        assert_eq!(resistor.value, 1000.0);
        assert!(resistor.g2);
    }

    #[test]
    fn test_parse_resistor_with_comment() {
        let resistor_str = "R1 1 0 1000 % This is a comment";
        let resistor = resistor_str.parse::<Resistor>().unwrap();

        assert_eq!(resistor.name, 1);
        assert_eq!(resistor.plus, "1");
        assert_eq!(resistor.minus, "0");
        assert_eq!(resistor.value, 1000.0);
        assert!(!resistor.g2);
    }

    #[test]
    fn test_invalid_resistor_format() {
        let resistor_str = "R1 1 0";
        let result = resistor_str.parse::<Resistor>();
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_resistor_name() {
        let resistor_str = "R 1 0 1000";
        let result = resistor_str.parse::<Resistor>();
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_resistor_value() {
        let resistor_str = "R1 1 0 abc";
        let result = resistor_str.parse::<Resistor>();
        assert!(result.is_err());
    }
}
