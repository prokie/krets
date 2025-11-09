use crate::{prelude::*, stampable::Stampable};
use faer::{
    Mat,
    prelude::Solve,
    sparse::{SparseColMat, Triplet},
};
use krets_parser::{circuit::Circuit, elements::Element};
use log::info;
use std::collections::HashMap;

/// Solves for the DC operating point of the circuit.
///
/// This function implements the Newton-Raphson iterative method to find the DC steady-state
/// solution for a potentially non-linear circuit.
pub fn solve(circuit: &Circuit, config: &SolverConfig) -> Result<HashMap<String, f64>> {
    let index_map = &circuit.index_map;
    let size = index_map.len();

    // Capacitors act as open circuits in DC analysis and can be filtered out.
    let elements: Vec<&Element> = circuit
        .elements
        .iter()
        .filter(|e| !matches!(e, Element::Capacitor(_)))
        .collect();

    // Check if the circuit contains any non-linear elements. If not, the solver
    // only needs to run for one iteration.
    let has_nonlinear_elements = elements.iter().any(|e| e.is_nonlinear());

    let mut result = HashMap::new();
    let mut previous_result = HashMap::new();

    for iter in 0..config.maximum_iterations {
        // This is the core of the Newton-Raphson method. The Jacobian (g_stamps)
        // and the RHS vector (e_stamps) are recalculated based on the solution from
        // the previous iteration (`previous_result`).
        let mut g_stamps = Vec::new();
        let mut e_stamps = Vec::new();

        for element in &elements {
            g_stamps.extend(element.stamp_conductance_matrix_dc(index_map, &previous_result));
            e_stamps.extend(element.stamp_excitation_vector_dc(index_map, &previous_result));
        }

        let g_stamps_summed = sum_triplets(&g_stamps);
        let e_stamps_summed = sum_triplets(&e_stamps);

        let lu = SparseColMat::try_new_from_triplets(size, size, &g_stamps_summed)
            .map_err(|_| Error::MatrixBuild)?
            .sp_lu()
            .map_err(|_| Error::MatrixDecomposition)?;

        let mut b = Mat::zeros(size, 1);
        for &Triplet { row, col, val } in &e_stamps_summed {
            b[(row, col)] = val;
        }

        let x = lu.solve(&b);

        result = index_map
            .iter()
            .map(|(node, &idx)| (node.clone(), x[(idx, 0)]))
            .collect();

        // For purely linear circuits, we only need one iteration.
        if !has_nonlinear_elements {
            break;
        }

        if convergence_check(&previous_result, &result, config) {
            info!("Converged after {} iterations", iter + 1);
            break;
        }

        // Move current result to previous_result for the next iteration.
        previous_result.clone_from(&result);

        if iter == config.maximum_iterations - 1 {
            info!("Warning: Maximum iterations reached without convergence.");
            return Err(Error::MaximumIterationsExceeded(config.maximum_iterations));
        }
    }

    // Return the final converged operating point solution.
    Ok(result)
}
