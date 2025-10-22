use crate::prelude::*;

use nom::{
    IResult, Parser,
    branch::alt,
    bytes::complete::{tag, tag_no_case},
    character::complete::space1,
    combinator::{all_consuming, opt},
    sequence::preceded,
};
use std::f64::consts::PI;

#[derive(Debug, Clone)]
/// Represents a capacitor in a circuit.
pub struct Capacitor {
    /// Name of the capacitor.
    pub name: String,
    /// Value of the capacitor.
    pub value: f64,
    /// Positive node of the capacitor.
    pub plus: String,
    /// Negative node of the capacitor.
    pub minus: String,
    /// If the capacitor is G2.
    pub g2: bool,
}

impl Identifiable for Capacitor {
    fn identifier(&self) -> String {
        format!("C{}", self.name)
    }
}

impl Stampable for Capacitor {
    fn add_conductance_matrix_dc_stamp(
        &self,
        _index_map: &HashMap<String, usize>,
        _solution_map: &HashMap<String, f64>,
    ) -> Vec<faer::sparse::Triplet<usize, usize, f64>> {
        // A capacitor is an open circuit in DC analysis, so it contributes nothing to the DC conductance matrix.
        vec![]
    }

    fn add_conductance_matrix_ac_stamp(
        &self,
        index_map: &HashMap<String, usize>,
        _solution_map: &HashMap<String, f64>,
        frequency: f64,
    ) -> Vec<faer::sparse::Triplet<usize, usize, c64>> {
        let index_plus = index_map.get(&format!("V({})", self.plus));
        let index_minus = index_map.get(&format!("V({})", self.minus));

        let admittance = c64 {
            re: 0.0,
            im: 2.0 * PI * frequency * self.value,
        };

        let mut triplets = Vec::with_capacity(4);

        if !self.g2 {
            if let Some(&index_plus) = index_plus {
                triplets.push(Triplet::new(index_plus, index_plus, admittance));
            }
            if let Some(&index_minus) = index_minus {
                triplets.push(Triplet::new(index_minus, index_minus, admittance));
            }
            if let (Some(&index_plus), Some(&index_minus)) = (index_plus, index_minus) {
                triplets.push(Triplet::new(index_plus, index_minus, -admittance));
                triplets.push(Triplet::new(index_minus, index_plus, -admittance));
            }
        } else {
            let index_current = index_map.get(&format!("I({})", self.identifier()));

            if let (Some(&index_plus), Some(&index_current)) = (index_plus, index_current) {
                // -Y contribution for V_plus
                triplets.push(Triplet::new(index_current, index_plus, -admittance));
            }

            if let (Some(&index_minus), Some(&index_current)) = (index_minus, index_current) {
                // +Y contribution for V_minus
                triplets.push(Triplet::new(index_current, index_minus, admittance));
            }

            if let Some(&index_current) = index_current {
                // +1 contribution for I_c
                triplets.push(Triplet::new(
                    index_current,
                    index_current,
                    c64 { re: 1.0, im: 0.0 },
                ));
            }
        }

        triplets
    }

    fn add_excitation_vector_dc_stamp(
        &self,
        _index_map: &HashMap<String, usize>,
        _solution_map: &HashMap<String, f64>,
    ) -> Vec<faer::sparse::Triplet<usize, usize, f64>> {
        // Capacitors are passive and don't contribute to the DC excitation vector.
        vec![]
    }

    fn add_excitation_vector_ac_stamp(
        &self,
        _index_map: &HashMap<String, usize>,
        _solution_map: &HashMap<String, f64>,
        _frequency: f64,
    ) -> Vec<faer::sparse::Triplet<usize, usize, c64>> {
        // Capacitors are passive and don't contribute to the AC excitation vector.
        vec![]
    }

    fn add_conductance_matrix_transient_stamp(
        &self,
        index_map: &HashMap<String, usize>,
        _solution_map: &HashMap<String, f64>, // Not needed for a linear capacitor's conductance
        _prev_solution: &HashMap<String, f64>, // Not needed for a linear capacitor's conductance
        h: f64,
    ) -> Vec<Triplet<usize, usize, f64>> {
        let g = self.value / h;

        let index_plus = index_map.get(&format!("V({})", self.plus));
        let index_minus = index_map.get(&format!("V({})", self.minus));

        let mut triplets = Vec::with_capacity(4);

        if let Some(&ip) = index_plus {
            triplets.push(Triplet::new(ip, ip, g));
        }
        if let Some(&im) = index_minus {
            triplets.push(Triplet::new(im, im, g));
        }
        if let (Some(&ip), Some(&im)) = (index_plus, index_minus) {
            triplets.push(Triplet::new(ip, im, -g));
            triplets.push(Triplet::new(im, ip, -g));
        }

        triplets
    }

