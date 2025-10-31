use crate::prelude::*;
pub struct Subcircuit {
    pub name: String,
    pub pins: Vec<String>,
    pub elements: Vec<Element>,
}

impl Subcircuit {
    pub fn new(name: impl Into<String>, pins: impl IntoIterator<Item = impl Into<String>>) -> Self {
        Self {
            name: name.into(),
            pins: pins.into_iter().map(Into::into).collect(),
            elements: Vec::new(),
        }
    }
}
