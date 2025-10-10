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

    // Error indicating that the format of an element in the input string is invalid.
    #[error("Invalid element format: {0}")]
    InvalidElementFormat(String),

    // Error indicating that a float value could not be parsed.
    #[error("Invalid format: {0}")]
    Unexpected(String),

    // Error indicating that a float value could not be parsed.
    #[error("Decomposition failed")]
    DecompositionFailed,
}
