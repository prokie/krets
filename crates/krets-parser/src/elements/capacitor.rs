use super::{Identifiable, Stampable};
use crate::prelude::*;
use faer::{c64, sparse::Triplet};
use std::{f64::consts::PI, str::FromStr};

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
    // fn add_ac_stamp(&self, mna_matrix: &mut MnaMatrix, frequency: f64) {
    //     let index_plus = mna_matrix.index_map.get(&format!("V({})", self.plus));
    //     let index_minus = mna_matrix.index_map.get(&format!("V({})", self.minus));

    //     let impedance = c64 {
    //         re: 0.0,
    //         im: -1.0 / (2.0 * PI * frequency * self.value),
    //     };
    //     let conductance_matrix = &mut mna_matrix.complex_conductance_matrix;

    //     if !self.g2 {
    //         if let Some(&index_plus) = index_plus {
    //             conductance_matrix[(index_plus, index_plus)] += 1.0 / impedance;
    //         }
    //         if let Some(&index_minus) = index_minus {
    //             conductance_matrix[(index_minus, index_minus)] += 1.0 / impedance
    //         }

    //         if let (Some(&index_plus), Some(&index_minus)) = (index_plus, index_minus) {
    //             conductance_matrix[(index_plus, index_minus)] -= 1.0 / impedance;
    //             conductance_matrix[(index_minus, index_plus)] -= 1.0 / impedance;
    //         }
    //     } else {
    //         let index_current = mna_matrix
    //             .index_map
    //             .get(&format!("I({})", self.identifier()));

    //         if let (Some(&index_plus), Some(&index_current)) = (index_plus, index_current) {
    //             conductance_matrix[(index_current, index_plus)] = 1.0 / impedance;
    //         }

    //         if let (Some(&index_minus), Some(&index_current)) = (index_minus, index_current) {
    //             conductance_matrix[(index_current, index_minus)] = 1.0 / impedance;
    //         }

    //         if let Some(&index_current) = index_current {
    //             conductance_matrix[(index_current, index_current)] = c64 { re: 1.0, im: 0.0 };
    //         }
    //     }
    // }

    // fn add_dc_stamp(&self, _mna_matrix: &mut MnaMatrix) {
    //     todo!()
    // }

    fn conductance_matrix_dc_stamp(
        &self,
        _index_map: &std::collections::HashMap<String, usize>,
    ) -> Vec<faer::sparse::Triplet<usize, usize, f64>> {
        todo!()
    }

    fn conductance_matrix_ac_stamp(
        &self,
        index_map: &std::collections::HashMap<String, usize>,
        frequency: f64,
    ) -> Vec<faer::sparse::Triplet<usize, usize, c64>> {
        let index_plus = index_map.get(&format!("V({})", self.plus));
        let index_minus = index_map.get(&format!("V({})", self.minus));
        let index_current = index_map.get(&format!("I({})", self.identifier()));

        let mut triplets = Vec::with_capacity(4);

        let impedance = c64 {
            re: 0.0,
            im: -1.0 / (2.0 * PI * frequency * self.value),
        };

        if !self.g2 {
            if let Some(&index_plus) = index_plus {
                triplets.push(Triplet::new(index_plus, index_plus, 1.0 / impedance));
            }
            if let Some(&index_minus) = index_minus {
                triplets.push(Triplet::new(index_minus, index_minus, 1.0 / impedance));
            }

            if let (Some(&index_plus), Some(&index_minus)) = (index_plus, index_minus) {
                triplets.push(Triplet::new(index_plus, index_minus, -1.0 / impedance));
                triplets.push(Triplet::new(index_minus, index_plus, -1.0 / impedance));
            }
        } else {
            if let (Some(&index_plus), Some(&index_current)) = (index_plus, index_current) {
                triplets.push(Triplet::new(index_current, index_plus, 1.0 / impedance));
            }

            if let (Some(&index_minus), Some(&index_current)) = (index_minus, index_current) {
                triplets.push(Triplet::new(index_current, index_minus, 1.0 / impedance));
            }

            if let Some(&index_current) = index_current {
                triplets.push(Triplet::new(
                    index_current,
                    index_current,
                    c64 { re: 1.0, im: 0.0 },
                ));
            }
        }

        triplets
    }

    fn excitation_vector_dc_stamp(
        &self,
        _index_map: &std::collections::HashMap<String, usize>,
    ) -> Vec<faer::sparse::Triplet<usize, usize, f64>> {
        todo!()
    }

    fn excitation_vector_ac_stamp(
        &self,
        _index_map: &std::collections::HashMap<String, usize>,
        _frequency: f64,
    ) -> Vec<faer::sparse::Triplet<usize, usize, c64>> {
        vec![]
    }
}

impl FromStr for Capacitor {
    type Err = crate::prelude::Error;

    fn from_str(s: &str) -> Result<Self> {
        let mut parts: Vec<&str> = s.split_whitespace().collect();

        if parts.contains(&"%") {
            let index = parts.iter().position(|&x| x == "%").unwrap();
            parts.truncate(index);
        }

        if parts.len() != 4 && parts.len() != 5 {
            return Err(Error::InvalidFormat(format!(
                "Invalid capacitor format: '{}'",
                s
            )));
        }

        if parts[0].len() <= 1 {
            return Err(Error::InvalidFormat(format!(
                "Capacitor name is too short: '{}'",
                s
            )));
        }

        let name = parts[0][1..]
            .parse::<u32>()
            .map_err(|_| Error::InvalidNodeName(format!("Invalid capacitor name: '{}'", s)))?;
        let plus = parts[1].to_string();
        let minus = parts[2].to_string();
        let value = parts[3]
            .parse::<f64>()
            .map_err(|_| Error::InvalidFloatValue(format!("Invalid capacitor value: '{}'", s)))?;
        let g2 = parts.len() == 5 && parts[4] == "G2";

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
    fn test_invalid_capacitor_value() {
        let capacitor_str = "C1 1 0 abc";
        let result = capacitor_str.parse::<Capacitor>();
        assert!(result.is_err());
    }
}
