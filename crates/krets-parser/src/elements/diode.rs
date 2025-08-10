use faer::sparse::Triplet;

use crate::{constants::THERMAL_VOLTAGE, elements::Stampable, prelude::*};
use std::{collections::HashMap, str::FromStr};

use super::Identifiable;

#[derive(Debug, Clone)]
/// Represents a diode in a circuit.
pub struct Diode {
    /// Name of the diode.
    pub name: u32,
    /// Value of the diode.
    pub options: DiodeOptions,
    /// Positive node of the diode.
    pub plus: String,
    /// Negative node of the diode.
    pub minus: String,
}

impl Identifiable for Diode {
    fn identifier(&self) -> String {
        format!("D{}", self.name)
    }
}

#[derive(Debug, Clone)]
/// Options for the diode, including saturation current, parasitic resistance, and emission coefficient.
pub struct DiodeOptions {
    /// The Saturation current.
    pub saturation_current: f64,
    /// The Parasitic resistance.
    pub parasitic_resistance: f64,
    /// The Emission coefficient.
    pub emission_coefficient: f64,
}

impl Default for DiodeOptions {
    fn default() -> Self {
        DiodeOptions {
            saturation_current: 1e-12, // Default value for saturation current
            parasitic_resistance: 0.0, // Default value for parasitic resistance
            emission_coefficient: 1.0, // Default value for emission coefficient
        }
    }
}

impl Stampable for Diode {
    fn add_conductance_matrix_dc_stamp(
        &self,
        index_map: &HashMap<String, usize>,
        solution_map: &HashMap<String, f64>,
    ) -> Vec<Triplet<usize, usize, f64>> {
        // Get node indices
        let index_plus = index_map.get(&format!("V({})", self.plus));
        let index_minus = index_map.get(&format!("V({})", self.minus));

        // The conductance of the diode based on the diode voltage and saturation current.
        let conductance = self.conductance(solution_map);

        let mut triplets = Vec::with_capacity(2);

        if let Some(&index_plus) = index_plus {
            triplets.push(Triplet::new(index_plus, index_plus, conductance));
        }
        if let Some(&index_minus) = index_minus {
            triplets.push(Triplet::new(index_minus, index_minus, conductance));
        }

        if let (Some(&index_plus), Some(&index_minus)) = (index_plus, index_minus) {
            triplets.push(Triplet::new(index_plus, index_minus, -conductance));
            triplets.push(Triplet::new(index_minus, index_plus, -conductance));
        }
        triplets
    }

    fn add_conductance_matrix_ac_stamp(
        &self,
        _index_map: &HashMap<String, usize>,
        _solution_map: &HashMap<String, f64>,
        _frequency: f64,
    ) -> Vec<Triplet<usize, usize, faer::c64>> {
        todo!()
    }

    fn add_excitation_vector_dc_stamp(
        &self,
        index_map: &HashMap<String, usize>,
        solution_map: &HashMap<String, f64>,
    ) -> Vec<Triplet<usize, usize, f64>> {
        // Get node indices
        let index_plus = index_map.get(&format!("V({})", self.plus));
        let index_minus = index_map.get(&format!("V({})", self.minus));

        let equivalent_current = self.equivalent_current(solution_map);

        let mut triplets = Vec::with_capacity(2);

        if let Some(&index_plus) = index_plus {
            triplets.push(Triplet::new(index_plus, 0, -equivalent_current));
        }
        if let Some(&index_minus) = index_minus {
            triplets.push(Triplet::new(index_minus, 0, equivalent_current));
        }
        triplets
    }

    fn add_excitation_vector_ac_stamp(
        &self,
        _index_map: &HashMap<String, usize>,
        _solution_map: &HashMap<String, f64>,
        _frequency: f64,
    ) -> Vec<Triplet<usize, usize, faer::c64>> {
        todo!()
    }
}

