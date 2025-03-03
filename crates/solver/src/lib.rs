pub mod error;
pub mod mna_matrix;
pub mod prelude;
use crate::prelude::*;
use faer::Mat;
use matrix::Matrix;
use parser::circuit::Circuit;

pub struct Solver {
    circuit: Circuit,
}

impl Solver {
    pub fn new(circuit: Circuit) -> Self {
        Self { circuit }
    }

    pub fn assemble_mna_system(self) {
        println!("Assembling MNA system...");
        let number_of_nodes = self.circuit.nodes.len();
        let g2_elements = self.circuit.get_g2_elements();

        // The size of the MNA matrix is the number of nodes plus the number of group 2 elements
        // excluding ground node.
        let size = number_of_nodes + g2_elements.len() - 1;
        let mut x_nodes: Vec<String> = self
            .circuit
            .node_map
            .keys()
            .map(|key| format!("V({key})"))
            .collect();
        x_nodes.sort();
        x_nodes.extend(g2_elements.iter().map(|key| format!("I({key})")));

        let mut matrix_a = Matrix::new_empty(size, size);
        let mut matrix_b = Matrix::new_empty(size, 1);

        // let mut matrix = MNAMatrix {
        //     a: SparseColMat::<usize, f64>::new(size, size),
        //     z: Mat::<f64>::zeros(size, 1),
        //     nodes: x_nodes,
        // };

        for current_source in self.circuit.get_current_sources() {
            let index_plus = self.circuit.node_map.get(&current_source.plus);
            let index_minus = self.circuit.node_map.get(&current_source.minus);

            if let Some(&index_plus) = index_plus {
                matrix_b[(index_plus, 0)] -= current_source.stamp();
            }
            if let Some(&index_minus) = index_minus {
                matrix_b[(index_minus, 0)] += current_source.stamp();
            }
        }

        for resistor in self.circuit.get_g1_resistors() {
            let index_plus = self.circuit.node_map.get(&resistor.plus);
            let index_minus = self.circuit.node_map.get(&resistor.minus);

            if let Some(&index_plus) = index_plus {
                matrix_a[(index_plus, index_plus)] += 1. / resistor.value;
            }

            if let Some(&index_minus) = index_minus {
                matrix_a[(index_minus, index_minus)] += 1. / resistor.value;
            }

            if let (Some(&index_plus), Some(&index_minus)) = (index_plus, index_minus) {
                matrix_a[(index_plus, index_minus)] -= 1. / resistor.value;
                matrix_a[(index_minus, index_plus)] -= 1. / resistor.value;
            }
        }

        for (offset, voltage_source) in self.circuit.get_voltage_sources().iter().enumerate() {
            let index_plus = self.circuit.node_map.get(&voltage_source.plus);
            let index_minus = self.circuit.node_map.get(&voltage_source.minus);

            if let Some(&index_plus) = index_plus {
                matrix_a[(index_plus, offset + number_of_nodes - 1)] += 1.0;
                matrix_a[(offset + number_of_nodes - 1, index_plus)] += 1.0;

                matrix_b[(offset + number_of_nodes - 1, 0)] = voltage_source.value;
            }

            if let Some(&index_minus) = index_minus {
                matrix_a[(index_minus, offset + number_of_nodes - 1)] -= 1.0;
                matrix_a[(offset + number_of_nodes - 1, index_minus)] -= 1.0;
            }
        }
        println!("{matrix_a}");
        println!("{matrix_b}");

        let lu = matrix_a.to_sparse_col_mat().sp_lu().unwrap();
        let x = faer::linalg::solvers::Solve::solve(&lu, &matrix_b.to_dense_mat());
        println!("Solution x = {x:?}");
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

    #[test]
    fn test_assemble_mna_system() {
        // This is taken from Figure 2.35 in the book.
        let circuit_description = "
V1 5 0 2
V2 3 2 0.2
V3 7 6 2
I1 4 8 1e-3
I2 0 6 1e-3
R1 1 5 1.5
R2 1 2 1
R3 5 2 50 G2 % this is a group 2 element
R4 5 6 0.1
R5 2 6 1.5
R6 3 4 0.1
R7 8 0 1e3
R8 4 0 10 G2 % this is a group 2 element
";
        let circuit = parser::parse_circuit_description(circuit_description).unwrap();
        let solver = Solver::new(circuit);
        solver.assemble_mna_system();
    }

    #[test]
    fn test_case_1() {
        // This is taken from website.
        let circuit_description = "
V1 2 1 32
R1 1 0 2
R2 2 3 4
R3 2 0 8
V2 3 0 20
";
        let circuit = parser::parse_circuit_description(circuit_description).unwrap();
        let solver = Solver::new(circuit);
        solver.assemble_mna_system();
    }

    #[test]
    fn test_voltage_divider() {
        let circuit_description = "
V1 in 0 1
R1 in out 1000
R2 out 0 2000
";
        let circuit = parser::parse_circuit_description(circuit_description).unwrap();
        let solver = Solver::new(circuit);
        solver.assemble_mna_system();
    }
}
