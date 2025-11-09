use crate::prelude::*;
use nom::{Parser, branch::alt};
pub mod bjt;
pub mod capacitor;
pub mod current_source;
pub mod diode;
pub mod inductor;
pub mod nmosfet;
pub mod resistor;
pub mod subcircuit;
pub mod voltage_source;
/// Represents any component that can be included in a circuit simulation.
#[derive(Debug, Clone)]
pub enum Element {
    VoltageSource(voltage_source::VoltageSource),
    CurrentSource(current_source::CurrentSource),
    Resistor(resistor::Resistor),
    Capacitor(capacitor::Capacitor),
    Inductor(inductor::Inductor),
    Diode(diode::Diode),
    BJT(bjt::BJT),
    NMOSFET(nmosfet::NMOSFET),
    SubcktInstance(subcircuit::SubcircuitInstance),
}

/// A macro to forward a method call to the correct inner element struct.
/// This reduces boilerplate code for the `Element` enum wrappers.
macro_rules! dispatch {
    ($self:expr, $method:ident($($args:expr),*)) => {
        match $self {
            Element::VoltageSource(e) => e.$method($($args),*),
            Element::CurrentSource(e) => e.$method($($args),*),
            Element::Resistor(e) => e.$method($($args),*),
            Element::Capacitor(e) => e.$method($($args),*),
            Element::Inductor(e) => e.$method($($args),*),
            Element::Diode(e) => e.$method($($args),*),
            Element::BJT(e) => e.$method($($args),*),
            Element::NMOSFET(e) => e.$method($($args),*),
            Element::SubcktInstance(e) => e.$method($($args),*),
        }
    };
}

pub fn parse_element(input: &str) -> Result<Element> {
    let (_, element) = alt((
        map(parse_resistor, Element::Resistor),
        map(parse_capacitor, Element::Capacitor),
        map(parse_inductor, Element::Inductor),
        map(parse_voltage_source, Element::VoltageSource),
        map(parse_current_source, Element::CurrentSource),
        map(parse_diode, Element::Diode),
        map(parse_bjt, Element::BJT),
        map(parse_nmosfet, Element::NMOSFET),
        map(parse_subckt_instance, Element::SubcktInstance),
    ))
    .parse(input)
    .map_err(|e| {
        Error::Unexpected(format!(
            "Failed to parse element from input '{}': parser error: {:?}",
            input, e
        ))
    })?;

    Ok(element)
}

impl From<voltage_source::VoltageSource> for Element {
    fn from(item: voltage_source::VoltageSource) -> Self {
        Element::VoltageSource(item)
    }
}
impl From<current_source::CurrentSource> for Element {
    fn from(item: current_source::CurrentSource) -> Self {
        Element::CurrentSource(item)
    }
}
impl From<resistor::Resistor> for Element {
    fn from(item: resistor::Resistor) -> Self {
        Element::Resistor(item)
    }
}
impl From<capacitor::Capacitor> for Element {
    fn from(item: capacitor::Capacitor) -> Self {
        Element::Capacitor(item)
    }
}
impl From<inductor::Inductor> for Element {
    fn from(item: inductor::Inductor) -> Self {
        Element::Inductor(item)
    }
}
impl From<diode::Diode> for Element {
    fn from(item: diode::Diode) -> Self {
        Element::Diode(item)
    }
}
impl From<bjt::BJT> for Element {
    fn from(item: bjt::BJT) -> Self {
        Element::BJT(item)
    }
}
impl From<nmosfet::NMOSFET> for Element {
    fn from(item: nmosfet::NMOSFET) -> Self {
        Element::NMOSFET(item)
    }
}

impl Element {
    /// Retrieves the nodes associated with the element.
    pub fn nodes(&self) -> Vec<&str> {
        match self {
            Element::VoltageSource(v) => vec![&v.plus, &v.minus],
            Element::CurrentSource(i) => vec![&i.plus, &i.minus],
            Element::Resistor(r) => vec![&r.plus, &r.minus],
            Element::Capacitor(c) => vec![&c.plus, &c.minus],
            Element::Inductor(l) => vec![&l.plus, &l.minus],
            Element::Diode(d) => vec![&d.plus, &d.minus],
            Element::BJT(b) => vec![&b.collector, &b.emitter, &b.base],
            Element::NMOSFET(m) => vec![&m.drain, &m.gate, &m.source],
            Element::SubcktInstance(s) => s.nodes.iter().map(String::as_str).collect(),
        }
    }

    pub fn nodes_mut(&mut self) -> Vec<&mut String> {
        match self {
            Element::VoltageSource(v) => vec![&mut v.plus, &mut v.minus],
            Element::CurrentSource(i) => vec![&mut i.plus, &mut i.minus],
            Element::Resistor(r) => vec![&mut r.plus, &mut r.minus],
            Element::Capacitor(c) => vec![&mut c.plus, &mut c.minus],
            Element::Inductor(l) => vec![&mut l.plus, &mut l.minus],
            Element::Diode(d) => vec![&mut d.plus, &mut d.minus],
            Element::BJT(b) => vec![&mut b.collector, &mut b.emitter, &mut b.base],
            Element::NMOSFET(m) => vec![&mut m.drain, &mut m.gate, &mut m.source],
            Element::SubcktInstance(s) => s.nodes.iter_mut().collect(),
        }
    }

    pub fn name(&self) -> &str {
        match self {
            Element::VoltageSource(v) => &v.name,
            Element::CurrentSource(i) => &i.name,
            Element::Resistor(r) => &r.name,
            Element::Capacitor(c) => &c.name,
            Element::Inductor(l) => &l.name,
            Element::Diode(d) => &d.name,
            Element::BJT(b) => &b.name,
            Element::NMOSFET(m) => &m.name,
            Element::SubcktInstance(s) => &s.instance_name,
        }
    }
    pub fn set_name(&mut self, new_name: &str) {
        match self {
            Element::VoltageSource(v) => v.name = new_name.to_string(),
            Element::CurrentSource(i) => i.name = new_name.to_string(),
            Element::Resistor(r) => r.name = new_name.to_string(),
            Element::Capacitor(c) => c.name = new_name.to_string(),
            Element::Inductor(l) => l.name = new_name.to_string(),
            Element::Diode(d) => d.name = new_name.to_string(),
            Element::BJT(b) => b.name = new_name.to_string(),
            Element::NMOSFET(m) => m.name = new_name.to_string(),
            Element::SubcktInstance(s) => s.instance_name = new_name.to_string(),
        }
    }

    /// Determines if the element requires a dedicated branch current (Group 2) in MNA.
    pub fn is_g2(&self) -> bool {
        match self {
            // Voltage sources and inductors are always group 2.
            Element::VoltageSource(_) => true,
            Element::Inductor(_) => true,
            // The parser determines if these are Group 2.
            Element::Resistor(e) => e.g2,
            Element::Capacitor(e) => e.g2,
            Element::CurrentSource(_) => true,
            // Non-linear elements are linearized into Group 1 companion models.
            Element::Diode(_)
            | Element::BJT(_)
            | Element::NMOSFET(_)
            | Element::SubcktInstance(_) => false,
        }
    }

    /// Checks if the element is non-linear.
    pub fn is_nonlinear(&self) -> bool {
        matches!(
            self,
            Element::Diode(_) | Element::BJT(_) | Element::NMOSFET(_)
        )
    }

    pub fn identifier(&self) -> String {
        dispatch!(self, identifier())
    }
}

impl std::fmt::Display for Element {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.identifier())
    }
}