impl Diode {
    /// Returns the voltage at the plus node.
    pub fn v_plus(&self, solution_map: &HashMap<String, f64>) -> f64 {
        *solution_map
            .get(&format!("V({})", self.plus))
            .unwrap_or(&0.5)
    }

    /// Returns the voltage at the minus node.
    pub fn v_minus(&self, solution_map: &HashMap<String, f64>) -> f64 {
        *solution_map
            .get(&format!("V({})", self.minus))
            .unwrap_or(&0.0)
    }

    /// Returns the voltage across the diode (v_plus - v_minus).
    pub fn v_d(&self, solution_map: &HashMap<String, f64>) -> f64 {
        self.v_plus(solution_map) - self.v_minus(solution_map)
    }

    /// Returns the conductance of the diode based on the diode voltage and saturation current.
    fn conductance(&self, solution_map: &HashMap<String, f64>) -> f64 {
        let diode_voltage = self.v_d(solution_map);
        let saturation_current = self.options.saturation_current;

        (saturation_current / THERMAL_VOLTAGE) * f64::exp(diode_voltage / THERMAL_VOLTAGE)
    }

    fn current(&self, solution_map: &HashMap<String, f64>) -> f64 {
        // The current through the diode based on the diode voltage and saturation current.
        let diode_voltage = self.v_d(solution_map);
        let saturation_current = self.options.saturation_current;

        saturation_current * (f64::exp(diode_voltage / THERMAL_VOLTAGE) - 1.0)
    }

    fn equivalent_current(&self, solution_map: &HashMap<String, f64>) -> f64 {
        let diode_voltage = self.v_d(solution_map);

        self.current(solution_map) - self.conductance(solution_map) * diode_voltage
    }
}

impl FromStr for Diode {
    type Err = crate::prelude::Error;

    fn from_str(s: &str) -> Result<Self> {
        let mut parts: Vec<&str> = s.split_whitespace().collect();

        if parts.contains(&"%") {
            let index = parts.iter().position(|&x| x == "%").unwrap();
            parts.truncate(index);
        }

        if parts.len() != 3 && parts.len() != 4 {
            return Err(Error::InvalidFormat(format!("Invalid diode format: '{s}'")));
        }

        if parts[0].len() <= 1 {
            return Err(Error::InvalidFormat(format!(
                "Diode name is too short: '{s}'"
            )));
        }

        let name = parts[0][1..]
            .parse::<u32>()
            .map_err(|_| Error::InvalidNodeName(format!("Invalid diode name: '{s}'")))?;
        let plus = parts[1].to_string();
        let minus = parts[2].to_string();

        let diode_options = DiodeOptions::default();

        Ok(Diode {
            name,
            options: diode_options,
            plus,
            minus,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_diode() {
        let diode_str = "D1 1 0";
        let diode = diode_str.parse::<Diode>().unwrap();

        assert_eq!(diode.name, 1);
        assert_eq!(diode.plus, "1");
        assert_eq!(diode.minus, "0");
    }

    #[test]
    fn test_parse_diode_with_value() {
        let diode_str = "D1 1 0 0.7";
        let diode = diode_str.parse::<Diode>().unwrap();

        assert_eq!(diode.name, 1);
        assert_eq!(diode.plus, "1");
        assert_eq!(diode.minus, "0");
    }

    #[test]
    fn test_parse_diode_with_comment() {
        let diode_str = "D1 1 0 % This is a comment";
        let diode = diode_str.parse::<Diode>().unwrap();

        assert_eq!(diode.name, 1);
        assert_eq!(diode.plus, "1");
        assert_eq!(diode.minus, "0");
    }

    #[test]
    fn test_invalid_diode_format() {
        let diode_str = "D1 1";
        let result = diode_str.parse::<Diode>();
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_diode_name() {
        let diode_str = "D 1 0";
        let result = diode_str.parse::<Diode>();
        assert!(result.is_err());
    }
}
