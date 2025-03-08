use std::collections::HashMap;

use crate::elements::Element;

#[derive(Debug, Clone)]
/// A structured representation of a circuit element.
pub struct Circuit {
    /// A list of circuit elements.
    pub elements: Vec<Element>,

    /// A hashmap mapping nodes/elements to the MNA Matrix.
    pub index_map: HashMap<String, usize>,

    /// A list of nodes in the circuit.
    pub nodes: Vec<String>,
}

impl Circuit {
    /// Create a new circuit
    pub fn new(
        elements: Vec<Element>,
        index_map: HashMap<String, usize>,
        nodes: Vec<String>,
    ) -> Self {
        Circuit {
            elements,
            index_map,
            nodes,
        }
    }
}
