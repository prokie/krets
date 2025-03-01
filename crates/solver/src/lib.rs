pub mod error;
pub mod matrix;
pub mod prelude;
use crate::prelude::*;
use faer::Mat;
use parser::circuit::Circuit;

pub struct Solver {
    circuit: Circuit,
}

impl Solver {
    pub fn new(circuit: Circuit) -> Self {
        Self { circuit }
    }

    /// Generates the incidence matrix of the circuit.
    ///
    /// The incidence matrix is an `n × m` matrix `M`, where:
    /// - `n` is the number of nodes in the circuit.
    /// - `m` is the number of circuit elements (edges).
    ///
    /// Each column corresponds to a circuit element, with:
    /// - `+1` if the element's tail is connected to the node.
    /// - `-1` if the element's head is connected to the node.
    /// - `0` otherwise.
    ///
    /// ### Properties:
    /// 1. Each column has exactly one `+1` and one `-1`, all other entries are `0`.
    /// 2. The sum of all rows is the zero vector, meaning rows are not linearly independent.
    ///
    /// # Returns
    /// A `Mat<f64>` representing the incidence matrix of the circuit.
    ///
    /// # Errors
    /// - Returns an error if a node name is not a non-negative integer.
    ///
    /// # Panics
    /// - Panics if the sum of all rows is not the zero vector.
    pub fn incident_matrix(self) -> Result<Mat<f64>> {
        let number_of_nodes = self.circuit.nodes.len();
        let number_of_elements = self.circuit.elements.len();

        let mut incident_matrix = Mat::<f64>::zeros(number_of_nodes, number_of_elements);

        for (index, element) in self.circuit.elements.iter().enumerate() {
            let nodes = element.nodes();
            let node_plus = nodes[0]
                .parse::<usize>()
                .map_err(|e| error::Error::InvalidUsizeValue(e.to_string()))?;
            let node_minus = nodes[1]
                .parse::<usize>()
                .map_err(|e| error::Error::InvalidUsizeValue(e.to_string()))?;

            incident_matrix[(node_plus, index)] = 1.0;
            incident_matrix[(node_minus, index)] = -1.0;
        }

        let sum: f64 = incident_matrix.row_iter().map(|row| row.sum()).sum();
        assert_eq!(sum, 0.0);

        Ok(incident_matrix)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_incident_matrix() {
        // This is taken from Figure 2.12 in the book.
        let circuit_description = "
R1 0 1 1000
R2 1 3 2000
R3 1 2 3000
R4 0 2 4000
R5 3 0 5000
R6 3 0 6000
        ";
        let circuit = parser::parse_circuit_description(circuit_description).unwrap();
        let solver = Solver::new(circuit);
        solver.incident_matrix().unwrap();
    }
}
