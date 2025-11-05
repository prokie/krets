use log::info;
use std::collections::HashMap;

use crate::{config::SolverConfig, prelude::*, solver::op};
use faer::{
    Mat, c64,
    prelude::Solve,
    sparse::{SparseColMat, Triplet},
};
use krets_parser::{analyses::AcAnalysis, circuit::Circuit, elements::Stampable};

/// Solves for the small-signal AC response of the circuit at a given frequency.
///
/// This function first calculates the DC operating point to determine the linearized models
/// for non-linear components. It then constructs and solves the complex-valued MNA
/// system for the specified frequency.
pub fn solve(
    circuit: &Circuit,
    config: &SolverConfig,
    parameters: &AcAnalysis,
) -> Result<Vec<HashMap<String, c64>>> {
    // Changed return type
    // First, find the DC operating point. This is crucial for linearizing non-linear components.
    info!("Calculating DC operating point for AC analysis...");
    let dc_solution = op::solve(circuit, config)?;
    info!("DC operating point calculated.");

    let index_map = &circuit.index_map;
    let size = index_map.len();
    let mut all_results = Vec::new(); // Store results for each frequency

    // --- Frequency Sweep Logic ---
    let frequencies = parameters.clone().generate_frequencies();
    info!(
        "Starting AC sweep over {} frequencies...",
        frequencies.len()
    );

    for frequency in frequencies {
        if frequency <= 0.0 {
            // Skip non-positive frequencies as they are physically meaningless
            // and can cause issues (e.g., divide by zero in impedance calculations).
            info!("Skipping non-positive frequency: {frequency}");
            continue;
        }
        // Recalculate stamps for the current frequency
        let mut g_stamps = Vec::new();
        let mut e_stamps = Vec::new();

        for element in &circuit.elements {
            g_stamps.extend(element.stamp_conductance_matrix_ac(
                index_map,
                &dc_solution,
                frequency, // Use current frequency
            ));
            e_stamps.extend(element.stamp_excitation_vector_ac(
                index_map,
                &dc_solution,
                frequency, // Use current frequency
            ));
        }

        let g_stamps_summed = sum_triplets(&g_stamps);
        let e_stamps_summed = sum_triplets(&e_stamps);

        // --- Solve MNA System for current frequency ---
        let g_mat = SparseColMat::try_new_from_triplets(size, size, &g_stamps_summed)
            .map_err(|e| Error::Unexpected(format!("Matrix build failed at f={frequency}: {e}")))?;

        let lu = g_mat.sp_lu().map_err(|_| Error::DecompositionFailed)?;

        let mut b = Mat::zeros(size, 1); // Use complex matrix
        for &Triplet { row, col, val } in &e_stamps_summed {
            // Ensure indices are within bounds
            if row < size && col < 1 {
                b[(row, col)] = val;
            } else {
                // Log or handle the error appropriately
                info!(
                    "Warning: Out-of-bounds triplet indices ignored: row={row}, col={col} for size={size}"
                );
            }
        }

        // Make sure b has the correct dimensions before solving
        if b.nrows() != size || b.ncols() != 1 {
            return Err(Error::Unexpected(format!(
                "Excitation vector b has incorrect dimensions: {}x{} (expected {}x1)",
                b.nrows(),
                b.ncols(),
                size
            )));
        }

        let x = lu.solve(&b);

        let mut solution_map: HashMap<String, c64> = index_map
            .iter()
            .map(|(node, &idx)| (node.clone(), x[(idx, 0)]))
            .collect();

        // Include the current frequency in the results for this step.
        solution_map.insert("frequency".to_string(), c64::new(frequency, 0.0));

        all_results.push(solution_map); // Add results for this frequency
        // info!("Solved for f = {} Hz", frequency);
    }
    Ok(all_results) // Return the collected results
}
