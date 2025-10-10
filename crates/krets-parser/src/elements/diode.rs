use faer::{c64, sparse::Triplet};

use crate::{constants::THERMAL_VOLTAGE, elements::Stampable, prelude::*};
use std::{collections::HashMap, str::FromStr};

use super::Identifiable;

#[derive(Debug, Clone)]
/// Represents a diode in a circuit.
pub struct Diode {
    /// Name of the diode.
    pub name: u32,
    /// The name of the diode model to use.
    pub model_name: String,
    /// Model parameters for the diode.
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
    /// The Saturation current (Is).
    pub saturation_current: f64,
    /// The Parasitic resistance (Rs).
    pub parasitic_resistance: f64,
    /// The Emission coefficient (N).
    pub emission_coefficient: f64,
}

impl Default for DiodeOptions {
    fn default() -> Self {
        DiodeOptions {
            saturation_current: 1e-12,
            parasitic_resistance: 0.0,
            emission_coefficient: 1.0,
        }
    }
}

impl Stampable for Diode {
    fn add_conductance_matrix_dc_stamp(
        &self,
        index_map: &HashMap<String, usize>,
        solution_map: &HashMap<String, f64>,
    ) -> Vec<Triplet<usize, usize, f64>> {
        let index_plus = index_map.get(&format!("V({})", self.plus));
        let index_minus = index_map.get(&format!("V({})", self.minus));

        // The linearized conductance of the diode for the current iteration.
        let conductance = self.conductance(solution_map);

        let mut triplets = Vec::with_capacity(4);

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
        index_map: &HashMap<String, usize>,
        solution_map: &HashMap<String, f64>,
        _frequency: f64,
    ) -> Vec<Triplet<usize, usize, faer::c64>> {
        // FIX: Implemented AC stamp. For a diode, the small-signal AC conductance
        // at a given DC bias point is the same as its linearized DC conductance.
        let conductance = self.conductance(solution_map);
        let conductance_complex = c64::new(conductance, 0.0);

        let index_plus = index_map.get(&format!("V({})", self.plus));
        let index_minus = index_map.get(&format!("V({})", self.minus));

        let mut triplets = Vec::with_capacity(4);

        if let Some(&index_plus) = index_plus {
            triplets.push(Triplet::new(index_plus, index_plus, conductance_complex));
        }
        if let Some(&index_minus) = index_minus {
            triplets.push(Triplet::new(index_minus, index_minus, conductance_complex));
        }
        if let (Some(&index_plus), Some(&index_minus)) = (index_plus, index_minus) {
            triplets.push(Triplet::new(index_plus, index_minus, -conductance_complex));
            triplets.push(Triplet::new(index_minus, index_plus, -conductance_complex));
        }
        triplets
    }

    fn add_excitation_vector_dc_stamp(
        &self,
        index_map: &HashMap<String, usize>,
        solution_map: &HashMap<String, f64>,
    ) -> Vec<Triplet<usize, usize, f64>> {
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
        // A diode is a passive component and does not
        // contribute to the excitation vector in small-signal AC analysis.
        vec![]
    }
}

impl Diode {
    // NOTE: This initial guess helps convergence but isn't a robust solution for all circuits.
    pub fn v_plus(&self, solution_map: &HashMap<String, f64>) -> f64 {
        *solution_map
            .get(&format!("V({})", self.plus))
            .unwrap_or(&0.5)
    }

    pub fn v_minus(&self, solution_map: &HashMap<String, f64>) -> f64 {
        *solution_map
            .get(&format!("V({})", self.minus))
            .unwrap_or(&0.0)
    }

    pub fn v_d(&self, solution_map: &HashMap<String, f64>) -> f64 {
        self.v_plus(solution_map) - self.v_minus(solution_map)
    }

    fn conductance(&self, solution_map: &HashMap<String, f64>) -> f64 {
        let diode_voltage = self.limit_diode_voltage(self.v_d(solution_map));
        let n = self.options.emission_coefficient;
        let is = self.options.saturation_current;

        (is / (n * THERMAL_VOLTAGE)) * f64::exp(diode_voltage / (n * THERMAL_VOLTAGE))
    }

    fn current(&self, solution_map: &HashMap<String, f64>) -> f64 {
        let diode_voltage = self.limit_diode_voltage(self.v_d(solution_map));
        let n = self.options.emission_coefficient;
        let is = self.options.saturation_current;

        is * (f64::exp(diode_voltage / (n * THERMAL_VOLTAGE)) - 1.0)
    }

    fn equivalent_current(&self, solution_map: &HashMap<String, f64>) -> f64 {
        let diode_voltage = self.v_d(solution_map);
        self.current(solution_map) - self.conductance(solution_map) * diode_voltage
    }

    // Voltage limiting function to prevent floating-point overflows
    // in the exponential function, which is a common issue in circuit simulators.
    fn limit_diode_voltage(&self, vd: f64) -> f64 {
        let n = self.options.emission_coefficient;
        let is = self.options.saturation_current;
        let v_critical = n * THERMAL_VOLTAGE * f64::ln(f64::MAX * n * THERMAL_VOLTAGE / is);
        vd.clamp(-v_critical, v_critical)
    }
}

impl FromStr for Diode {
    type Err = crate::prelude::Error;

    fn from_str(s: &str) -> Result<Self> {
        let s_without_comment = s.split('%').next().unwrap_or("").trim();
        let parts: Vec<&str> = s_without_comment.split_whitespace().collect();

        if parts.len() != 3 && parts.len() != 4 {
            return Err(Error::InvalidFormat(format!("Invalid diode format: '{s}'")));
        }

        // IMPROVEMENT: Add check for identifier prefix 'D'.
        if !parts[0].starts_with(['D', 'd']) {
            return Err(Error::InvalidFormat(format!(
                "Invalid diode identifier: '{s}'"
            )));
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

        // IMPROVEMENT: Parse the model name. In a full simulator, you would use this
        // name to look up the DiodeOptions from a .MODEL statement.
        let model_name = if parts.len() == 4 {
            parts[3].to_string()
        } else {
            // SPICE often has a default model if none is specified.
            "default".to_string()
        };

        let diode_options = DiodeOptions::default();

        Ok(Diode {
            name,
            model_name,
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
        assert_eq!(diode.model_name, "default");
    }

    #[test]
    fn test_parse_diode_with_model() {
        let diode_str = "D1 1 0 1N4148";
        let diode = diode_str.parse::<Diode>().unwrap();

        assert_eq!(diode.name, 1);
        assert_eq!(diode.plus, "1");
        assert_eq!(diode.minus, "0");
        assert_eq!(diode.model_name, "1N4148");
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

    #[test]
    fn test_invalid_prefix() {
        let s = "R1 1 0 1N4148";
        let result = s.parse::<Diode>();
        assert!(result.is_err());
    }
}
