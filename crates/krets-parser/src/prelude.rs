use nom::bytes::complete::take_while1;
use nom::{IResult, Parser};

pub use crate::error::Error;
pub type Result<T> = core::result::Result<T, Error>;

pub use crate::utils::parse_value;

pub fn alphanumeric_or_underscore1(input: &str) -> IResult<&str, &str> {
    take_while1(|c: char| c.is_ascii_alphanumeric() || c == '_').parse(input)
}
