use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    // crate error for trying to parse string to usize
    #[error("Invalid usize value: {0}")]
    InvalidUsizeValue(String),

    // Error when an element is not found in the netlist
    #[error("Element '{0}' not found in the netlist")]
    ElementNotFound(String),

    // Error when maximum iterations are exceeded
    #[error("Maximum iterations exceeded: {0}")]
    MaximumIterationsExceeded(usize),
}
