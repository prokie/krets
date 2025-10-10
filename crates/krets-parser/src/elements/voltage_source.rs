use faer::c64;
use faer::sparse::Triplet;

use crate::prelude::*;
use std::str::FromStr;
use std::{collections::HashMap, fmt};

use super::{Identifiable, Stampable};

#[derive(Debug, Clone)]
/// Represents a voltage source in a circuit.
pub struct VoltageSource {
    /// Name of the voltage source.
    pub name: u32,
    /// Value of the DC voltage source.
    pub value: f64,
    /// Positive node of the voltage source.
    pub plus: String,
    /// Negative node of the voltage source.
    pub minus: String,
    /// For AC analysis, the amplitude of the AC voltage source.
    pub ac_amplitude: Option<f64>,
}

impl Identifiable for VoltageSource {
    fn identifier(&self) -> String {
        format!("V{}", self.name)
    }
}

impl Stampable for VoltageSource {
    fn add_conductance_matrix_dc_stamp(
        &self,
        index_map: &HashMap<String, usize>,
        _solution_map: &HashMap<String, f64>,
    ) -> Vec<Triplet<usize, usize, f64>> {
        let index_plus = index_map.get(&format!("V({})", self.plus));
        let index_minus = index_map.get(&format!("V({})", self.minus));
        let index_current = index_map.get(&format!("I({})", self.identifier()));

        let mut triplets = Vec::with_capacity(4);

        if let (Some(&ip), Some(&ic)) = (index_plus, index_current) {
            triplets.push(Triplet::new(ip, ic, 1.0));
            triplets.push(Triplet::new(ic, ip, 1.0));
        }

        if let (Some(&im), Some(&ic)) = (index_minus, index_current) {
            triplets.push(Triplet::new(im, ic, -1.0));
            triplets.push(Triplet::new(ic, im, -1.0));
        }

        triplets
    }

    fn add_conductance_matrix_ac_stamp(
        &self,
        index_map: &std::collections::HashMap<String, usize>,
        _solution_map: &HashMap<String, f64>,
        _frequency: f64,
    ) -> Vec<faer::sparse::Triplet<usize, usize, c64>> {
        let index_plus = index_map.get(&format!("V({})", self.plus));
        let index_minus = index_map.get(&format!("V({})", self.minus));
        let index_current = index_map.get(&format!("I({})", self.identifier()));
        let one = c64::new(1.0, 0.0);
        let mut triplets = Vec::with_capacity(4);

        if let (Some(&ip), Some(&ic)) = (index_plus, index_current) {
            triplets.push(Triplet::new(ip, ic, one));
            triplets.push(Triplet::new(ic, ip, one));
        }

        if let (Some(&im), Some(&ic)) = (index_minus, index_current) {
            triplets.push(Triplet::new(im, ic, -one));
            triplets.push(Triplet::new(ic, im, -one));
        }

        triplets
    }

    fn add_excitation_vector_dc_stamp(
        &self,
        index_map: &HashMap<String, usize>,
        _solution_map: &HashMap<String, f64>,
    ) -> Vec<Triplet<usize, usize, f64>> {
        let mut triplets = Vec::with_capacity(1);
        if let Some(&ic) = index_map.get(&format!("I({})", self.identifier())) {
            triplets.push(Triplet::new(ic, 0, self.value));
        }
        triplets
    }

    fn add_excitation_vector_ac_stamp(
        &self,
        index_map: &HashMap<String, usize>,
        _solution_map: &HashMap<String, f64>,
        _frequency: f64,
    ) -> Vec<Triplet<usize, usize, c64>> {
        let mut triplets = Vec::with_capacity(1);

        if let (Some(&ic), Some(ac_amplitude)) = (
            index_map.get(&format!("I({})", self.identifier())),
            self.ac_amplitude,
        ) {
            triplets.push(Triplet::new(ic, 0, c64::new(ac_amplitude, 0.0)));
        }
        triplets
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
        let s_without_comment = s.split('%').next().unwrap_or("").trim();
        let parts: Vec<&str> = s_without_comment.split_whitespace().collect();

        if parts.len() != 4 && parts.len() != 6 {
            return Err(Error::InvalidFormat(format!(
                "Invalid voltage source format: Expected 4 or 6 parts, found {}, in '{s}'",
                parts.len()
            )));
        }

        if !parts[0].starts_with(['V', 'v']) {
            return Err(Error::InvalidFormat(format!(
                "Invalid voltage source identifier: '{s}'"
            )));
        }

        if parts[0].len() <= 1 {
            return Err(Error::InvalidFormat(format!(
                "Voltage source name is too short: '{s}'"
            )));
        }

        let name = parts[0][1..]
            .parse::<u32>()
            .map_err(|_| Error::InvalidNodeName(format!("Invalid voltage source name: '{s}'")))?;
        let plus = parts[1].to_string();
        let minus = parts[2].to_string();
        let value = parts[3].parse::<f64>().map_err(|_| {
            Error::InvalidFloatValue(format!("Invalid voltage source value: '{s}'"))
        })?;

        let ac_amplitude = if parts.len() == 6 {
            if parts[4].eq_ignore_ascii_case("AC") {
                Some(parts[5].parse::<f64>().map_err(|_| {
                    Error::InvalidFloatValue(format!("Invalid AC amplitude value: '{s}'"))
                })?)
            } else {
                return Err(Error::InvalidFormat(format!(
                    "Invalid AC specification: expected 'AC', found '{}' in '{s}'",
                    parts[4]
                )));
            }
        } else {
            None
        };

        Ok(VoltageSource {
            name,
            value,
            plus,
            minus,
            ac_amplitude,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- Parser Tests ---
    #[test]
    fn test_parse_voltage_source() {
        let s = "V1 1 0 5";
        let vs = s.parse::<VoltageSource>().unwrap();
        assert_eq!(vs.name, 1);
        assert_eq!(vs.plus, "1");
        assert_eq!(vs.minus, "0");
        assert_eq!(vs.value, 5.0);
        assert_eq!(vs.ac_amplitude, None);
    }

    #[test]
    fn test_parse_with_ac() {
        let s = "V2 3 4 0 AC 1.5";
        let vs = s.parse::<VoltageSource>().unwrap();
        assert_eq!(vs.name, 2);
        assert_eq!(vs.value, 0.0);
        assert_eq!(vs.ac_amplitude, Some(1.5));
    }

    #[test]
    fn test_parse_case_insensitive() {
        let s = "v3 5 6 12 ac 10";
        let vs = s.parse::<VoltageSource>().unwrap();
        assert_eq!(vs.name, 3);
        assert_eq!(vs.ac_amplitude, Some(10.0));
    }

    #[test]
    fn test_parse_with_comment() {
        let s = "V1 1 0 5 % DC value";
        let vs = s.parse::<VoltageSource>().unwrap();
        assert_eq!(vs.value, 5.0);
    }

    #[test]
    fn test_invalid_format_too_few_parts() {
        assert!("V1 1 0".parse::<VoltageSource>().is_err());
    }

    #[test]
    fn test_invalid_format_too_many_parts() {
        assert!("V1 1 0 5 6".parse::<VoltageSource>().is_err());
    }

    #[test]
    fn test_invalid_ac_format() {
        assert!("V1 1 0 5 AC".parse::<VoltageSource>().is_err()); // Missing value
        assert!("V1 1 0 5 DC 1".parse::<VoltageSource>().is_err()); // Wrong keyword
    }
}
