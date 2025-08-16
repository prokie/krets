use std::collections::HashMap;

use crate::config::SolverConfig;
use crate::prelude::*;
use crate::solver::{convergence_check, sum_triplets};
use faer::Mat;
use faer::prelude::Solve;
use faer::sparse::{SparseColMat, Triplet};
use krets_parser::circuit::Circuit;
use krets_parser::elements::Element;

pub struct DcSolver {
    pub config: SolverConfig,
    pub circuit: Circuit,
}

impl DcSolver {
    pub fn new(circuit: Circuit, config: SolverConfig) -> Self {
        DcSolver { circuit, config }
    }

    pub fn solve(&self) -> Result<Vec<HashMap<String, f64>>> {
        let index_map = &self.circuit.index_map;
        let size = index_map.len();

        // Remove capacitors from elements since they are not included in DC analysis.
        let elements = &self
            .circuit
            .elements
            .iter()
            .filter(|e| !matches!(e, Element::Capacitor(_)))
            .collect::<Vec<_>>();

        let non_linear_elements = &elements
            .iter()
            .filter(|e| e.is_nonlinear())
            .collect::<Vec<_>>();

        let mut result = HashMap::new();
        let mut previous_result = HashMap::new();

        let mut g_stamps = Vec::new();
        let mut e_stamps = Vec::new();

        let mut results: Vec<HashMap<String, f64>> = vec![];

        for element in elements {
            g_stamps.extend(element.add_conductance_matrix_dc_stamp(index_map, &previous_result));
            e_stamps.extend(element.add_excitation_vector_dc_stamp(index_map, &previous_result));
        }

        for iter in 0..self.config.maximum_iterations {
            for nonlinear_element in non_linear_elements {
                // Subtract previous stamp
                g_stamps.extend(
                    nonlinear_element.undo_conductance_matrix_dc_stamp(index_map, &previous_result),
                );
                e_stamps.extend(
                    nonlinear_element.undo_excitation_vector_dc_stamp(index_map, &previous_result),
                );

                // Add new stamp
                g_stamps
                    .extend(nonlinear_element.add_conductance_matrix_dc_stamp(index_map, &result));
                e_stamps
                    .extend(nonlinear_element.add_excitation_vector_dc_stamp(index_map, &result));
            }

            let g_stamps = sum_triplets(&g_stamps);
            let e_stamps = sum_triplets(&e_stamps);

            let lu = SparseColMat::try_new_from_triplets(size, size, &g_stamps)
                .expect("Failed to build sparse matrix")
                .sp_lu()
                .expect("LU decomposition failed");

            let mut b = Mat::zeros(size, 1);
            for &Triplet { row, col, val } in &e_stamps {
                b[(row, col)] = val;
            }

            let x = lu.solve(&b);

            result = index_map
                .iter()
                .map(|(node, &idx)| (node.clone(), x[(idx, 0)]))
                .collect();

            if non_linear_elements.is_empty() {
                // If there are no nonlinear elements, we can exit early.
                break;
            }

            if convergence_check(&previous_result, &result, &self.config) {
                println!("Converged after {} iterations", iter + 1);
                break;
            }

            // Move current result to previous_result for next iteration
            previous_result = std::mem::take(&mut result);

            if iter == self.config.maximum_iterations - 1 {
                println!("Warning: Maximum iterations reached without convergence.");

                return Err(Error::MaximumIterationsExceeded(
                    self.config.maximum_iterations,
                ));
            }
        }

        Ok(results)
    }
}
