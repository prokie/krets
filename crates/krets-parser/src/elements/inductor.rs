use crate::prelude::*;

use nom::{
    IResult, Parser, branch::alt, bytes::complete::tag, character::complete::space1,
    combinator::all_consuming, sequence::preceded,
};

use std::f64::consts::PI;

#[derive(Debug, Clone)]
/// Represents an inductor in a circuit.
pub struct Inductor {
    /// Name of the inductor.
    pub name: String,
    /// Value of the inductor in Henries.
    pub value: f64,
    /// Positive node of the inductor.
    pub plus: String,
    /// Negative node of the inductor.
    pub minus: String,
}

impl Identifiable for Inductor {
    fn identifier(&self) -> String {
        format!("L{}", self.name)
    }
}

impl Stampable for Inductor {
    fn add_conductance_matrix_dc_stamp(
        &self,
        index_map: &HashMap<String, usize>,
        _solution_map: &HashMap<String, f64>,
    ) -> Vec<Triplet<usize, usize, f64>> {
        let index_plus = index_map.get(&format!("V({})", self.plus));
        let index_minus = index_map.get(&format!("V({})", self.minus));
        let index_current = index_map.get(&format!("I({})", self.identifier()));

        let mut triplets = Vec::with_capacity(4);

        if let (Some(&index_plus), Some(&index_current)) = (index_plus, index_current) {
            triplets.push(Triplet::new(index_plus, index_current, 1.0));
        }

        if let (Some(&index_minus), Some(&index_current)) = (index_minus, index_current) {
            triplets.push(Triplet::new(index_minus, index_current, -1.0));
        }

        if let (Some(&index_plus), Some(&index_current)) = (index_plus, index_current) {
            triplets.push(Triplet::new(index_current, index_plus, 1.0));
        }
        if let (Some(&index_minus), Some(&index_current)) = (index_minus, index_current) {
            triplets.push(Triplet::new(index_current, index_minus, -1.0));
        }

        triplets
    }

    fn add_conductance_matrix_ac_stamp(
        &self,
        index_map: &HashMap<String, usize>,
        _solution_map: &HashMap<String, f64>,
        frequency: f64,
    ) -> Vec<Triplet<usize, usize, c64>> {
        let index_plus = index_map.get(&format!("V({})", self.plus));
        let index_minus = index_map.get(&format!("V({})", self.minus));
        let index_current = index_map.get(&format!("I({})", self.identifier()));
        let impedance = c64::new(0.0, 2.0 * PI * frequency * self.value);
        let mut triplets = Vec::with_capacity(5);

        if let (Some(&index_plus), Some(&index_current)) = (index_plus, index_current) {
            triplets.push(Triplet::new(index_plus, index_current, c64::new(1.0, 0.0)));
        }

        if let (Some(&index_minus), Some(&index_current)) = (index_minus, index_current) {
            triplets.push(Triplet::new(
                index_minus,
                index_current,
                c64::new(-1.0, 0.0),
            ));
        }

        if let (Some(&index_plus), Some(&index_current)) = (index_plus, index_current) {
            triplets.push(Triplet::new(index_current, index_plus, c64::new(1.0, 0.0)));
        }
        if let (Some(&index_minus), Some(&index_current)) = (index_minus, index_current) {
            triplets.push(Triplet::new(
                index_current,
                index_minus,
                c64::new(-1.0, 0.0),
            ));
        }

        if let Some(&index_current) = index_current {
            triplets.push(Triplet::new(index_current, index_current, -impedance));
        }

        triplets
    }

    fn add_excitation_vector_dc_stamp(
        &self,
        _index_map: &HashMap<String, usize>,
        _solution_map: &HashMap<String, f64>,
    ) -> Vec<Triplet<usize, usize, f64>> {
        // An ideal inductor is passive and has no internal sources.
        vec![]
    }

    fn add_excitation_vector_ac_stamp(
        &self,
        _index_map: &HashMap<String, usize>,
        _solution_map: &HashMap<String, f64>,
        _frequency: f64,
    ) -> Vec<Triplet<usize, usize, c64>> {
        // An ideal inductor is passive and has no internal sources.
        vec![]
    }

