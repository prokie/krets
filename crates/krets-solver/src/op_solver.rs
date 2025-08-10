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

        for iter in 0..self.config.maximum_iterations {
            let mut g_stamps = Vec::new();
            let mut e_stamps = Vec::new();

            elements
                .iter()
                .filter(|e| !matches!(e, Element::Capacitor(_)))
                .map(|e| {
                    (
                        e.conductance_matrix_dc_stamp(index_map, &previous_result),
                        e.excitation_vector_dc_stamp(index_map, &previous_result),
                    )
                })
                .for_each(|(g, e)| {
                    g_stamps.extend(g);
                    e_stamps.extend(e);
                });

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

            // The residue criterion for convergence
            let mut total_current = 0.0;
            let mut maximum_current = 0.0f64;
            for (key, value) in result.iter() {
                if key.starts_with("I") {
                    total_current += value;

                    maximum_current = maximum_current.max(value.abs());
                }
            }
            dbg!(
                total_current - self.config.relative_tolerance * maximum_current
                    + self.config.current_absolute_tolerance
            );

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
