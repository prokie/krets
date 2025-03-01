pub mod current_source;
pub mod voltage_source;

/// Represents a circuit element.
#[derive(Debug)]
pub enum Element {
    /// A voltage source element.
    VoltageSource(voltage_source::VoltageSource),

    /// A current source element.
    CurrentSource(current_source::CurrentSource),
}
