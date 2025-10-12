use std::collections::HashMap;

use crate::{config::SolverConfig, prelude::*, solver::op};
use faer::{
    Mat, c64,
    prelude::Solve,
    sparse::{SparseColMat, Triplet},
};
use krets_parser::{circuit::Circuit, elements::Stampable};

/// Solves for the small-signal AC response of the circuit at a given frequency.
///
/// This function first calculates the DC operating point to determine the linearized models
/// for non-linear components. It then constructs and solves the complex-valued MNA
/// system for the specified frequency.
pub fn solve(
    circuit: &Circuit,
    config: &SolverConfig,
    frequency: f64,
) -> Result<HashMap<String, c64>> {
    // First, find the DC operating point. This is crucial for linearizing non-linear components.
    let dc_solution = op::solve(circuit, config)?;

    let index_map = &circuit.index_map;
    let size = index_map.len();
    let mut g_stamps = Vec::new();
    let mut e_stamps = Vec::new();

    for element in &circuit.elements {
        g_stamps.extend(element.add_conductance_matrix_ac_stamp(
            index_map,
            &dc_solution,
            frequency,
        ));
        e_stamps.extend(element.add_excitation_vector_ac_stamp(index_map, &dc_solution, frequency));
    }

    let g_stamps_summed = sum_triplets(&g_stamps);
    let e_stamps_summed = sum_triplets(&e_stamps);

    let lu = SparseColMat::try_new_from_triplets(size, size, &g_stamps_summed)
        .map_err(|e| Error::Unexpected(e.to_string()))?
        .sp_lu()
        .map_err(|_| Error::DecompositionFailed)?;

    let mut b = Mat::zeros(size, 1);
    for &Triplet { row, col, val } in &e_stamps_summed {
        b[(row, col)] = val;
    }
    let x = lu.solve(&b);

    let mut solution_map: HashMap<String, c64> = index_map
        .iter()
        .map(|(node, &index)| (node.clone(), x[(index, 0)]))
        .collect();

    // Include the frequency in the results for context.
    solution_map.insert("frequency".to_string(), c64::new(frequency, 0.0));

    Ok(solution_map)
}
