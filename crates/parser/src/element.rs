use std::clone;

pub mod bipolar_junction_transistor;
pub mod capacitor;
pub mod current_source;
pub mod diode;
pub mod inductor;
pub mod nmos;
pub mod pmos;
pub mod resistor;
pub mod voltage_source;

/// Represents different types of elements in a netlist.
#[derive(Clone)]
pub enum Element {
    /// A capacitor element.
    Capacitor(capacitor::Capacitor),
    /// A resistor element.
    Resistor(resistor::Resistor),
    /// An inductor element.
    Inductor(inductor::Inductor),
    /// A voltage source element.
    VoltageSource(voltage_source::VoltageSource),
    /// A current source element.
    CurrentSource(current_source::CurrentSource),
    /// A diode element.
    Diode(diode::Diode),
    /// A bipolar junction transistor element.
    BipolarJunctionTransistor(bipolar_junction_transistor::BipolarJunctionTransistor),
    /// An NMOS transistor.
    NMOS(nmos::NMOS),
    /// A PMOS transistor.
    PMOS(pmos::PMOS),
}

impl Element {
    /// Returns the name of the element as a string slice.
    pub fn name(&self) -> &str {
        match self {
            Element::Capacitor(capacitor) => capacitor.name.as_str(),
            Element::Resistor(resistor) => resistor.name.as_str(),
            Element::Inductor(inductor) => inductor.name.as_str(),
            Element::VoltageSource(voltage_source) => voltage_source.name.as_str(),
            Element::CurrentSource(current_source) => current_source.name.as_str(),
            Element::Diode(diode) => diode.name.as_str(),
            Element::BipolarJunctionTransistor(bjt) => bjt.name.as_str(),
            Element::NMOS(nmos) => nmos.name.as_str(),
            Element::PMOS(pmos) => pmos.name.as_str(),
        }
    }
}

impl Nodes for Element {
    fn nodes(&self) -> Vec<String> {
        match self {
            Element::Capacitor(capacitor) => capacitor.nodes(),
            Element::Resistor(resistor) => resistor.nodes(),
            Element::Inductor(inductor) => inductor.nodes(),
            Element::VoltageSource(voltage_source) => voltage_source.nodes(),
            Element::CurrentSource(current_source) => current_source.nodes(),
            Element::Diode(diode) => diode.nodes(),
            Element::BipolarJunctionTransistor(bjt) => bjt.nodes(),
            Element::NMOS(nmos) => nmos.nodes(),
            Element::PMOS(pmos) => pmos.nodes(),
        }
    }
}

pub trait Nodes {
    fn nodes(&self) -> Vec<String>;
}
