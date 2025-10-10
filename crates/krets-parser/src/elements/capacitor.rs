use super::{Identifiable, Stampable};
use crate::prelude::*;
use faer::{c64, sparse::Triplet};
use std::{collections::HashMap, f64::consts::PI, str::FromStr};

#[derive(Debug, Clone)]
/// Represents a capacitor in a circuit.
pub struct Capacitor {
    /// Name of the capacitor.
    pub name: u32,
    /// Value of the capacitor.
    pub value: f64,
    /// Positive node of the capacitor.
    pub plus: String,
    /// Negative node of the capacitor.
    pub minus: String,
    /// If the capacitor is G2.
    pub g2: bool,
}

impl Identifiable for Capacitor {
    fn identifier(&self) -> String {
        format!("C{}", self.name)
    }
}

impl Stampable for Capacitor {
    fn add_conductance_matrix_dc_stamp(
        &self,
        _index_map: &HashMap<String, usize>,
        _solution_map: &HashMap<String, f64>,
    ) -> Vec<faer::sparse::Triplet<usize, usize, f64>> {
        // A capacitor is an open circuit in DC analysis, so it contributes nothing to the DC conductance matrix.
        vec![]
    }

    fn add_conductance_matrix_ac_stamp(
        &self,
        index_map: &HashMap<String, usize>,
        _solution_map: &HashMap<String, f64>,
        frequency: f64,
    ) -> Vec<faer::sparse::Triplet<usize, usize, c64>> {
        let index_plus = index_map.get(&format!("V({})", self.plus));
        let index_minus = index_map.get(&format!("V({})", self.minus));

        let admittance = c64 {
            re: 0.0,
            im: 2.0 * PI * frequency * self.value,
        };

        let mut triplets = Vec::with_capacity(4);

        if !self.g2 {
            if let Some(&index_plus) = index_plus {
                triplets.push(Triplet::new(index_plus, index_plus, admittance));
            }
            if let Some(&index_minus) = index_minus {
                triplets.push(Triplet::new(index_minus, index_minus, admittance));
            }
            if let (Some(&index_plus), Some(&index_minus)) = (index_plus, index_minus) {
                triplets.push(Triplet::new(index_plus, index_minus, -admittance));
                triplets.push(Triplet::new(index_minus, index_plus, -admittance));
            }
        } else {
            let index_current = index_map.get(&format!("I({})", self.identifier()));

            if let (Some(&index_plus), Some(&index_current)) = (index_plus, index_current) {
                // -Y contribution for V_plus
                triplets.push(Triplet::new(index_current, index_plus, -admittance));
            }

            if let (Some(&index_minus), Some(&index_current)) = (index_minus, index_current) {
                // +Y contribution for V_minus
                triplets.push(Triplet::new(index_current, index_minus, admittance));
            }

            if let Some(&index_current) = index_current {
                // +1 contribution for I_c
                triplets.push(Triplet::new(
                    index_current,
                    index_current,
                    c64 { re: 1.0, im: 0.0 },
                ));
            }
        }

        triplets
    }

    fn add_excitation_vector_dc_stamp(
        &self,
        _index_map: &HashMap<String, usize>,
        _solution_map: &HashMap<String, f64>,
    ) -> Vec<faer::sparse::Triplet<usize, usize, f64>> {
        // Capacitors are passive and don't contribute to the DC excitation vector.
        vec![]
    }

    fn add_excitation_vector_ac_stamp(
        &self,
        _index_map: &HashMap<String, usize>,
        _solution_map: &HashMap<String, f64>,
        _frequency: f64,
    ) -> Vec<faer::sparse::Triplet<usize, usize, c64>> {
        // Capacitors are passive and don't contribute to the AC excitation vector.
        vec![]
    }
}

impl FromStr for Capacitor {
    type Err = crate::prelude::Error;

