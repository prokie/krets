pub mod config;
pub mod error;
pub mod prelude;
pub mod solver;
pub mod stampable;
use crate::prelude::*;

/// Holds the output data from a completed analysis.
///
/// Each variant corresponds to a variant in the `Analysis` enum and holds
/// the specific data structure for that analysis type's results.
#[derive(Debug, Clone)]
pub enum AnalysisResult {
    /// Result of an Operating Point analysis.
    /// A single HashMap representing the DC solution.
    Op(HashMap<String, f64>),

    /// Result of a DC Sweep analysis.
    /// A vector of HashMaps, where each map is the solution at one sweep point.
    Dc(Vec<HashMap<String, f64>>),

    /// Result of an AC Small-Signal Analysis.
    /// A vector of HashMaps, where each map is the solution at one frequency.
    Ac(Vec<HashMap<String, c64>>),

    /// Result of a Transient analysis.
    /// A vector of HashMaps, where each map is the solution at one
    /// time step.
    Transient(Vec<HashMap<String, f64>>),
}

impl AnalysisResult {
    /// Unwraps the `AnalysisResult` to get the `Op` result.
    ///
    /// # Panics
    /// Panics if the result is not `AnalysisResult::Op`.
    pub fn into_op(self) -> HashMap<String, f64> {
        match self {
            AnalysisResult::Op(result) => result,
            _ => panic!("Called `into_op()` on a non-Op result"),
        }
    }

    /// Unwraps the `AnalysisResult` to get the `Dc` result.
    ///
    /// # Panics
    /// Panics if the result is not `AnalysisResult::Dc`.
    pub fn into_dc(self) -> Vec<HashMap<String, f64>> {
        match self {
            AnalysisResult::Dc(result) => result,
            _ => panic!("Called `into_dc()` on a non-Dc result"),
        }
    }

    /// Unwraps the `AnalysisResult` to get the `Ac` result.
    ///
    /// # Panics
    /// Panics if the result is not `AnalysisResult::Ac`.
    pub fn into_ac(self) -> Vec<HashMap<String, c64>> {
        match self {
            AnalysisResult::Ac(result) => result,
            _ => panic!("Called `into_ac()` on a non-Ac result"),
        }
    }

    /// Unwraps the `AnalysisResult` to get the `Transient` result.
    ///
    /// # Panics
    /// Panics if the result is not `AnalysisResult::Transient`.
    pub fn into_transient(self) -> Vec<HashMap<String, f64>> {
        match self {
            AnalysisResult::Transient(result) => result,
            _ => panic!("Called `into_transient()` on a non-Transient result"),
        }
    }
}
