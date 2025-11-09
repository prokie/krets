pub use crate::error::Error;
pub type Result<T> = core::result::Result<T, Error>;

pub use crate::elements::Element;
pub use crate::elements::bjt::parse_bjt;
pub use crate::elements::capacitor::parse_capacitor;
pub use crate::elements::current_source::parse_current_source;
pub use crate::elements::diode::parse_diode;
pub use crate::elements::inductor::parse_inductor;
pub use crate::elements::nmosfet::parse_nmosfet;
pub use crate::elements::parse_element;
pub use crate::elements::resistor::parse_resistor;
pub use crate::elements::subcircuit::parse_subckt_instance;
pub use crate::elements::voltage_source::parse_voltage_source;
pub use crate::utils::parse_value;
pub use crate::utils::{alphanumeric_or_underscore1, parse_key_value, value_parser};
pub use nom::combinator::map;
pub use nom::{
    IResult, Parser,
    branch::alt,
    bytes::complete::{tag, tag_no_case},
    character::complete::{alphanumeric1, space1},
    combinator::{all_consuming, opt},
    sequence::preceded,
};
pub use std::collections::HashMap;
pub use std::str::FromStr;
