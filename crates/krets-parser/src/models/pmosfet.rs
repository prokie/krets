use crate::{models::ModelTrait, prelude::*};

#[derive(Debug, PartialEq, Clone)]
pub struct PMosfetModel {
    // Name
    pub name: String,
    // The width of the MOSFET channel in meters.
    // In netlist is specified with parameter "W"
    pub width: f64,
    // The length of the MOSFET channel in meters.
    // In netlist is specified with parameter "L"
    pub length: f64,
    // The voltage threshold of the MOSFET in volts.
    // In netlist is specified with parameter "VTO"
    pub voltage_threshold: f64,
    // The device transconductance of the MOSFET in A/V^2.
    // In netlist is specified with parameter "K"
    pub process_transconductance: f64,
    // Channel length modulation parameter in 1/V.
    // In netlist is specified with parameter "LAMBDA"
    pub channel_length_modulation: f64,
}

impl Default for PMosfetModel {
    fn default() -> Self {
        PMosfetModel {
            name: "default".to_string(),
            width: 1e-6,                     // Default width of 1 micrometer
            length: 1e-6,                    // Default length of 1 micrometer
            voltage_threshold: 0.0,          // Default threshold voltage of 0.0 V
            process_transconductance: 2e-5,  // Default process transconductance
            channel_length_modulation: 0.02, // Default channel length modulation
        }
    }
}

impl PMosfetModel {
    /// Calculates the beta parameter of the MOSFET.
    /// Beta is defined as: β = K * (W / L), where K is the process transconductance,
    /// W is the width, and L is the length of the MOSFET channel.
    pub fn beta(&self) -> f64 {
        self.process_transconductance * (self.width / self.length)
    }
}

impl ModelTrait for PMosfetModel {
    fn apply_model_parameters(&mut self, parameters: &HashMap<String, f64>) {
        for (key, value) in parameters {
            match key.to_lowercase().as_str() {
                "w" => self.width = *value,
                "l" => self.length = *value,
                "vto" => self.voltage_threshold = *value,
                "kp" => self.process_transconductance = *value,
                "lambda" => self.channel_length_modulation = *value,
                _ => {
                    // Unknown parameter; could log a warning or ignore
                }
            }
        }
    }
}
