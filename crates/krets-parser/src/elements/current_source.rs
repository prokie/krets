use faer::sparse::Triplet;

use crate::prelude::*;
use std::{collections::HashMap, str::FromStr};

use super::{Identifiable, Stampable};

#[derive(Debug, Clone)]
/// Represents a current source in a circuit.
pub struct CurrentSource {
    /// The name of the current source.
    pub name: u32,
    /// The value of the current source.
    pub value: f64,
    /// The positive node of the current source.
    pub plus: String,
    /// The negative node of the current source.
    pub minus: String,
    // If the current source is G2.
    pub g2: bool,
}

impl Identifiable for CurrentSource {
    fn identifier(&self) -> String {
        format!("I{}", self.name)
    }
}

impl Stampable for CurrentSource {
    fn conductance_matrix_dc_stamp(
        &self,
        index_map: &HashMap<String, usize>,
        _solution_map: &HashMap<String, f64>,
    ) -> Vec<Triplet<usize, usize, f64>> {
        let index_plus = index_map.get(&format!("V({})", self.plus));
        let index_minus = index_map.get(&format!("V({})", self.minus));
        let index_current = index_map.get(&format!("I({})", self.identifier()));

        let mut triplets = Vec::with_capacity(2);

        if let (Some(&index_plus), Some(&index_minus), Some(&index_current)) =
            (index_plus, index_minus, index_current)
        {
            triplets.push(Triplet::new(index_plus, index_current, 1.0));
            triplets.push(Triplet::new(index_minus, index_current, -1.0));
        }
        triplets
    }

    fn conductance_matrix_ac_stamp(
        &self,
        _index_map: &HashMap<String, usize>,
        _solution_map: &HashMap<String, f64>,
        _frequency: f64,
    ) -> Vec<Triplet<usize, usize, faer::c64>> {
        todo!()
    }

    fn excitation_vector_dc_stamp(
        &self,
        index_map: &HashMap<String, usize>,
        _solution_map: &HashMap<String, f64>,
    ) -> Vec<Triplet<usize, usize, f64>> {
        let mut triplets = Vec::with_capacity(3);

        if let Some(&i) = index_map.get(&format!("V({})", self.plus)) {
            triplets.push(Triplet::new(i, 0, -self.value));
        }

        if let Some(&i) = index_map.get(&format!("V({})", self.minus)) {
            triplets.push(Triplet::new(i, 0, self.value));
        }

        if let Some(&i) = index_map.get(&format!("I({})", self.identifier())) {
            triplets.push(Triplet::new(i, 0, self.value));
        }

        triplets
    }

    fn excitation_vector_ac_stamp(
        &self,
        _index_map: &HashMap<String, usize>,
        _solution_map: &HashMap<String, f64>,
        _frequency: f64,
    ) -> Vec<Triplet<usize, usize, faer::c64>> {
        todo!()
    }
    // fn add_dc_stamp(&self, mna_matrix: &mut MnaMatrix) {
    //     let index_plus = mna_matrix.index_map.get(&format!("V({})", self.plus));
    //     let index_minus = mna_matrix.index_map.get(&format!("V({})", self.minus));
    //     let index_current = mna_matrix
    //         .index_map
    //         .get(&format!("I({})", self.identifier()));

    //     if let Some(&index_plus) = index_plus {
    //         mna_matrix.excitation_vector[(index_plus, 0)] = -self.value;
    //     }

    //     if let Some(&index_minus) = index_minus {
    //         mna_matrix.excitation_vector[(index_minus, 0)] = self.value;
    //     }

    //     if let (Some(&index_plus), Some(&index_minus), Some(&index_current)) =
    //         (index_plus, index_minus, index_current)
    //     {
    //         mna_matrix.excitation_vector[(index_current, 0)] = self.value;
    //         mna_matrix.conductance_matrix[(index_plus, index_current)] = 1.0;
    //         mna_matrix.conductance_matrix[(index_minus, index_current)] = -1.0;
    //     }
    // }

    // fn add_ac_stamp(&self, mna_matrix: &mut MnaMatrix, _frequency: f64) {
    //     self.add_dc_stamp(mna_matrix);
    // }
}

impl FromStr for CurrentSource {
    type Err = crate::prelude::Error;

    fn from_str(s: &str) -> Result<Self> {
        let parts: Vec<&str> = s.split_whitespace().collect();

        if parts.len() != 4 && parts.len() != 5 {
            return Err(Error::InvalidFormat(
                "Invalid current source format".to_string(),
            ));
        }

        if parts[0].len() <= 1 {
            return Err(Error::InvalidFormat(
                "Current source name is too short".to_string(),
            ));
        }

        let name = parts[0][1..]
            .parse::<u32>()
            .map_err(|_| Error::InvalidNodeName("Invalid current source name".to_string()))?;
        let plus = parts[1].to_string();
        let minus = parts[2].to_string();
        let value = parts[3]
            .parse::<f64>()
            .map_err(|_| Error::InvalidFloatValue("Invalid current source value".to_string()))?;

        let g2 = parts.len() == 5 && parts[4] == "G2";

        Ok(CurrentSource {
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
    fn test_parse_current_source() {
        let current_source_str = "I1 1 0 0.001";
        let current_source = current_source_str.parse::<CurrentSource>().unwrap();

        assert_eq!(current_source.name, 1);
        assert_eq!(current_source.plus, "1");
        assert_eq!(current_source.minus, "0");
        assert_eq!(current_source.value, 0.001);
        assert!(!current_source.g2);
    }

    #[test]
    fn test_parse_current_source_with_group() {
        let current_source_str = "I1 1 0 0.001 G2";
        let current_source = current_source_str.parse::<CurrentSource>().unwrap();

        assert_eq!(current_source.name, 1);
        assert_eq!(current_source.plus, "1");
        assert_eq!(current_source.minus, "0");
        assert_eq!(current_source.value, 0.001);
        assert!(current_source.g2);
    }

    #[test]
    fn test_invalid_current_source_format() {
        let current_source_str = "I1 1 0";
        let result = current_source_str.parse::<CurrentSource>();
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_current_source_name() {
        let current_source_str = "I 1 0 0.001";
        let result = current_source_str.parse::<CurrentSource>();
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_current_source_value() {
        let current_source_str = "I1 1 0 abc";
        let result = current_source_str.parse::<CurrentSource>();
        assert!(result.is_err());
    }
}
