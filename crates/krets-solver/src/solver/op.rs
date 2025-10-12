use std::collections::HashMap;

use crate::prelude::*;
use faer::{
    Mat,
    prelude::Solve,
    sparse::{SparseColMat, Triplet},
};
use krets_parser::{
    circuit::Circuit,
    elements::{Element, Stampable},
};

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
        // --- Rebuild MNA matrices in each iteration ---
        // This is the core of the Newton-Raphson method. The Jacobian (g_stamps)
        // and the RHS vector (e_stamps) are recalculated based on the solution from
        // the previous iteration (`previous_result`).
        let mut g_stamps = Vec::new();
        let mut e_stamps = Vec::new();

        for element in &elements {
            g_stamps.extend(element.add_conductance_matrix_dc_stamp(index_map, &previous_result));
            e_stamps.extend(element.add_excitation_vector_dc_stamp(index_map, &previous_result));
        }

        let g_stamps_summed = sum_triplets(&g_stamps);
        let e_stamps_summed = sum_triplets(&e_stamps);

        let lu = SparseColMat::try_new_from_triplets(size, size, &g_stamps_summed)
            .expect("Failed to build sparse matrix")
            .sp_lu()
            .expect("LU decomposition failed");

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
            println!("Converged after {} iterations", iter + 1);
            break;
        }

        // Move current result to previous_result for the next iteration.
        previous_result = result.clone();

        if iter == config.maximum_iterations - 1 {
            println!("Warning: Maximum iterations reached without convergence.");
            return Err(Error::MaximumIterationsExceeded(config.maximum_iterations));
        }
    }

    // Return the final converged operating point solution.
    Ok(result)
}

/// A helper function to pretty-print the MNA matrix for debugging purposes.
#[allow(dead_code)] // Prevents warnings if not used (e.g., in release builds).
fn print_matrix(
    triplets: &[Triplet<usize, usize, f64>],
    size: usize,
    index_map: &HashMap<String, usize>,
) {
    // Create a reverse mapping from index to name for easier lookup of headers.
    let mut rev_index_map: Vec<String> = vec![String::new(); size];
    for (name, &idx) in index_map {
        if idx < size {
            rev_index_map[idx] = name.clone();
        }
    }

    // Convert triplets to a HashMap for efficient (row, col) -> value lookups.
    let matrix_map: HashMap<(usize, usize), f64> = triplets
        .iter()
        .map(|&Triplet { row, col, val }| ((row, col), val))
        .collect();

    // Print header row with column names.
    print!("{:<12}", ""); // Spacer for row names column.
    for i in 0..size {
        print!("{:<12}", rev_index_map[i]);
    }
    println!();
    println!("{}", "-".repeat(12 * (size + 1)));

    // Print each row with its name and values.
    for r in 0..size {
        print!("{:<12}", rev_index_map[r]);
        for c in 0..size {
            let val = matrix_map.get(&(r, c)).unwrap_or(&0.0);
            print!("{:<12.4}", val);
        }
        println!();
    }
}
