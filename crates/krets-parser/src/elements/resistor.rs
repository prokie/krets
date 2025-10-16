use faer::sparse::Triplet;
use nom::{
    IResult, Parser, branch::alt, bytes::complete::tag, character::complete::space1,
    combinator::all_consuming, sequence::preceded,
};

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
    /// g2
    pub g2: bool,
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

        let mut triplets;

        if self.g2 {
            triplets = Vec::with_capacity(5);
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
        } else {
            triplets = Vec::with_capacity(4);

            let g = 1.0 / self.value;
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

        let mut triplets;

        if self.g2 {
            triplets = Vec::with_capacity(5);
            let one = c64::new(1.0, 0.0);
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
        } else {
            triplets = Vec::with_capacity(4);
            let g = c64::new(1.0 / self.value, 0.0);
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
fn parse_resistor(input: &str) -> IResult<&str, Resistor> {
    let (input, _) = alt((tag("R"), tag("r"))).parse(input)?;
    let (input, name) = alphanumeric_or_underscore1(input)?;
    let (input, plus) = preceded(space1, alphanumeric_or_underscore1).parse(input)?;
    let (input, minus) = preceded(space1, alphanumeric_or_underscore1).parse(input)?;
    let (input, value) = preceded(space1, value_parser).parse(input)?;

    let resistor = Resistor {
        name: name.parse().unwrap_or(0),
        plus: plus.to_string(),
        minus: minus.to_string(),
        value,
        g2: false,
    };

    Ok((input, resistor))
}

impl FromStr for Resistor {
    type Err = crate::prelude::Error;

    fn from_str(s: &str) -> Result<Self> {
        let s_without_comment = s.split('%').next().unwrap_or("").trim();
        let (_, resistor) = all_consuming(parse_resistor)
            .parse(s_without_comment)
            .map_err(|e| Error::InvalidFormat(e.to_string()))?;

        if resistor.value <= 0.0 {
            return Err(Error::InvalidFloatValue(format!(
                "Resistor value must be positive: '{s}'"
            )));
        }

        Ok(resistor)
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
