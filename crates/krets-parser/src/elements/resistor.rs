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
    /// Value of the resistor in Ohms.
    pub value: f64,
    /// Positive node of the resistor.
    pub plus: String,
    /// Negative node of the resistor.
    pub minus: String,
}

impl Stampable for Resistor {
    fn add_conductance_matrix_dc_stamp(
        &self,
        index_map: &HashMap<String, usize>,
        _solution_map: &HashMap<String, f64>,
    ) -> Vec<Triplet<usize, usize, f64>> {
        let index_plus = index_map.get(&format!("V({})", self.plus));
        let index_minus = index_map.get(&format!("V({})", self.minus));
        let index_current = index_map.get(&format!("I({})", self.identifier()));

        let mut triplets = Vec::with_capacity(5);

        if let (Some(&index_plus), Some(&index_current)) = (index_plus, index_current) {
            triplets.push(Triplet::new(index_plus, index_current, 1.0));
            triplets.push(Triplet::new(index_current, index_plus, 1.0));
        }

        if let (Some(&index_minus), Some(&index_current)) = (index_minus, index_current) {
            triplets.push(Triplet::new(index_minus, index_current, -1.0));
            triplets.push(Triplet::new(index_current, index_minus, -1.0));
        }

        if let Some(&index_current) = index_current {
            triplets.push(Triplet::new(index_current, index_current, -self.value));
        }

        triplets
    }

    fn add_conductance_matrix_ac_stamp(
        &self,
        index_map: &HashMap<String, usize>,
        _solution_map: &HashMap<String, f64>,
        _frequency: f64,
    ) -> Vec<Triplet<usize, usize, c64>> {
        let index_plus = index_map.get(&format!("V({})", self.plus));
        let index_minus = index_map.get(&format!("V({})", self.minus));
        let index_current = index_map.get(&format!("I({})", self.identifier()));
        let one = c64::new(1.0, 0.0);
        let mut triplets = Vec::with_capacity(5);

        if let (Some(&index_plus), Some(&index_current)) = (index_plus, index_current) {
            triplets.push(Triplet::new(index_plus, index_current, one));
            triplets.push(Triplet::new(index_current, index_plus, one));
        }

        if let (Some(&index_minus), Some(&index_current)) = (index_minus, index_current) {
            triplets.push(Triplet::new(index_minus, index_current, -one));
            triplets.push(Triplet::new(index_current, index_minus, -one));
        }

        if let Some(&index_current) = index_current {
            triplets.push(Triplet::new(
                index_current,
                index_current,
                -c64::new(self.value, 0.0),
            ));
        }

        triplets
    }

    fn add_excitation_vector_dc_stamp(
        &self,
        _index_map: &HashMap<String, usize>,
        _solution_map: &HashMap<String, f64>,
    ) -> Vec<Triplet<usize, usize, f64>> {
        // A resistor is a passive component and does not add to the excitation vector.
        Vec::new()
    }

    fn add_excitation_vector_ac_stamp(
        &self,
        _index_map: &HashMap<String, usize>,
        _solution_map: &HashMap<String, f64>,
        _frequency: f64,
    ) -> Vec<Triplet<usize, usize, c64>> {
        // A resistor is a passive component and does not add to the excitation vector.
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
        // IMPROVEMENT: Handle comments robustly.
        let s_without_comment = s.split('%').next().unwrap_or("").trim();
        let parts: Vec<&str> = s_without_comment.split_whitespace().collect();

        // FIX: A resistor definition should have exactly 4 parts.
        if parts.len() != 4 {
            return Err(Error::InvalidFormat(format!(
                "Invalid resistor format: '{s}'"
            )));
        }

        // IMPROVEMENT: Add check for identifier prefix 'R'.
        if !parts[0].starts_with(['R', 'r']) {
            return Err(Error::InvalidFormat(format!(
                "Invalid resistor identifier: '{s}'"
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
        let value = parse_value(parts[3]).map_err(|e| {
            Error::InvalidFormat(format!("Invalid capacitor value in '{}': {}", s, e))
        })?;

        // IMPROVEMENT: Prevent division by zero in the stamping logic.
        if value <= 0.0 {
            return Err(Error::InvalidFloatValue(format!(
                "Resistor value must be positive: '{s}'"
            )));
        }

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
        assert_eq!(resistor.value, 1000.0);
    }

    #[test]
    fn test_parse_lowercase() {
        let s = "r5 2 3 1.5k"; // Note: SPICE suffixes like 'k' are not yet supported
        let _ = s.parse::<Resistor>();
        // This should fail on '1.5k' but pass the 'r' check. Let's test for a valid value.
        let s_valid = "r5 2 3 1500";
        assert!(s_valid.parse::<Resistor>().is_ok());
    }

    #[test]
    fn test_invalid_resistor_format() {
        let resistor_str = "R1 1 0";
        let result = resistor_str.parse::<Resistor>();
        assert!(result.is_err());
    }

    // NEW: Test for invalid prefix.
    #[test]
    fn test_invalid_prefix() {
        let s = "C1 1 0 1000";
        assert!(s.parse::<Resistor>().is_err());
    }

    #[test]
    fn test_invalid_resistor_name() {
        let resistor_str = "R 1 0 1000";
        let result = resistor_str.parse::<Resistor>();
        assert!(result.is_err());
    }

    // NEW: Test for zero-value resistance.
    #[test]
    fn test_error_on_zero_value() {
        let s = "R1 1 0 0";
        assert!(s.parse::<Resistor>().is_err());
    }

    #[test]
    fn test_invalid_resistor_value() {
        let resistor_str = "R1 1 0 abc";
        let result = resistor_str.parse::<Resistor>();
        assert!(result.is_err());
    }
}
