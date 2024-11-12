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
