use std::collections::HashMap;

use crate::prelude::*;
use faer::{
    Mat,
    prelude::Solve,
    sparse::{SparseColMat, Triplet},
};
use krets_parser::{
    analyses::DcAnalysis,
    circuit::Circuit,
    elements::{Element, Identifiable, Stampable},
};

/// Solves for the DC response of a circuit while sweeping a source.
///
/// This function performs a DC sweep analysis by repeatedly solving for the circuit's
/// operating point at each step of the sweep.
pub fn solve(
    circuit: &mut Circuit,
    config: &SolverConfig,
    dc_analysis: DcAnalysis,
) -> Result<Vec<HashMap<String, f64>>> {
    let index_map = &circuit.index_map;
    let size = index_map.len();

    // Find the index of the element to be swept. This is faster than finding the element by name in every loop.
    let sweep_element_index = circuit
        .elements
        .iter()
        .position(|x| x.identifier() == dc_analysis.element)
        .ok_or_else(|| Error::ElementNotFound(dc_analysis.element.clone()))?;

    // Store the original value of the swept element to restore it after the analysis.
    let original_value = match &circuit.elements[sweep_element_index] {
        Element::VoltageSource(vs) => vs.dc_value,
        Element::CurrentSource(is) => is.value,
        _ => {
            return Err(Error::InvalidElementFormat(
                "DC sweep element must be a voltage or current source".to_string(),
            ));
        }
    };

    let mut all_results = Vec::new();
    let mut last_op_solution = HashMap::new(); // Use last solution as a "warm start" for the next step

    // --- Outer loop for the DC sweep ---
    // Use an integer-based loop to avoid floating-point precision issues.
    let num_steps =
        ((dc_analysis.stop - dc_analysis.start) / dc_analysis.step_size).abs() as usize + 1;

    for i in 0..num_steps {
        let current_sweep_val = dc_analysis.start + (i as f64 * dc_analysis.step_size);

        // Update the value of the sweep element for the current step.
        match &mut circuit.elements[sweep_element_index] {
            Element::VoltageSource(vs) => vs.dc_value = current_sweep_val,
            Element::CurrentSource(is) => is.value = current_sweep_val,
            _ => unreachable!(),
        }

        // --- Inner Newton-Raphson loop to solve the OP for this sweep step ---
        let mut op_result = HashMap::new();
        let mut previous_op_result = last_op_solution.clone(); // Warm start from previous sweep point

        let elements: Vec<&Element> = circuit
            .elements
            .iter()
            .filter(|e| !matches!(e, Element::Capacitor(_)))
            .collect();
        let has_nonlinear_elements = elements.iter().any(|e| e.is_nonlinear());

        for iter in 0..config.maximum_iterations {
            let mut g_stamps = Vec::new();
            let mut e_stamps = Vec::new();

            for element in &elements {
                g_stamps.extend(
                    element.add_conductance_matrix_dc_stamp(index_map, &previous_op_result),
                );
                e_stamps
                    .extend(element.add_excitation_vector_dc_stamp(index_map, &previous_op_result));
            }

            let g_stamps_summed = sum_triplets(&g_stamps);
            let e_stamps_summed = sum_triplets(&e_stamps);

            // FIX: Use `.map_err()` to convert the LU decomposition error.
            let lu = SparseColMat::try_new_from_triplets(size, size, &g_stamps_summed)
                .map_err(|e| Error::Unexpected(e.to_string()))?
                .sp_lu()
                .map_err(|_| Error::DecompositionFailed)?;

            let mut b = Mat::zeros(size, 1);
            for &Triplet { row, col, val } in &e_stamps_summed {
                b[(row, col)] = val;
            }
            let x = lu.solve(&b);

            op_result = index_map
                .iter()
                .map(|(node, &idx)| (node.clone(), x[(idx, 0)]))
                .collect();

            if !has_nonlinear_elements {
                break; // Circuit is linear, one iteration is enough.
            }
            if convergence_check(&previous_op_result, &op_result, config) {
                break; // Converged for this sweep point.
            }
            previous_op_result = op_result.clone();

            if iter == config.maximum_iterations - 1 {
                return Err(Error::MaximumIterationsExceeded(config.maximum_iterations));
            }
        }

        last_op_solution = op_result.clone();
        all_results.push(op_result);
    }

    // Restore the original value of the swept element.
    match &mut circuit.elements[sweep_element_index] {
        Element::VoltageSource(vs) => vs.dc_value = original_value,
        Element::CurrentSource(is) => is.value = original_value,
        _ => unreachable!(),
    }

    Ok(all_results)
}
