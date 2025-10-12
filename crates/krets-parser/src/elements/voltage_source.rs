use crate::prelude::*;
use faer::c64;
use faer::sparse::Triplet;
use regex::Regex;
use std::str::FromStr;
use std::{collections::HashMap, fmt};

use super::{Identifiable, Stampable};
#[derive(Debug, Clone)]
/// Defines the parameters for a PULSE voltage source.
pub struct Pulse {
    /// Initial value before the pulse.
    pub initial_value: f64,
    /// Value during the pulse.
    pub pulsed_value: f64,
    /// Time before the pulse begins.
    pub delay_time: f64,
    /// Time for the value to rise from initial to pulsed.
    pub rise_time: f64,
    /// Time for the value to fall from pulsed to initial.
    pub fall_time: f64,
    /// Duration of the pulse at its pulsed value.
    pub pulse_width: f64,
    /// Total duration of one cycle of the pulse.
    pub period: f64,
}

impl Pulse {
    /// Calculates the value of the pulse at a given time.
    pub fn value_at(&self, time: f64) -> f64 {
        if time < self.delay_time {
            return self.initial_value;
        }

        // Adjust time to be within one period cycle
        let t = (time - self.delay_time) % self.period;

        if t < self.rise_time {
            // Rising edge
            self.initial_value + (self.pulsed_value - self.initial_value) * (t / self.rise_time)
        } else if t < self.rise_time + self.pulse_width {
            // Pulse is high
            self.pulsed_value
        } else if t < self.rise_time + self.pulse_width + self.fall_time {
            // Falling edge
            self.pulsed_value
                + (self.initial_value - self.pulsed_value)
                    * ((t - self.rise_time - self.pulse_width) / self.fall_time)
        } else {
            // Pulse is low (after falling edge)
            self.initial_value
        }
    }
}

impl VoltageSource {
    /// Returns the nodes associated with the element.
    pub fn nodes(&self) -> Vec<&str> {
        vec![&self.plus, &self.minus]
    }

    /// Calculates the source's value at a specific time for transient analysis.
    pub fn transient_value_at(&self, time: f64) -> f64 {
        match &self.source_type {
            TimeVarying::Dc => self.dc_value,
            TimeVarying::Pulse(p) => p.value_at(time),
        }
    }
}

#[derive(Debug, Clone)]
/// Represents the type of a voltage source, which can be DC or time-varying.
pub enum TimeVarying {
    Dc,
    Pulse(Pulse),
}

#[derive(Debug, Clone)]
/// Represents a voltage source in a circuit.
pub struct VoltageSource {
    pub name: u32,
    pub plus: String,
    pub minus: String,
    /// The default DC value of the source.
    pub dc_value: f64,
    pub ac_amplitude: Option<f64>,
    /// The time-varying behavior of the source.
    pub source_type: TimeVarying,
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
            triplets.push(Triplet::new(ic, 0, self.dc_value));
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

    fn add_excitation_vector_transient_stamp(
        &self,
        index_map: &HashMap<String, usize>,
        solution_map: &HashMap<String, f64>,
        _prev_solution: &HashMap<String, f64>,
        _time_step: f64,
    ) -> Vec<Triplet<usize, usize, f64>> {
        let current_time = solution_map.get("time").cloned().unwrap_or(0.0);
        if let Some(&ic) = index_map.get(&format!("I({})", self.identifier())) {
            vec![Triplet::new(ic, 0, self.transient_value_at(current_time))]
        } else {
            vec![]
        }
    }
}

impl fmt::Display for VoltageSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "V{} {} {} {}",
            self.name, self.plus, self.minus, self.dc_value,
        )
    }
}

impl FromStr for VoltageSource {
    type Err = crate::prelude::Error;

