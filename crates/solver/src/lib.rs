use std::collections::HashMap;

use faer::Mat;
use parser::{
    element::{Element, Nodes},
    Netlist,
};
pub struct Solver {
    /// The netlist to solve
    pub netlist: Netlist,

    /// The nodes of the netlist
    pub nodes: Vec<String>,

    /// A map of node names to their index in the nodes vector
    pub node_map: HashMap<String, usize>,
}

impl Solver {
    pub fn new(netlist: Netlist) -> Self {
        let mut nodes = Vec::new();
        let mut node_map = HashMap::new();

        for element in &netlist.elements {
            for node in element.nodes() {
                if !nodes.contains(&node) {
                    nodes.push(node.clone());
                }
            }
        }

        let nodes_without_ground: Vec<String> =
            nodes.iter().filter(|node| **node != "0").cloned().collect();

        node_map.insert("0".to_string(), 0);
        for (index, node) in nodes_without_ground.iter().enumerate() {
            node_map.insert(node.to_string(), index + 1);
        }

        Self {
            netlist,
            nodes,
            node_map,
        }
    }

    // fn generate_matrix_a() {}

    pub fn generate_matrix_g(&self) -> Mat<f64> {
        let num_nodes = self.nodes.len() - 1;
        let mut matrix_g = Mat::<f64>::zeros(num_nodes, num_nodes);

        self.netlist
            .elements
            .iter()
            .filter_map(|element| {
                if let Element::Resistor(resistor) = element {
                    Some(resistor)
                } else {
                    None
                }
            })
            .for_each(|resistor| {
                let node1_index = self.node_map[&resistor.node1];
                let node2_index = self.node_map[&resistor.node2];
                let conductance = 1.0 / resistor.value;

                if node1_index != 0 {
                    matrix_g[(node1_index - 1, node1_index - 1)] += conductance;
                }

                if node2_index != 0 {
                    matrix_g[(node2_index - 1, node2_index - 1)] += conductance;
                }

                if node1_index != 0 && node2_index != 0 {
                    matrix_g[(node1_index - 1, node2_index - 1)] -= conductance;
                    matrix_g[(node2_index - 1, node1_index - 1)] -= conductance;
                }
            });

        matrix_g
    }

    fn generate_matrix_b(self) -> Mat<f64> {
        let number_of_voltage_sources = self
            .netlist
            .elements
            .iter()
            .filter_map(|element| {
                if let Element::VoltageSource(voltage_source) = element {
                    Some(voltage_source)
                } else {
                    None
                }
            })
            .count();

        let mut matrix_b = Mat::<f64>::zeros(number_of_voltage_sources, number_of_voltage_sources);

        for (index, element) in self.netlist.elements.iter().enumerate() {
            if let Element::VoltageSource(voltage_source) = element {
                let node1_index = self.node_map[&voltage_source.node1];
                let node2_index = self.node_map[&voltage_source.node2];

                if node1_index != 0 {
                    matrix_b[(index, node1_index - 1)] = 1.0;
                }

                if node2_index != 0 {
                    matrix_b[(index, node2_index - 1)] = -1.0;
                }
            }
        }

        matrix_b
    }

    // fn generate_matrix_c() {}

    // fn generate_matrix_d() {}
}
