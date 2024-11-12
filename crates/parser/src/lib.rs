#![allow(dead_code)]

pub mod element;
pub mod error;
pub mod prelude;
pub mod utils;
use crate::prelude::*;

use element::{Element, ElementKind};
use error::Error;

pub struct Netlist {
    pub elements: Vec<ElementKind>,
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
        if line.starts_with('C') {
            elements.push(element::capacitor::Capacitor(
                line.parse::<element::capacitor::Capacitor>()?,
            ));
        } else if line.starts_with('R') {
            elements.push(Element::Resistor(
                line.parse::<element::resistor::Resistor>()?,
            ));
        } else if line.starts_with('L') {
            elements.push(Element::Inductor(
                line.parse::<element::inductor::Inductor>()?,
            ));
        } else if line.starts_with('V') {
            elements.push(Element::VoltageSource(
                line.parse::<element::voltage_source::VoltageSource>()?,
            ));
        } else if line.starts_with('I') {
            elements.push(Element::CurrentSource(
                line.parse::<element::current_source::CurrentSource>()?,
            ));
        } else if line.starts_with('D') {
            elements.push(Element::Diode(line.parse::<element::diode::Diode>()?));
        } else if line.starts_with('Q') {
            elements.push(Element::BipolarJunctionTransistor(
                line.parse::<element::bipolar_junction_transistor::BipolarJunctionTransistor>()?,
            ));
        } else if line.starts_with('M') {
            elements.push(Element::Mosfet(line.parse::<element::mosfet::Mosfet>()?));
        } else {
            return Err(Error::UnknownElement(line.to_string()));
        }
    }

    let netlist = Netlist { elements, title };

    Ok(netlist)
}