    fn from_str(s: &str) -> Result<Self> {
        let s_without_comment = s.split('%').next().unwrap_or("").trim();
        let re = Regex::new(r"[()\s]+").unwrap();

        let parts: Vec<&str> = re
            .split(s_without_comment)
            .filter(|s| !s.is_empty()) // remove empty strings
            .collect();

        if !parts[0].starts_with(['V', 'v']) || parts[0].len() <= 1 {
            return Err(Error::InvalidFormat(format!(
                "Invalid voltage source identifier: '{s}'"
            )));
        }
        let name = parts[0][1..]
            .parse::<u32>()
            .map_err(|_| Error::InvalidNodeName(format!("Invalid voltage source name: '{s}'")))?;

        let plus = parts[1].to_string();
        let minus = parts[2].to_string();

        // Find where the parameters start (after plus/minus nodes)
        let mut current_idx = 3;
        let dc_value = if parts
            .get(current_idx)
            .unwrap_or(&"")
            .eq_ignore_ascii_case("DC")
        {
            current_idx += 1; // Skip "DC" keyword
            let val = parse_value(
                parts
                    .get(current_idx)
                    .ok_or_else(|| Error::InvalidFormat("Missing DC value".to_string()))?,
            )?;
            current_idx += 1;
            val
        } else {
            // If no "DC", the next part must be the value
            let val = parse_value(
                parts
                    .get(current_idx)
                    .ok_or_else(|| Error::InvalidFormat("Missing DC value".to_string()))?,
            )?;
            current_idx += 1;
            val
        };

        let mut source_type = TimeVarying::Dc;
        let mut ac_amplitude = None;

        // Parse remaining optional parts (AC, PULSE, etc.)
        while current_idx < parts.len() {
            let keyword = parts[current_idx].to_uppercase();
            match keyword.as_str() {
                "AC" => {
                    current_idx += 1;
                    ac_amplitude = Some(parse_value(parts.get(current_idx).ok_or_else(|| {
                        Error::InvalidFormat("Missing AC amplitude".to_string())
                    })?)?);
                    current_idx += 1;
                }
                "PULSE" => {
                    // PULSE (V1 V2 TD TR TF PW PER)
                    let pulse_str = s_without_comment
                        .split_at(s_without_comment.to_uppercase().find("PULSE").unwrap())
                        .1;
                    let pulse_parts_str = pulse_str
                        .trim_start_matches("PULSE")
                        .trim()
                        .trim_matches('(')
                        .trim_matches(')');
                    let pulse_parts: Vec<&str> = pulse_parts_str.split_whitespace().collect();
                    if pulse_parts.len() != 7 {
                        return Err(Error::InvalidFormat(format!(
                            "PULSE requires 7 parameters, found {} in '{}'",
                            pulse_parts.len(),
                            s
                        )));
                    }
                    source_type = TimeVarying::Pulse(Pulse {
                        initial_value: parse_value(pulse_parts[0])?,
                        pulsed_value: parse_value(pulse_parts[1])?,
                        delay_time: parse_value(pulse_parts[2])?,
                        rise_time: parse_value(pulse_parts[3])?,
                        fall_time: parse_value(pulse_parts[4])?,
                        pulse_width: parse_value(pulse_parts[5])?,
                        period: parse_value(pulse_parts[6])?,
                    });
                    current_idx = parts.len(); // PULSE is the last parameter
                }
                _ => {
                    return Err(Error::InvalidFormat(format!(
                        "Unknown voltage source parameter '{}' in '{s}'",
                        keyword
                    )));
                }
            }
        }

        Ok(VoltageSource {
            name,
            plus,
            minus,
            dc_value,
            ac_amplitude,
            source_type,
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
        assert_eq!(vs.dc_value, 5.0);
        assert_eq!(vs.ac_amplitude, None);
    }

    #[test]
    fn test_parse_with_ac() {
        let s = "V2 3 4 0 AC 1.5";
        let vs = s.parse::<VoltageSource>().unwrap();
        assert_eq!(vs.name, 2);
        assert_eq!(vs.dc_value, 0.0);
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
        assert_eq!(vs.dc_value, 5.0);
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

    #[test]
    fn test_parse_pulse() {
        let s = "V1 in 0 dc 0 PULSE(0 5 1u 100u 100u 5u 10u)";
        let vs = s.parse::<VoltageSource>().unwrap();
        assert_eq!(vs.dc_value, 0.0);
        match vs.source_type {
            TimeVarying::Pulse(p) => {
                assert_eq!(p.initial_value, 0.0);
                assert_eq!(p.pulsed_value, 5.0);
                assert!((p.delay_time - 1e-6).abs() < 1e-12);
                assert!((p.rise_time - 100e-6).abs() < 1e-12);
                assert!((p.fall_time - 100e-6).abs() < 1e-12);
                assert!((p.pulse_width - 5e-6).abs() < 1e-12);
                assert!((p.period - 10e-6).abs() < 1e-12);
            }
            _ => panic!("Expected Pulse source type"),
        }
    }

    #[test]
    fn test_pulse_value_at_time() {
        let pulse = Pulse {
            initial_value: 0.0,
            pulsed_value: 5.0,
            delay_time: 1e-6,  // 1us
            rise_time: 10e-9,  // 10ns
            fall_time: 10e-9,  // 10ns
            pulse_width: 5e-6, // 5us
            period: 10e-6,     // 10us
        };

        let epsilon = 1e-9;

        // 1. Before delay time
        assert!(
            (pulse.value_at(0.5e-6) - 0.0).abs() < epsilon,
            "Failed before delay"
        );

        // 2. Mid-rise time
        // t = 1us + 5ns = 1.005us
        assert!(
            (pulse.value_at(1.005e-6) - 2.5).abs() < epsilon,
            "Failed during rise"
        );

        // 3. During pulse width
        // t = 1us + 10ns + 2us = 3.01us
        assert!(
            (pulse.value_at(3.01e-6) - 5.0).abs() < epsilon,
            "Failed during pulse width"
        );

        // 4. Mid-fall time
        // t = 1us + 10ns + 5us + 5ns = 6.015us
        assert!(
            (pulse.value_at(6.015e-6) - 2.5).abs() < epsilon,
            "Failed during fall"
        );

        // 5. After fall time, before period ends
        // t = 8us
        assert!(
            (pulse.value_at(8.0e-6) - 0.0).abs() < epsilon,
            "Failed after fall"
        );

        // 6. In the next period, during rise time
        // t = 10us (period) + 1us (delay) + 5ns = 11.005us
        assert!(
            (pulse.value_at(11.005e-6) - 2.5).abs() < epsilon,
            "Failed in next period"
        );
    }
}
