use std::collections::HashMap;

use crate::elements::{
    Element, current_source::CurrentSource, resistor::Resistor, voltage_source::VoltageSource,
};

#[derive(Debug)]
/// A structured representation of a circuit element.
pub struct Circuit {
    /// A list of circuit elements.
    pub elements: Vec<Element>,

    /// A hashmap mapping node names to node indexes.
    pub node_map: HashMap<String, usize>,

    /// A list of nodes in the circuit.
    pub nodes: Vec<String>,
}

impl Circuit {
    /// Create a new circuit
    pub fn new(
        elements: Vec<Element>,
        node_map: HashMap<String, usize>,
        nodes: Vec<String>,
    ) -> Self {
        Circuit {
            elements,
            node_map,
            nodes,
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

    pub fn get_g1_resistors(&self) -> Vec<&Resistor> {
        self.elements
            .iter()
            .filter_map(|e| {
                if let Element::Resistor(r) = e {
                    if !r.g2 {
                        return Some(r);
                    }
                }
                None
            })
            .collect()
    }

    pub fn get_g2_resistors(&self) -> Vec<&Resistor> {
        self.elements
            .iter()
            .filter_map(|e| {
                if let Element::Resistor(r) = e {
                    if r.g2 {
                        return Some(r);
                    }
                }
                None
            })
            .collect()
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
