use crate::prelude::*;
use faer::c64;
use faer::sparse::Triplet;
use nom::{
    IResult, Parser,
    branch::alt,
    bytes::complete::{is_not, tag, tag_no_case},
    character::complete::{alphanumeric1, space0, space1},
    combinator::{all_consuming, map, map_res, opt},
    multi::many0,
    number::complete::double,
    sequence::{delimited, preceded},
};
use std::str::FromStr;
use std::{collections::HashMap, fmt};

use super::{Identifiable, Stampable};

#[derive(Debug, Clone, PartialEq)]
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

/// An enum to represent the different types of optional parameters.
#[derive(Debug, PartialEq)]
enum Param {
    Dc(f64),
    Ac(f64),
    Pulse(Pulse),
}

/// Parses a DC parameter block, e.g., "dc 5.0"
fn parse_dc_param(input: &str) -> IResult<&str, Param> {
    map(preceded((tag_no_case("dc"), space1), double), Param::Dc).parse(input)
}

/// Parses an AC parameter block, e.g., "ac 10 90"
fn parse_ac_param(input: &str) -> IResult<&str, Param> {
    map(preceded((tag_no_case("ac"), space1), double), Param::Ac).parse(input)
}

/// A nom parser that recognizes a value token and parses it using our custom logic.
fn value_parser(input: &str) -> IResult<&str, f64> {
    // 1. Recognize a token (any sequence of chars that isn't a space or parenthesis).
    let token_parser = is_not(" \t\r\n()");

    // 2. Apply your custom parsing function to the recognized token.
    map_res(token_parser, parse_value).parse(input)
}

fn parse_pulse_param(input: &str) -> IResult<&str, Param> {
    // Define a parser for all the values inside the parentheses
    let values_parser = (
        preceded(space0, value_parser),
        preceded(space1, value_parser),
        preceded(space1, value_parser),
        preceded(space1, value_parser),
        preceded(space1, value_parser),
        preceded(space1, value_parser),
        preceded(space1, value_parser),
    );

    let (
        input,
        (initial_value, pulsed_value, delay_time, rise_time, fall_time, pulse_width, period),
    ) = preceded(
        tag_no_case("pulse"),
        delimited(
            preceded(space0, tag("(")),
            values_parser,
            preceded(space0, tag(")")),
        ),
    )
    .parse(input)?;

    let pulse = Pulse {
        initial_value,
        pulsed_value,
        delay_time,
        rise_time,
        fall_time,
        pulse_width,
        period,
    };

    Ok((input, Param::Pulse(pulse)))
}

/// Main nom parser for the VoltageSource
fn parse_voltage_source(input: &str) -> IResult<&str, VoltageSource> {
    let (input, _) = alt((tag("V"), tag("v"))).parse(input)?;
    let (input, name) = alphanumeric1(input)?;
    let (input, plus) = preceded(space1, alphanumeric1).parse(input)?;
    let (input, minus) = preceded(space1, alphanumeric1).parse(input)?;

    let (input, implicit_dc) = opt(preceded(space1, double)).parse(input)?;

    let parse_any_param = preceded(
        space1,
        alt((parse_dc_param, parse_ac_param, parse_pulse_param)),
    );

    // 3. Use `many0` to parse zero or more parameter blocks in any order.
    let (input, params) = many0(parse_any_param).parse(input)?;

    // 4. Process the collected parameters to build the struct
    let mut dc_value = implicit_dc.unwrap_or(0.0);
    let mut ac_amplitude = 0.0;
    let mut pulse: Option<Pulse> = None;

    for param in params {
        match param {
            Param::Dc(val) => dc_value = val,
            Param::Ac(val) => ac_amplitude = val,
            Param::Pulse(val) => pulse = Some(val),
        }
    }

    let voltage_source = VoltageSource {
        name: name.parse().unwrap_or(0),
        plus: plus.to_string(),
        minus: minus.to_string(),
        dc_value,
        ac_amplitude,
        pulse,
    };

    Ok((input, voltage_source))
}

impl VoltageSource {
    /// Returns the nodes associated with the element.
    pub fn nodes(&self) -> Vec<&str> {
        vec![&self.plus, &self.minus]
    }

    /// Calculates the source's value at a specific time for transient analysis.
    pub fn transient_value_at(&self, time: f64) -> f64 {
        if let Some(pulse) = &self.pulse {
            pulse.value_at(time)
        } else {
            self.dc_value
        }
    }
}

#[derive(Debug, Clone)]
/// Represents a voltage source in a circuit.
pub struct VoltageSource {
    pub name: u32,
    pub plus: String,
    pub minus: String,
    pub dc_value: f64,
    pub ac_amplitude: f64,
    pub pulse: Option<Pulse>,
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

        if let Some(&ic) = index_map.get(&format!("I({})", self.identifier())) {
            triplets.push(Triplet::new(ic, 0, c64::new(self.ac_amplitude, 0.0)));
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

        let (_, voltage_source) = all_consuming(parse_voltage_source)
            .parse(s_without_comment)
            .map_err(|e| Error::InvalidFormat(e.to_string()))?;

        Ok(voltage_source)
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
        assert_eq!(vs.ac_amplitude, 0.0);
    }

    #[test]
    fn test_parse_with_ac() {
        let s = "V2 3 4 0 AC 1.5";
        let vs = s.parse::<VoltageSource>().unwrap();
        assert_eq!(vs.name, 2);
        assert_eq!(vs.dc_value, 0.0);
        assert_eq!(vs.ac_amplitude, 1.5);
    }

    #[test]
    fn test_parse_case_insensitive() {
        let s = "v3 5 6 12 ac 10";
        let vs = s.parse::<VoltageSource>().unwrap();
        assert_eq!(vs.name, 3);
        assert_eq!(vs.ac_amplitude, 10.0);
    }

    #[test]
    fn test_parse_with_comment() {
        let s = "V1 1 0 5 % DC value";
        let vs = s.parse::<VoltageSource>().unwrap();
        assert_eq!(vs.dc_value, 5.0);
    }

    #[test]
    fn test_invalid_format_too_many_parts() {
        assert!("V1 1 0 5 6".parse::<VoltageSource>().is_err());
    }

    #[test]
    fn test_parse_pulse() {
        let s = "V1 in 0 dc 0 PULSE(0 5 1u 100u 100u 5u 10u)";
        let vs = s.parse::<VoltageSource>().unwrap();
        let epsilon = 1e-9;
        assert_eq!(vs.dc_value, 0.0);

        assert!(vs.pulse.is_some());
        let pulse = vs.pulse.unwrap();
        assert!((pulse.initial_value - 0.0).abs() < epsilon);
        assert!((pulse.pulsed_value - 5.0).abs() < epsilon);
        assert!((pulse.delay_time - 1e-6).abs() < epsilon);
        assert!((pulse.rise_time - 100e-6).abs() < epsilon);
        assert!((pulse.fall_time - 100e-6).abs() < epsilon);
        assert!((pulse.pulse_width - 5e-6).abs() < epsilon);
        assert!((pulse.period - 10e-6).abs() < epsilon);
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
