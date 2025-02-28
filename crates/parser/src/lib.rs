use elements::Element;

pub mod elements;
pub mod error;
pub mod prelude;
use crate::prelude::*;

pub struct Netlist {
    pub elements: Vec<Element>,
}

pub fn parse_circuit_description(input: &str) -> Result<Netlist> {
    let lines: Vec<&str> = input.trim().lines().map(str::trim).collect();

    if lines.is_empty() {
        return Err(Error::EmptyNetlist);
    }

    let mut elements: Vec<Element> = Vec::new();

    for line in lines {
        if line.is_empty() {
            continue;
        }

        if line.starts_with("V") {
            elements.push(Element::VoltageSource(line.parse()?));
        }
    }

    let netlist = Netlist { elements };

    Ok(netlist)
}
