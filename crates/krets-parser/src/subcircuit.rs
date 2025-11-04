use crate::prelude::*;
use nom::{
    IResult, Parser, bytes::complete::tag_no_case, character::complete::space1, multi::many0,
    sequence::preceded,
};
#[derive(Debug, Clone)]
pub struct Subcircuit {
    pub name: String,
    pub pins: Vec<String>,
    pub elements: Vec<Element>,
}

#[derive(Debug, Clone)]
pub struct SubcircuitInstance {
    pub instance_name: String,
    pub definition_name: String,
    pub nodes: Vec<String>,
}

impl SubcircuitInstance {
    pub fn new(
        instance_name: impl Into<String>,
        definition_name: impl Into<String>,
        nodes: Vec<&str>,
    ) -> Self {
        Self {
            instance_name: instance_name.into(),
            definition_name: definition_name.into(),
            nodes: nodes.into_iter().map(Into::into).collect(),
        }
    }
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

pub fn parse_subckt_instance(input: &str) -> IResult<&str, SubcircuitInstance> {
    let (input, _) = tag_no_case("x").parse(input)?;
    let (input, instance_name) = alphanumeric_or_underscore1(input)?;
    let (input, nodes) = many0(preceded(space1, alphanumeric_or_underscore1)).parse(input)?;

    let definition_name = nodes.last().unwrap();
    let nodes = &nodes[..nodes.len() - 1];
    Ok((
        input,
        SubcircuitInstance::new(
            instance_name.to_string(),
            definition_name.to_string(),
            nodes.to_vec(),
        ),
    ))
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
