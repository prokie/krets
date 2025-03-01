use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    // crate error for trying to parse string to usize
    #[error("Invalid usize value: {0}")]
    InvalidUsizeValue(String),
}
