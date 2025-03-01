use crate::elements::Element;

#[derive(Debug)]
/// A structured representation of a circuit element.
pub struct Netlist {
    /// A list of circuit elements.
    pub elements: Vec<Element>,
}
