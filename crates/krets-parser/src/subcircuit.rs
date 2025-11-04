use crate::prelude::*;
use nom::{
    IResult, Parser, bytes::complete::tag_no_case, character::complete::space1, multi::many0,
    sequence::preceded,
};
pub struct Subcircuit {
    pub name: String,
    pub pins: Vec<String>,
    pub elements: Vec<Element>,
}

impl Subcircuit {
    pub fn new(name: impl Into<String>, pins: Vec<&str>) -> Self {
        Self {
            name: name.into(),
            pins: pins.into_iter().map(Into::into).collect(),
            elements: Vec::new(),
        }
    }
}

pub fn parse_subckt_header(input: &str) -> IResult<&str, Subcircuit> {
    let (input, _) = tag_no_case(".subckt").parse(input)?;
    let (input, name) = preceded(space1, alphanumeric_or_underscore1).parse(input)?;
    let (input, pins) = many0(preceded(space1, alphanumeric_or_underscore1)).parse(input)?;
    Ok((input, Subcircuit::new(name, pins)))
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_header() {
        let subckt_str = ".SUBCKT my_subckt in out vdd gnd";
        let (_, subckt) = parse_subckt_header(subckt_str).unwrap();
        assert_eq!(subckt.name, "my_subckt");
        assert_eq!(subckt.pins, vec!["in", "out", "vdd", "gnd"]);
    }
}
