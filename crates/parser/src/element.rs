pub mod bipolar_junction_transistor;
pub mod capacitor;
pub mod current_source;
pub mod diode;
pub mod inductor;
pub mod mosfet;
pub mod resistor;
pub mod voltage_source;

pub struct Element {
    pub name: String,
    pub value: f64,
    pub node1: String,
    pub node2: String,
    pub kind: ElementKind,
}

/// Represents different types of elements in a netlist.
pub enum ElementKind {
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
    /// A mosfet element.
    Mosfet(mosfet::Mosfet),
}
