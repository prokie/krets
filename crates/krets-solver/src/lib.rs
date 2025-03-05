pub mod error;

pub mod prelude;

use std::collections::HashMap;

use krets_matrix::{Matrix, mna_matrix::MnaMatrix};
use krets_parser::circuit::Circuit;

pub struct Solver {
    circuit: Circuit,
}

impl Solver {
    pub const fn new(circuit: Circuit) -> Self {
        Self { circuit }
    }

    pub fn solve(self) -> HashMap<String, f64> {
        let size = self.circuit.index_map.len();

        let mut mna_matrix = MnaMatrix {
            conductance_matrix: Matrix::new_empty(size, size),
            excitation_vector: Matrix::new_empty(size, 1),
            index_map: self.circuit.index_map,
        };

        for element in self.circuit.elements {
            element.add_stamp(&mut mna_matrix);
        }

        mna_matrix.solve()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    //     #[test]
    //     fn test_assemble_mna_system() {
    //         // This is taken from Figure 2.35 in the book.
    //         let circuit_description = "
    // V1 5 0 2
    // V2 3 2 0.2
    // V3 7 6 2
    // I1 4 8 1e-3
    // I2 0 6 1e-3
    // R1 1 5 1.5
    // R2 1 2 1
    // R3 5 2 50 G2 % this is a group 2 element
    // R4 5 6 0.1
    // R5 2 6 1.5
    // R6 3 4 0.1
    // R7 8 0 1e3
    // R8 4 0 10 G2 % this is a group 2 element
    // ";
    //         let circuit = parser::parse_circuit_description(circuit_description).unwrap();
    //         let solver = Solver::new(circuit);
    //         solver.assemble_mna_system();
    //     }

    //     #[test]
    //     fn test_case_1() {
    //         // This is taken from website.
    //         let circuit_description = "
    // V1 2 1 32
    // R1 1 0 2
    // R2 2 3 4
    // R3 2 0 8
    // V2 3 0 20
    // ";
    //         let circuit = parser::parse_circuit_description(circuit_description).unwrap();
    //         let solver = Solver::new(circuit);
    //         solver.assemble_mna_system();
    //     }

    #[test]
    fn test_voltage_divider() {
        let circuit_description = "
V1 in 0 1
R1 in out 1000
R2 out 0 2000
";
        let circuit = krets_parser::parse_circuit_description(circuit_description).unwrap();
        let solver = Solver::new(circuit);
        let solution = solver.solve();

        let v_in = solution.get("V(in)").unwrap();
        let v_out = solution.get("V(out)").unwrap();
        let i_v1 = solution.get("I(V1)").unwrap();

        assert!((v_in - 1.0).abs() < 1e-3);
        assert!((v_out - 0.6667).abs() < 1e-3);
        assert!((i_v1 - 1. / 3000.).abs() < 1e-3);
    }
}
