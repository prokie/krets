use faer::sparse::Triplet;

use super::{Identifiable, Stampable};
use crate::prelude::*;
use faer::c64;
use std::collections::HashMap;
use std::fmt;
use std::str::FromStr;

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
}

impl Stampable for Resistor {
    fn conductance_matrix_dc_stamp(
        &self,
        index_map: &HashMap<String, usize>,
        _solution_map: &HashMap<String, f64>,
    ) -> Vec<Triplet<usize, usize, f64>> {
        let index_plus = index_map.get(&format!("V({})", self.plus));
        let index_minus = index_map.get(&format!("V({})", self.minus));
        let index_current = index_map.get(&format!("I({})", self.identifier()));

        let mut triplets = Vec::with_capacity(5);

        if let (Some(&index_plus), Some(&index_current)) = (index_plus, index_current) {
            triplets.push(Triplet {
                row: index_plus,
                col: index_current,
                val: 1.0,
            });

            triplets.push(Triplet {
                row: index_current,
                col: index_plus,
                val: 1.0,
            });
        }

        if let (Some(&index_minus), Some(&index_current)) = (index_minus, index_current) {
            triplets.push(Triplet {
                row: index_minus,
                col: index_current,
                val: -1.0,
            });

            triplets.push(Triplet {
                row: index_current,
                col: index_minus,
                val: -1.0,
            });
        }

        if let Some(&index_current) = index_current {
            triplets.push(Triplet {
                row: index_current,
                col: index_current,
                val: -self.value,
            });
        }

        triplets
    }

    fn conductance_matrix_ac_stamp(
        &self,
        index_map: &HashMap<String, usize>,
        _solution_map: &HashMap<String, f64>,
        _frequency: f64,
    ) -> Vec<Triplet<usize, usize, c64>> {
        let index_plus = index_map.get(&format!("V({})", self.plus));
        let index_minus = index_map.get(&format!("V({})", self.minus));
        let mut triplets = Vec::with_capacity(5);

        let index_current = index_map.get(&format!("I({})", self.identifier()));

        if let (Some(&index_plus), Some(&index_current)) = (index_plus, index_current) {
            triplets.push(Triplet {
                row: index_plus,
                col: index_current,
                val: c64 { im: 0.0, re: 1.0 },
            });

            triplets.push(Triplet {
                row: index_current,
                col: index_plus,
                val: c64 { im: 0.0, re: 1.0 },
            });
        }

        if let (Some(&index_minus), Some(&index_current)) = (index_minus, index_current) {
            triplets.push(Triplet {
                row: index_minus,
                col: index_current,
                val: c64 { im: 0.0, re: -1.0 },
            });

            triplets.push(Triplet {
                row: index_current,
                col: index_minus,
                val: c64 { im: 0.0, re: -1.0 },
            });
        }

        if let Some(&index_current) = index_current {
            triplets.push(Triplet {
                row: index_current,
                col: index_current,
                val: c64 {
                    im: 0.0,
                    re: -self.value,
                },
            });
        }

        triplets
    }

    fn excitation_vector_dc_stamp(
        &self,
        _index_map: &HashMap<String, usize>,
        _solution_map: &HashMap<String, f64>,
    ) -> Vec<Triplet<usize, usize, f64>> {
        Vec::new()
    }

    fn excitation_vector_ac_stamp(
        &self,
        _index_map: &HashMap<String, usize>,
        _solution_map: &HashMap<String, f64>,
        _frequency: f64,
    ) -> Vec<Triplet<usize, usize, c64>> {
        Vec::new()
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
            "R{} {} {} {}",
            self.name, self.plus, self.minus, self.value,
        )
    }
}

impl FromStr for Resistor {
    type Err = crate::prelude::Error;

    fn from_str(s: &str) -> Result<Self> {
        let mut parts: Vec<&str> = s.split_whitespace().collect();

        if parts.contains(&"%") {
            let index = parts.iter().position(|&x| x == "%").unwrap();
            parts.truncate(index);
        }

        if parts.len() != 4 && parts.len() != 5 {
            return Err(Error::InvalidFormat(format!(
                "Invalid resistor format: '{s}'"
            )));
        }

        if parts[0].len() <= 1 {
            return Err(Error::InvalidFormat(format!(
                "Resistor name is too short: '{s}'"
            )));
        }

        let name = parts[0][1..]
            .parse::<u32>()
            .map_err(|_| Error::InvalidNodeName(format!("Invalid resistor name: '{s}'")))?;
        let plus = parts[1].to_string();
        let minus = parts[2].to_string();
        let value = parts[3]
            .parse::<f64>()
            .map_err(|_| Error::InvalidFloatValue(format!("Invalid resistor value: '{s}'")))?;

        Ok(Resistor {
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
    fn test_parse_resistor() {
        let resistor_str = "R1 1 0 1000";
        let resistor = resistor_str.parse::<Resistor>().unwrap();

        assert_eq!(resistor.name, 1);
        assert_eq!(resistor.plus, "1");
        assert_eq!(resistor.minus, "0");
        assert_eq!(resistor.value, 1000.0);
    }

    #[test]
    fn test_parse_resistor_with_comment() {
        let resistor_str = "R1 1 0 1000 % This is a comment";
        let resistor = resistor_str.parse::<Resistor>().unwrap();

        assert_eq!(resistor.name, 1);
        assert_eq!(resistor.plus, "1");
        assert_eq!(resistor.minus, "0");
        assert_eq!(resistor.value, 1000.0);
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
