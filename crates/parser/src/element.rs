pub mod bipolar_junction_transistor;
pub mod capacitor;
pub mod current_source;
pub mod diode;
pub mod inductor;
pub mod mosfet;
pub mod resistor;
pub mod voltage_source;

/// Represents different types of elements in a netlist.
pub enum Element {
    /// A capacitor element.
    Capacitor(capacitor::Capacitor),
    Resistor(resistor::Resistor),
    Inductor(inductor::Inductor),
    VoltageSource(voltage_source::VoltageSource),
    CurrentSource(current_source::CurrentSource),
    Diode(diode::Diode),
    BipolarJunctionTransistor(bipolar_junction_transistor::BipolarJunctionTransistor),
    Mosfet(mosfet::Mosfet),
}
