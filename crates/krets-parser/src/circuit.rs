use crate::prelude::*;

use crate::models::Model;
use crate::subcircuit::SubcircuitInstance;

#[derive(Debug, Clone)]
/// A structured representation of a circuit element.
pub struct Circuit {
    /// A list of circuit elements.
    pub elements: Vec<Element>,

    /// A hashmap mapping nodes/elements to the MNA Matrix.
    pub index_map: HashMap<String, usize>,

    /// A list of nodes in the circuit.
    pub nodes: Vec<String>,

    /// A list of models in the circuit.
    pub models: HashMap<String, Model>,

    /// A list of subcircuits in the circuit.
    pub subcircuit_definitions: HashMap<String, Subcircuit>,

    /// A list of subcircuit instances in the circuit.
    pub subcircuit_instances: Vec<SubcircuitInstance>,
}

impl Circuit {
    /// Create a new circuit
    pub fn new(
        elements: Vec<Element>,
        index_map: HashMap<String, usize>,
        nodes: Vec<String>,
        models: HashMap<String, Model>,
        subcircuit_definitions: HashMap<String, Subcircuit>,
        subcircuit_instances: Vec<SubcircuitInstance>,
    ) -> Self {
        Circuit {
            elements,
            index_map,
            nodes,
            models,
            subcircuit_definitions,
            subcircuit_instances,
        }
    }

    pub fn empty_circuit() -> Self {
        Circuit {
            elements: Vec::new(),
            index_map: HashMap::new(),
            nodes: Vec::new(),
            models: HashMap::new(),
            subcircuit_definitions: HashMap::new(),
            subcircuit_instances: Vec::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.elements.is_empty()
            && self.subcircuit_definitions.is_empty()
            && self.subcircuit_instances.is_empty()
    }
}
