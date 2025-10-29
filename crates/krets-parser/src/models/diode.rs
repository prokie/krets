use crate::{models::ModelTrait, prelude::*};

#[derive(Debug, PartialEq, Clone)]
pub struct DiodeModel {
    pub name: String,
    /// The Saturation current (Is).
    pub saturation_current: f64,
    /// The Parasitic resistance (Rs).
    pub parasitic_resistance: f64,
    /// The Emission coefficient (N).
    pub emission_coefficient: f64,
}

impl Default for DiodeModel {
    fn default() -> Self {
        DiodeModel {
            name: String::new(),
            saturation_current: 1e-12,
            parasitic_resistance: 0.0,
            emission_coefficient: 1.0,
        }
    }
}

impl ModelTrait for DiodeModel {
    fn apply_model_parameters(&mut self, parameters: &HashMap<String, f64>) {
        for (key, value) in parameters {
            match key.to_lowercase().as_str() {
                "is" => self.saturation_current = *value,
                "rs" => self.parasitic_resistance = *value,
                "n" => self.emission_coefficient = *value,
                _ => {
                    // Unknown parameter; could log a warning or ignore
                }
            }
        }
    }
}
