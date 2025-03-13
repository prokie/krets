use crate::prelude::*;
use std::collections::HashMap;

use krets_matrix::{complex_matrix::ComplexMatrix, matrix::Matrix, mna_matrix::MnaMatrix};
use krets_parser::{analyses::DcAnalysis, circuit::Circuit, elements::Element};

pub struct Solver {
    circuit: Circuit,
}

impl Solver {
    pub const fn new(circuit: Circuit) -> Self {
        Self { circuit }
    }

    pub fn solve_dc(self, dc_analysis: DcAnalysis) -> Result<Vec<HashMap<String, f64>>> {
        let size = self.circuit.index_map.len();

        let mut mna_matrix = MnaMatrix {
            conductance_matrix: Matrix::new_empty(size, size),
            excitation_vector: Matrix::new_empty(size, 1),
            index_map: self.circuit.index_map,
            complex_conductance_matrix: ComplexMatrix::new_empty(size, size),
            complex_excitation_vector: ComplexMatrix::new_empty(size, 1),
        };

        let mut dc_sweep_element = self
            .circuit
            .elements
            .iter()
            .find(|x| x.identifier() == dc_analysis.element)
            .cloned()
            .ok_or_else(|| Error::ElementNotFound(dc_analysis.element.clone()))?;

        let mut results: Vec<HashMap<String, f64>> = vec![];

        for element in &self.circuit.elements {
            if matches!(element, Element::Capacitor(_)) {
                continue;
            }

            element.add_dc_stamp(&mut mna_matrix);
        }

        let mut current = dc_analysis.start;

        while current <= dc_analysis.stop {
            dc_sweep_element.set_value(current);
            dc_sweep_element.add_dc_stamp(&mut mna_matrix);
            current += dc_analysis.step_size;
            results.push(mna_matrix.solve())
        }

        Ok(results)
    }

    pub fn solve_op(self) -> HashMap<String, f64> {
        let size = self.circuit.index_map.len();

        let mut mna_matrix = MnaMatrix {
            conductance_matrix: Matrix::new_empty(size, size),
            excitation_vector: Matrix::new_empty(size, 1),
            index_map: self.circuit.index_map,
            complex_conductance_matrix: ComplexMatrix::new_empty(size, size),
            complex_excitation_vector: ComplexMatrix::new_empty(size, 1),
        };

        for element in self.circuit.elements {
            if matches!(element, Element::Capacitor(_)) {
                continue;
            }
            element.add_dc_stamp(&mut mna_matrix);
        }

        mna_matrix.solve()
    }

    pub fn solve_ac(self) -> HashMap<String, f64> {
        let size = self.circuit.index_map.len();

        let mut mna_matrix = MnaMatrix {
            conductance_matrix: Matrix::new_empty(size, size),
            excitation_vector: Matrix::new_empty(size, 1),
            index_map: self.circuit.index_map,
            complex_conductance_matrix: ComplexMatrix::new_empty(size, size),
            complex_excitation_vector: ComplexMatrix::new_empty(size, 1),
        };

        for element in self.circuit.elements {
            if matches!(element, Element::Capacitor(_)) {
                continue;
            }
            element.add_dc_stamp(&mut mna_matrix);
        }

        mna_matrix.solve()
    }
}