    fn from_str(s: &str) -> Result<Self> {
        // IMPROVEMENT: Handle comments more robustly by splitting the string
        // at the comment character '%' before processing.
        let s_without_comment = s.split('%').next().unwrap_or("").trim();
        let parts: Vec<&str> = s_without_comment.split_whitespace().collect();

        if parts.len() < 4 || parts.len() > 5 {
            return Err(Error::InvalidFormat(format!(
                "Invalid capacitor format: '{s}'"
            )));
        }

        // FIX: Add check for identifier prefix 'C'
        if !parts[0].starts_with(['C', 'c']) {
            return Err(Error::InvalidFormat(format!(
                "Invalid capacitor identifier: '{s}'"
            )));
        }

        if parts[0].len() <= 1 {
            return Err(Error::InvalidFormat(format!(
                "Capacitor name is too short: '{s}'"
            )));
        }

        let name = parts[0][1..]
            .parse::<u32>()
            .map_err(|_| Error::InvalidNodeName(format!("Invalid capacitor name: '{s}'")))?;
        let plus = parts[1].to_string();
        let minus = parts[2].to_string();
        let value = parts[3]
            .parse::<f64>()
            .map_err(|_| Error::InvalidFloatValue(format!("Invalid capacitor value: '{s}'")))?;

        let g2 = if parts.len() == 5 {
            if parts[4].eq_ignore_ascii_case("G2") {
                true
            } else {
                return Err(Error::InvalidFormat(format!(
                    "Invalid 5th argument for capacitor: '{s}'. Expected 'G2'."
                )));
            }
        } else {
            false
        };

        Ok(Capacitor {
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
    fn test_parse_capacitor() {
        let capacitor_str = "C1 1 0 0.000001";
        let capacitor = capacitor_str.parse::<Capacitor>().unwrap();

        assert_eq!(capacitor.name, 1);
        assert_eq!(capacitor.plus, "1");
        assert_eq!(capacitor.minus, "0");
        assert_eq!(capacitor.value, 0.000001);
        assert!(!capacitor.g2);
    }

    #[test]
    fn test_parse_capacitor_with_group() {
        let capacitor_str = "C1 1 0 0.000001 G2";
        let capacitor = capacitor_str.parse::<Capacitor>().unwrap();

        assert_eq!(capacitor.name, 1);
        assert_eq!(capacitor.plus, "1");
        assert_eq!(capacitor.minus, "0");
        assert_eq!(capacitor.value, 0.000001);
        assert!(capacitor.g2);
    }

    #[test]
    fn test_parse_capacitor_with_comment() {
        let capacitor_str = "C1 1 0 0.000001 % This is a comment";
        let capacitor = capacitor_str.parse::<Capacitor>().unwrap();

        assert_eq!(capacitor.name, 1);
        assert_eq!(capacitor.plus, "1");
        assert_eq!(capacitor.minus, "0");
        assert_eq!(capacitor.value, 0.000001);
        assert!(!capacitor.g2);
    }

    #[test]
    fn test_parse_capacitor_with_comment_no_space() {
        let capacitor_str = "C1 1 0 1e-6%This is a comment";
        let capacitor = capacitor_str.parse::<Capacitor>().unwrap();

        assert_eq!(capacitor.name, 1);
        assert_eq!(capacitor.value, 1e-6);
        assert!(!capacitor.g2);
    }

    #[test]
    fn test_parse_capacitor_with_g2_and_comment() {
        let capacitor_str = "c2 3 4 10e-9 G2 % comment";
        let capacitor = capacitor_str.parse::<Capacitor>().unwrap();

        assert_eq!(capacitor.name, 2);
        assert_eq!(capacitor.value, 10e-9);
        assert!(capacitor.g2);
    }

    #[test]
    fn test_parse_lowercase() {
        let capacitor_str = "c1 1 0 1e-6 g2";
        let capacitor = capacitor_str.parse::<Capacitor>().unwrap();

        assert_eq!(capacitor.name, 1);
        assert!(capacitor.g2);
    }

    #[test]
    fn test_invalid_capacitor_format() {
        let capacitor_str = "C1 1 0";
        let result = capacitor_str.parse::<Capacitor>();
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_capacitor_name() {
        let capacitor_str = "C 1 0 0.000001";
        let result = capacitor_str.parse::<Capacitor>();
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_identifier_prefix() {
        let capacitor_str = "R1 1 0 100";
        let result = capacitor_str.parse::<Capacitor>();
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_fifth_argument() {
        let capacitor_str = "C1 1 0 1e-6 G3";
        let result = capacitor_str.parse::<Capacitor>();
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_capacitor_value() {
        let capacitor_str = "C1 1 0 abc";
        let result = capacitor_str.parse::<Capacitor>();
        assert!(result.is_err());
    }
}
