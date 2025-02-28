pub mod voltage_source;

pub enum Element {
    /// A voltage source element.
    VoltageSource(voltage_source::VoltageSource),
}
