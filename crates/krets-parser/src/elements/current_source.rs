use faer::{c64, sparse::Triplet};

use crate::prelude::*;
use std::{collections::HashMap, str::FromStr};

use super::{Identifiable, Stampable};

#[derive(Debug, Clone)]
/// Represents a current source in a circuit.
pub struct CurrentSource {
    /// The name of the current source.
    pub name: u32,
    /// The value of the current source in Amperes.
    pub value: f64,
    /// The positive node of the current source.
    pub plus: String,
    /// The negative node of the current source.
    pub minus: String,
}

impl Identifiable for CurrentSource {
    fn identifier(&self) -> String {
        format!("I{}", self.name)
    }
}

impl Stampable for CurrentSource {
    fn add_conductance_matrix_dc_stamp(
        &self,
        index_map: &HashMap<String, usize>,
        _solution_map: &HashMap<String, f64>,
    ) -> Vec<Triplet<usize, usize, f64>> {
        let index_plus = index_map.get(&format!("V({})", self.plus));
        let index_minus = index_map.get(&format!("V({})", self.minus));
        let index_current = index_map.get(&format!("I({})", self.identifier()));

        let mut triplets = Vec::with_capacity(3);
        if let Some(&index_current) = index_current {
            triplets.push(Triplet::new(index_current, index_current, 1.0));
        }
        if let (Some(&index_plus), Some(&index_current)) = (index_plus, index_current) {
            triplets.push(Triplet::new(index_plus, index_current, 1.0));
        }

        if let (Some(&index_minus), Some(&index_current)) = (index_minus, index_current) {
            triplets.push(Triplet::new(index_minus, index_current, -1.0));
        }
        triplets
    }

    fn add_conductance_matrix_ac_stamp(
        &self,
        index_map: &HashMap<String, usize>,
        _solution_map: &HashMap<String, f64>,
        _frequency: f64,
    ) -> Vec<Triplet<usize, usize, faer::c64>> {
        // FIX: Implemented the AC stamp, which is identical to the DC stamp for a
        // frequency-independent current source.
        let index_plus = index_map.get(&format!("V({})", self.plus));
        let index_minus = index_map.get(&format!("V({})", self.minus));
        let index_current = index_map.get(&format!("I({})", self.identifier()));

        let mut triplets = Vec::with_capacity(3);

        if let (Some(&index_plus), Some(&index_current)) = (index_plus, index_current) {
            triplets.push(Triplet::new(index_plus, index_current, c64::new(1.0, 0.0)));
        }

        if let (Some(&index_minus), Some(&index_current)) = (index_minus, index_current) {
            triplets.push(Triplet::new(
                index_minus,
                index_current,
                c64::new(-1.0, 0.0),
            ));
        }

        triplets
    }

    fn add_excitation_vector_dc_stamp(
        &self,
        index_map: &HashMap<String, usize>,
        _solution_map: &HashMap<String, f64>,
    ) -> Vec<Triplet<usize, usize, f64>> {
        // The branch equation for the source is I_source = value, which is represented
        // in the form A*x = z as 1*I_source = value. The '1' is handled by the solver's
        // identity matrix initialization, so we only need to provide the 'value' for the z vector.
        match index_map.get(&format!("I({})", self.identifier())) {
            Some(i) => vec![Triplet::new(*i, 0, self.value)],
            None => Vec::new(),
        }
    }

    fn add_excitation_vector_ac_stamp(
        &self,
        index_map: &HashMap<String, usize>,
        _solution_map: &HashMap<String, f64>,
        _frequency: f64,
    ) -> Vec<Triplet<usize, usize, faer::c64>> {
        // FIX: Implemented the AC excitation stamp. For a simple source, this is a
        // real value, but it's represented as a complex number.
        match index_map.get(&format!("I({})", self.identifier())) {
            Some(i) => vec![Triplet::new(*i, 0, c64::new(self.value, 0.0))],
            None => Vec::new(),
        }
    }
}

impl FromStr for CurrentSource {
    type Err = crate::prelude::Error;

    fn from_str(s: &str) -> Result<Self> {
        // IMPROVEMENT: Handle comments robustly.
        let s_without_comment = s.split('%').next().unwrap_or("").trim();
        let parts: Vec<&str> = s_without_comment.split_whitespace().collect();

        // FIX: A current source definition should have exactly 4 parts.
        if parts.len() != 4 {
            return Err(Error::InvalidFormat(format!(
                "Invalid current source format: '{s}'"
            )));
        }

        // IMPROVEMENT: Add check for identifier prefix 'I'.
        if !parts[0].starts_with(['I', 'i']) {
            return Err(Error::InvalidFormat(format!(
                "Invalid current source identifier: '{s}'"
            )));
        }

        if parts[0].len() <= 1 {
            return Err(Error::InvalidFormat(format!(
                "Current source name is too short: '{s}'"
            )));
        }

        let name = parts[0][1..]
            .parse::<u32>()
            .map_err(|_| Error::InvalidNodeName(format!("Invalid current source name: '{s}'")))?;
        let plus = parts[1].to_string();
        let minus = parts[2].to_string();
        let value = parts[3].parse::<f64>().map_err(|_| {
            Error::InvalidFloatValue(format!("Invalid current source value: '{s}'"))
        })?;

        Ok(CurrentSource {
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
    fn test_parse_current_source() {
        let current_source_str = "I1 1 0 0.001";
        let current_source = current_source_str.parse::<CurrentSource>().unwrap();

        assert_eq!(current_source.name, 1);
        assert_eq!(current_source.plus, "1");
        assert_eq!(current_source.minus, "0");
        assert_eq!(current_source.value, 0.001);
    }

    #[test]
    fn test_parse_with_comment() {
        let s = "I2 5 3 1.5 % Amperes";
        let source = s.parse::<CurrentSource>().unwrap();
        assert_eq!(source.name, 2);
        assert_eq!(source.value, 1.5);
    }

    #[test]
    fn test_parse_lowercase_identifier() {
        let s = "i5 vdd gnd 10";
        let source = s.parse::<CurrentSource>().unwrap();
        assert_eq!(source.name, 5);
        assert_eq!(source.plus, "vdd");
    }

    #[test]
    fn test_invalid_current_source_format() {
        let current_source_str = "I1 1 0";
        let result = current_source_str.parse::<CurrentSource>();
        assert!(result.is_err());
    }

    #[test]
    fn test_too_many_parts() {
        let s = "I1 1 0 1.0 G2";
        let result = s.parse::<CurrentSource>();
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_prefix() {
        let s = "V1 1 0 1.0";
        let result = s.parse::<CurrentSource>();
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
