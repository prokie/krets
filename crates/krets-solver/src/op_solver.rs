use std::collections::HashMap;

use faer::{
    Mat,
    prelude::Solve,
    sparse::{SparseColMat, Triplet},
};
use krets_parser::{circuit::Circuit, elements::Element};

use crate::prelude::*;
use crate::{config::SolverConfig, solver::sum_triplets};
pub struct OpSolver {
    pub config: SolverConfig,
    pub circuit: Circuit,
}

impl OpSolver {
    pub fn new(circuit: Circuit, config: SolverConfig) -> Self {
        OpSolver { circuit, config }
    }

    pub fn solve(&self) -> Result<HashMap<String, f64>> {
        let index_map = &self.circuit.index_map;
        let size = index_map.len();
        let elements = &self.circuit.elements;
        let mut result = HashMap::new();
        let mut previous_result = result.clone();

        let mut g_stamps = Vec::new();
        let mut e_stamps = Vec::new();

        for element in elements {
            if !matches!(element, Element::Capacitor(_)) {
                g_stamps
                    .extend(element.add_conductance_matrix_dc_stamp(index_map, &previous_result));
                e_stamps
                    .extend(element.add_excitation_vector_dc_stamp(index_map, &previous_result));
            }
        }

        for iter in 0..self.config.maximum_iterations {
            for nonlinear_element in elements.iter().filter(|e| matches!(e, Element::Diode(_))) {
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

            if !previous_result.is_empty()
                && result
                    .iter()
                    .all(|(node, &value)| (value - previous_result[node]).abs() < 1e-12)
            {
                println!("Converged after {} iterations", iter + 1);
                break;
            }
            previous_result = result.clone();

            if iter == self.config.maximum_iterations - 1 {
                println!("Warning: Maximum iterations reached without convergence.");
            }
        }

        Ok(result)
    }
}