    fn add_conductance_matrix_transient_stamp(
        &self,
        index_map: &HashMap<String, usize>,
        _solution_map: &HashMap<String, f64>,
        _prev_solution: &HashMap<String, f64>,
        h: f64,
    ) -> Vec<Triplet<usize, usize, f64>> {
        let index_plus = index_map.get(&format!("V({})", self.plus));
        let index_minus = index_map.get(&format!("V({})", self.minus));
        let index_current = index_map.get(&format!("I({})", self.identifier()));

        let mut triplets = Vec::with_capacity(5);

        if let Some(&ic) = index_current {
            triplets.push(Triplet::new(ic, ic, -self.value / h));
        }

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

    fn add_excitation_vector_transient_stamp(
        &self,
        index_map: &HashMap<String, usize>,
        _solution_map: &HashMap<String, f64>,
        prev_solution: &HashMap<String, f64>,
        h: f64,
    ) -> Vec<Triplet<usize, usize, f64>> {
        let index_current = index_map.get(&format!("I({})", self.identifier()));

        let i_prev = prev_solution
            .get(&format!("I({})", self.identifier()))
            .copied()
            .unwrap();

        if let Some(&ic) = index_current {
            vec![Triplet::new(ic, 0, -(self.value / h) * i_prev)]
        } else {
            vec![]
        }
    }
}
fn parse_inductor(input: &str) -> IResult<&str, Inductor> {
    let (input, _) = alt((tag("L"), tag("l"))).parse(input)?;
    let (input, name) = alphanumeric_or_underscore1(input)?;
    let (input, plus) = preceded(space1, alphanumeric_or_underscore1).parse(input)?;
    let (input, minus) = preceded(space1, alphanumeric_or_underscore1).parse(input)?;
    let (input, value) = preceded(space1, value_parser).parse(input)?;

    let inductor = Inductor {
        name: name.to_string(),
        plus: plus.to_string(),
        minus: minus.to_string(),
        value,
    };

    Ok((input, inductor))
}

impl FromStr for Inductor {
    type Err = crate::prelude::Error;

    fn from_str(s: &str) -> Result<Self> {
        let s_without_comment = s.split('%').next().unwrap_or("").trim();
        let (_, inductor) = all_consuming(parse_inductor)
            .parse(s_without_comment)
            .map_err(|e| Error::InvalidFormat(e.to_string()))?;

        Ok(inductor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_inductor() {
        let inductor_str = "L1 1 0 0.001";
        let inductor = inductor_str.parse::<Inductor>().unwrap();

        assert_eq!(inductor.name, "1");
        assert_eq!(inductor.plus, "1");
        assert_eq!(inductor.minus, "0");
        assert_eq!(inductor.value, 0.001);
    }

    #[test]
    fn test_parse_inductor_with_comment() {
        let inductor_str = "L1 1 0 0.001 % This is a comment";
        let inductor = inductor_str.parse::<Inductor>().unwrap();

        assert_eq!(inductor.name, "1");
        assert_eq!(inductor.value, 0.001);
    }

    #[test]
    fn test_parse_lowercase_and_scientific() {
        let s = "l2 vcc out 1e-6";
        let inductor = s.parse::<Inductor>().unwrap();
        assert_eq!(inductor.name, "2");
        assert_eq!(inductor.value, 1e-6);
    }

    #[test]
    fn test_invalid_inductor_format() {
        let inductor_str = "L1 1 0";
        let result = inductor_str.parse::<Inductor>();
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_inductor_name() {
        let inductor_str = "L 1 0 0.001";
        let result = inductor_str.parse::<Inductor>();
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_prefix() {
        let s = "R1 1 0 100";
        let result = s.parse::<Inductor>();
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_inductor_value() {
        let inductor_str = "L1 1 0 abc";
        let result = inductor_str.parse::<Inductor>();
        assert!(result.is_err());
    }
}