    fn add_excitation_vector_transient_stamp(
        &self,
        index_map: &HashMap<String, usize>,
        _solution_map: &HashMap<String, f64>, // Not needed for a linear capacitor's excitation
        prev_solution: &HashMap<String, f64>,
        h: f64,
    ) -> Vec<Triplet<usize, usize, f64>> {
        let index_plus = index_map.get(&format!("V({})", self.plus));
        let index_minus = index_map.get(&format!("V({})", self.minus));

        // Get the capacitor's voltage from the PREVIOUS time step.
        // Default to 0.0 if a node is not in the map (e.g., ground or first step).
        let v_plus_prev = prev_solution
            .get(&format!("V({})", self.plus))
            .copied()
            .unwrap_or(0.0);
        let v_minus_prev = prev_solution
            .get(&format!("V({})", self.minus))
            .copied()
            .unwrap_or(0.0);
        let v_prev = v_plus_prev - v_minus_prev;

        // Calculate the equivalent current source value: I_eq = (C/h) * v_prev
        let i_eq = -(self.value / h) * v_prev;

        let mut triplets = Vec::with_capacity(2);

        if let Some(&ip) = index_plus {
            triplets.push(Triplet::new(ip, 0, -i_eq));
        }
        if let Some(&im) = index_minus {
            triplets.push(Triplet::new(im, 0, i_eq));
        }

        triplets
    }
}
fn parse_capacitor(input: &str) -> IResult<&str, Capacitor> {
    let (input, _) = alt((tag("C"), tag("c"))).parse(input)?;
    let (input, name) = alphanumeric_or_underscore1(input)?;
    let (input, plus) = preceded(space1, alphanumeric_or_underscore1).parse(input)?;
    let (input, minus) = preceded(space1, alphanumeric_or_underscore1).parse(input)?;
    let (input, value) = preceded(space1, value_parser).parse(input)?;
    let (input, g2_opt) = opt(preceded(space1, tag_no_case("G2"))).parse(input)?;

    let capacitor = Capacitor {
        name: name.to_string(),
        plus: plus.to_string(),
        minus: minus.to_string(),
        value,
        g2: g2_opt.is_some(),
    };

    Ok((input, capacitor))
}

impl FromStr for Capacitor {
    type Err = crate::prelude::Error;

    fn from_str(s: &str) -> Result<Self> {
        let s_without_comment = s.split('%').next().unwrap_or("").trim();

        let (_, capacitor) = all_consuming(parse_capacitor)
            .parse(s_without_comment)
            .map_err(|e| Error::InvalidFormat(e.to_string()))?;

        Ok(capacitor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_capacitor() {
        let capacitor_str = "C1 1 0 0.000001";
        let capacitor = capacitor_str.parse::<Capacitor>().unwrap();

        assert_eq!(capacitor.name, "1");
        assert_eq!(capacitor.plus, "1");
        assert_eq!(capacitor.minus, "0");
        assert_eq!(capacitor.value, 0.000001);
        assert!(!capacitor.g2);
    }

    #[test]
    fn test_parse_capacitor_with_group() {
        let capacitor_str = "C1 1 0 0.000001 G2";
        let capacitor = capacitor_str.parse::<Capacitor>().unwrap();

        assert_eq!(capacitor.name, "1");
        assert_eq!(capacitor.plus, "1");
        assert_eq!(capacitor.minus, "0");
        assert_eq!(capacitor.value, 0.000001);
        assert!(capacitor.g2);
    }

    #[test]
    fn test_parse_capacitor_with_comment() {
        let capacitor_str = "C1 1 0 0.000001 % This is a comment";
        let capacitor = capacitor_str.parse::<Capacitor>().unwrap();

        assert_eq!(capacitor.name, "1");
        assert_eq!(capacitor.plus, "1");
        assert_eq!(capacitor.minus, "0");
        assert_eq!(capacitor.value, 0.000001);
        assert!(!capacitor.g2);
    }

    #[test]
    fn test_parse_capacitor_with_comment_no_space() {
        let capacitor_str = "C1 1 0 1e-6%This is a comment";
        let capacitor = capacitor_str.parse::<Capacitor>().unwrap();

        assert_eq!(capacitor.name, "1");
        assert_eq!(capacitor.value, 1e-6);
        assert!(!capacitor.g2);
    }

    #[test]
    fn test_parse_capacitor_with_g2_and_comment() {
        let capacitor_str = "c2 3 4 10e-9 G2 % comment";
        let capacitor = capacitor_str.parse::<Capacitor>().unwrap();

        assert_eq!(capacitor.name, "2");
        assert_eq!(capacitor.value, 10e-9);
        assert!(capacitor.g2);
    }

    #[test]
    fn test_parse_lowercase() {
        let capacitor_str = "c1 1 0 1e-6 g2";
        let capacitor = capacitor_str.parse::<Capacitor>().unwrap();

        assert_eq!(capacitor.name, "1");
        assert!(capacitor.g2);
    }

    #[test]
    fn test_invalid_capacitor_format() {
        let capacitor_str = "C1 1 0";
        let result = capacitor_str.parse::<Capacitor>();
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_capacitor_name() {
        let capacitor_str = "C 1 0 0.000001";
        let result = capacitor_str.parse::<Capacitor>();
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_identifier_prefix() {
        let capacitor_str = "R1 1 0 100";
        let result = capacitor_str.parse::<Capacitor>();
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_fifth_argument() {
        let capacitor_str = "C1 1 0 1e-6 G3";
        let result = capacitor_str.parse::<Capacitor>();
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_capacitor_value() {
        let capacitor_str = "C1 1 0 abc";
        let result = capacitor_str.parse::<Capacitor>();
        assert!(result.is_err());
    }
}
