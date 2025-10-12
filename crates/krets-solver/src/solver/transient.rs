use std::collections::HashMap;

use super::{convergence_check, sum_triplets};
use crate::{config::SolverConfig, prelude::*, solver::op};
use faer::{
    Mat,
    prelude::Solve,
    sparse::{SparseColMat, Triplet},
};
use krets_parser::{analyses::TransientAnalysis, circuit::Circuit, elements::Stampable};

/// Solves for the transient (time-domain) response of a circuit using a fixed time step.
///
/// This function follows the overall simulation flow described in Figure 5.25 of "Circuit Simulation" by Farid N. Najm.
///
/// # Algorithm
/// 1.  **Initial Condition:** It first solves for the DC operating point at t=0.
/// 2.  **Time-Stepping Loop:** It then iterates from t=0 to the specified stop time.
/// 3.  **Newton-Raphson at Each Time Step:** At each point in time, it uses an inner Newton-Raphson
///     loop to solve the non-linear system of algebraic equations that results from discretizing
///     the dynamic elements (capacitors and inductors).
///
/// # Note on Time Step
/// This is a fixed-step implementation. A more advanced solver would implement adaptive time-stepping
/// based on Local Truncation Error (LTE) estimation, as described in the user-provided pseudo-code.
pub fn solve(
    circuit: &Circuit,
    config: &SolverConfig,
    tran_analysis: &TransientAnalysis,
) -> Result<Vec<HashMap<String, f64>>> {
    // 1. Solve for the initial DC operating point (t=0).
    println!("Calculating initial operating point...");
    let mut initial_op = op::solve(circuit, config)?;
    initial_op.insert("time".to_string(), 0.0);
    let index_map = &circuit.index_map;
    let size = index_map.len();

    // The first result is the DC solution at t=0.
    let mut all_results = vec![initial_op];
    let time_step = tran_analysis.time_step;
    let num_steps = (tran_analysis.stop_time / time_step).round() as usize;

    // Check if the circuit contains any non-linear elements. If not, the solver
    // only needs to run for one iteration.
    let has_nonlinear_elements = &circuit.elements.iter().any(|e| e.is_nonlinear());

    println!(
        "Starting transient analysis from t=0 to t={}s with a {}s time step.",
        tran_analysis.stop_time, time_step
    );

    // --- Main time-stepping loop (Outer Loop) ---
    for step in 1..=num_steps {
        let current_time = step as f64 * time_step;
        let prev_solution = all_results.last().unwrap();

        // --- Inner Newton-Raphson loop to solve for the current time step `t_n` ---
        let mut op_result_at_t = HashMap::new();
        // Use the solution from the previous time step as the initial guess (a "warm start").
        let mut previous_nr_guess = prev_solution.clone();

        for iter in 0..config.maximum_iterations {
            let mut g_stamps = Vec::new();
            let mut e_stamps = Vec::new();

            // Build the MNA matrices using the discretized, linearized stamps (companion models).
            for element in &circuit.elements {
                g_stamps.extend(element.add_conductance_matrix_transient_stamp(
                    index_map,
                    &previous_nr_guess,
                    prev_solution,
                    time_step,
                ));
                e_stamps.extend(element.add_excitation_vector_transient_stamp(
                    index_map,
                    &previous_nr_guess,
                    prev_solution,
                    time_step,
                ));
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

            // #[cfg(debug_assertions)]
            // {
            //     println!("\n--- System State at Iteration: {} ---", iter);
            //     print_system(&g_stamps_summed, &b, &x, index_map);
            // }

            op_result_at_t = index_map
                .iter()
                .map(|(node, &idx)| (node.clone(), x[(idx, 0)]))
                .collect();

            op_result_at_t.insert("time".to_string(), current_time);

            // For purely linear circuits, we only need one iteration.
            if !has_nonlinear_elements {
                break;
            }

            if convergence_check(&previous_nr_guess, &op_result_at_t, config) {
                break; // Newton-Raphson converged for this time step.
            }
            previous_nr_guess = op_result_at_t.clone();

            if iter == config.maximum_iterations - 1 {
                return Err(Error::MaximumIterationsExceeded(config.maximum_iterations));
            }
        }

        println!("Converged at t = {}s", current_time);
        all_results.push(op_result_at_t);
    }
    Ok(all_results)
}

/// A helper function to pretty-print the full MNA system (Gx=b) for debugging.
#[allow(dead_code)]
fn print_system(
    g_triplets: &[Triplet<usize, usize, f64>],
    b_vector: &Mat<f64>,
    x_vector: &Mat<f64>,
    index_map: &HashMap<String, usize>,
) {
    let size = index_map.len();
    let mut rev_index_map: Vec<String> = vec![String::new(); size];
    for (name, &idx) in index_map {
        if idx < size {
            rev_index_map[idx] = name.clone();
        }
    }

    let matrix_map: HashMap<(usize, usize), f64> = g_triplets
        .iter()
        .map(|&Triplet { row, col, val }| ((row, col), val))
        .collect();

    // Print header
    print!("{:<12}", ""); // Spacer for row names
    for i in 0..size {
        print!("{:<12}", rev_index_map[i]);
    }
    println!(
        "{:<15}   {:<15}",
        "| x Vector (Solution)", "| b Vector (Excitation)"
    );
    println!("{}", "-".repeat(12 * (size + 1) + 32));

    // Print each row of the system
    for r in 0..size {
        print!("{:<12}", rev_index_map[r]);
        for c in 0..size {
            let val = matrix_map.get(&(r, c)).unwrap_or(&0.0);
            print!("{:<12.4}", val);
        }
        println!(
            "| {:<15.6e} | {:<15.6e}",
            x_vector.get(r, 0), // Use NaN for missing values
            b_vector.get(r, 0)
        );
    }
}
