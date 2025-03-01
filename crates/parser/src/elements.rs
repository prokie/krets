pub mod bjt;
pub mod capacitor;
pub mod current_source;
pub mod diode;
pub mod inductor;
pub mod mosfet;
pub mod resistor;
pub mod voltage_source;

/// Represents a circuit element.
///
/// # Element Groups
/// Definition 2.6. (Element Groups) All elements whose currents are to be eliminated will be
/// referred to as being in group 1, while all other elements will be referred to as group 2.
#[derive(Debug)]
pub enum Element {
    /// A voltage source element.
    VoltageSource(voltage_source::VoltageSource),

    /// A current source element.
    CurrentSource(current_source::CurrentSource),

    /// A resistor element.
    Resistor(resistor::Resistor),

    /// A capacitor element.
    Capacitor(capacitor::Capacitor),

    /// An inductor element.
    Inductor(inductor::Inductor),

    /// A diode element.
    Diode(diode::Diode),

    /// Bipolar Junction Transistor (BJT) element.
    BJT(bjt::BJT),

    /// Metal-Oxide-Semiconductor Field-Effect Transistor (MOSFET) element.
    MOSFET(mosfet::MOSFET),
}
