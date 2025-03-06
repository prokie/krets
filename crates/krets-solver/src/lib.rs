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
