#![allow(dead_code)]

pub mod element;
pub mod error;
pub mod prelude;
pub mod utils;
use crate::prelude::*;

use element::Element;
use error::Error;

pub struct Netlist {
    pub elements: Vec<Element>,
    pub title: String,
}

/// Parses a netlist string and returns a list of elements.
///
/// # Arguments
///
/// * `spice_deck` - A string slice that holds the netlist to be parsed.
///
/// # Returns
///
/// * `Ok([Netlist])` - A `Netlist` struct containing the parsed elements if the parsing is successful.
/// * `Err([ParseError])` - An error if the parsing fails.
///
/// # Errors
///
/// This function will return an error in the following cases:
///
/// * [`ParseError::EmptyNetlist`] - If the provided netlist string is empty.
/// * [`ParseError::InvalidFormat`] - If the netlist format is invalid.
/// * [`ParseError::UnknownElement`] - If an unknown element is encountered.
/// * [`ParseError::Unexpected`] - If an unexpected error occurs.
pub fn parse_netlist(spice_deck: &str) -> Result<Netlist> {
    let mut lines: Vec<&str> = spice_deck.trim().lines().map(str::trim).collect();

    if lines.is_empty() {
        return Err(Error::EmptyNetlist);
    }

    let title = lines.remove(0).to_string();

    let mut elements: Vec<Element> = Vec::new();

    for line in lines {
        if line.is_empty() {
            continue;
        }

        if line.starts_with('C') {
            elements.push(Element::Capacitor(line.parse()?));
        } else if line.starts_with('R') {
            elements.push(Element::Resistor(line.parse()?));
        } else if line.starts_with('L') {
            elements.push(Element::Inductor(line.parse()?));
        } else if line.starts_with('V') {
            elements.push(Element::VoltageSource(line.parse()?));
        } else if line.starts_with('I') {
            elements.push(Element::CurrentSource(line.parse()?));
        } else if line.starts_with('D') {
            elements.push(Element::Diode(line.parse()?));
        } else if line.starts_with('Q') {
            elements.push(Element::BipolarJunctionTransistor(line.parse()?));
        } else if line.starts_with("QN") {
            elements.push(Element::NMOS(line.parse()?));
        } else if line.starts_with("QP") {
            elements.push(Element::PMOS(line.parse()?));
        } else {
            return Err(Error::UnknownElement(line.to_string()));
        }
    }

    let netlist = Netlist { elements, title };

    Ok(netlist)
}
