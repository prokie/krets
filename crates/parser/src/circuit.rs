use std::collections::HashSet;

use crate::elements::{
    Element, current_source::CurrentSource, resistor::Resistor, voltage_source::VoltageSource,
};

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

    pub fn get_g2_elements(&self) -> Vec<&Element> {
        self.elements.iter().filter(|e| e.is_g2()).collect()
    }

    pub fn get_resistors(&self) -> Vec<&Resistor> {
        let resistors: Vec<&Resistor> = self
            .elements
            .iter()
            .filter_map(|e| {
                if let Element::Resistor(r) = e {
                    Some(r)
                } else {
                    None
                }
            })
            .collect();
        resistors
    }

    pub fn get_voltage_sources(&self) -> Vec<&VoltageSource> {
        let voltage_sources: Vec<&VoltageSource> = self
            .elements
            .iter()
            .filter_map(|e| {
                if let Element::VoltageSource(v) = e {
                    Some(v)
                } else {
                    None
                }
            })
            .collect();
        voltage_sources
    }

    pub fn get_current_sources(&self) -> Vec<&CurrentSource> {
        let current_sources: Vec<&CurrentSource> = self
            .elements
            .iter()
            .filter_map(|e| {
                if let Element::CurrentSource(c) = e {
                    Some(c)
                } else {
                    None
                }
            })
            .collect();
        current_sources
    }
}
