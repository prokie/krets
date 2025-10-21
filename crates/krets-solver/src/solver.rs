pub mod ac;
pub mod dc;
pub mod op;
pub mod transient;

use crate::config::SolverConfig;
use crate::prelude::*;
use faer::sparse::Triplet;
use krets_parser::analyses::{Analysis, AnalysisResult};
use krets_parser::circuit::Circuit;
use std::collections::HashMap;
use std::ops::AddAssign;

// Declare the sub-modules for each analysis type.

/// The main Solver struct, which acts as a dispatcher for different analysis types.
pub struct Solver {
    circuit: Circuit,
    config: SolverConfig,
}

impl Solver {
    pub const fn new(circuit: Circuit, config: SolverConfig) -> Self {
        Self { circuit, config }
    }

    /// Main entry point for running a circuit analysis.
    ///
    /// This function dispatches to the appropriate internal solver based on the
    /// `Analysis` enum variant provided.
    pub fn solve(&mut self, analysis: Analysis) -> Result<AnalysisResult> {
        match analysis {
            Analysis::Op => {
                let result = op::solve(&self.circuit, &self.config)?;
                Ok(AnalysisResult::Op(result))
            }
            Analysis::Dc(dc_params) => {
                // Pass the circuit mutably to allow the sweep to temporarily change element values.
                let result = dc::solve(&mut self.circuit, &self.config, &dc_params)?;
                Ok(AnalysisResult::Dc(result))
            }
            Analysis::Ac { frequency } => {
                let result = ac::solve(&self.circuit, &self.config, frequency)?;
                Ok(AnalysisResult::Ac(result))
            }
            Analysis::Transient(transient_params) => {
                // Pass the circuit mutably to allow time-dependent elements to update their state.
                let result = transient::solve(&self.circuit, &self.config, &transient_params)?;
                Ok(AnalysisResult::Transient(result))
            }
        }
    }
}

/// Generic function to sum triplets for both DC (f64) and AC (c64) analysis.
///
/// This function aggregates a list of MNA stamp contributions, summing the values
/// for any triplets that target the same matrix cell (row, col).
pub fn sum_triplets<N>(triplets: &[Triplet<usize, usize, N>]) -> Vec<Triplet<usize, usize, N>>
where
    N: Copy + AddAssign + Default,
{
    let mut map: HashMap<(usize, usize), N> = HashMap::new();
    for triplet in triplets {
        *map.entry((triplet.row, triplet.col)).or_default() += triplet.val;
    }
    map.into_iter()
        .map(|((row, col), val)| Triplet { row, col, val })
        .collect()
}

/// Checks if the Newton-Raphson iteration has converged.
///
/// Convergence is determined by comparing the change between the previous and current
/// solution vectors against a set of relative and absolute tolerances.
pub fn convergence_check(
    previous_result: &HashMap<String, f64>,
    result: &HashMap<String, f64>,
    config: &SolverConfig,
) -> bool {
    let reltol = config.relative_tolerance;
    let current_tol = config.current_absolute_tolerance;
    let voltage_tol = config.voltage_absolute_tolerance;

    if previous_result.is_empty() {
        return false;
    }

    result.iter().all(|(name, &value)| {
        let prev_value = previous_result.get(name).copied().unwrap_or(0.0);

        let diff = (value - prev_value).abs();
        let scale = value.abs().max(prev_value.abs());

        // Pick which absolute tolerance applies based on whether it's a voltage or current.
        let atol = if name.starts_with('I') {
            current_tol
        } else {
            voltage_tol
        };

        diff <= reltol * scale + atol
    })
}
