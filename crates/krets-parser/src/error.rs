use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    /// Error indicating that the format of the input string is invalid.
    #[error("Invalid format: {0}")]
    InvalidFormat(String),

    /// Error indicating that a float value in the input string is invalid.
    #[error("Invalid float value: {0}")]
    InvalidFloatValue(String),

    /// Error indicating that an unknown element was encountered in the input string.
    #[error("Unknown element: {0}")]
    UnknownElement(String),

    /// Error indicating that an unexpected error occurred.
    #[error("Unexpected error: {0}")]
    Unexpected(String),

    /// Error indicating that the netlist is empty.
    #[error("The netlist is empty")]
    EmptyNetlist,

    /// Error indicating that a node name in the input string is invalid.
    #[error("Invalid node name: {0}")]
    InvalidNodeName(String),

    /// Error indicating that the format of an element in the input string is invalid.
    #[error("Invalid element format: {0}")]
    InvalidElementFormat(String),

    /// Error indicating that an unknown element type was encountered in the input string.
    #[error("Unknown element type: {0}")]
    UnknownElementType(String),

    /// Error indicating a parsing failure on a specific line of the netlist.
    #[error("Parse error on line {line}: {message}")]
    ParseError { line: usize, message: String },

    #[error("IO error reading file: {0}")]
    Io(#[from] std::io::Error),

    #[error("TOML deserialization error: {0}")]
    Toml(#[from] toml::de::Error),
}
