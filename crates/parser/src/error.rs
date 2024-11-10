use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Invalid format: {0}")]
    InvalidFormat(String),

    #[error("Invalid float value: {0}")]
    InvalidFloatValue(String),

    #[error("Unknown element: {0}")]
    UnknownElement(String),

    #[error("Unexpected error: {0}")]
    Unexpected(String),

    #[error("The netlist is empty")]
    EmptyNetlist,
}
