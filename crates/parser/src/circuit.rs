use std::collections::HashSet;

use crate::elements::Element;

#[derive(Debug)]
/// A structured representation of a circuit element.
pub struct Circuit {
    /// A list of circuit elements.
    pub elements: Vec<Element>,

    /// A list of nodes in the circuit.
    pub nodes: Vec<String>,
}

impl Circuit {
    /// Create a new circuit
    pub fn new(elements: Vec<Element>) -> Self {
        // Collect all nodes from the elements into a HashSet to remove duplicates.
        let nodes: HashSet<String> = elements
            .iter()
            .flat_map(|x| x.nodes())
            .map(|s| s.to_string())
            .collect();

        Circuit {
            elements,
            nodes: nodes.into_iter().collect(),
        }
    }
}
