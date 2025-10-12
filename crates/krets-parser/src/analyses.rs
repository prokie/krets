use faer::c64;
use std::collections::HashMap;

/// Defines the type of analysis to be performed and its parameters.
#[derive(Debug, Clone)]
pub enum Analysis {
    /// DC Operating Point Analysis.
    Op,

    /// DC Sweep Analysis.
    Dc(DcAnalysis),

    /// AC Small-Signal Frequency Analysis.
    Ac { frequency: f64 },

    /// Transient Analysis.
    Transient(TransientAnalysis),
}

/// Contains the parameters for a DC Sweep analysis.
#[derive(Debug, Clone)]
pub struct DcAnalysis {
    /// The identifier of the element to sweep (e.g., "V1").
    pub element: String,
    /// The starting value for the sweep.
    pub start: f64,
    /// The ending value for the sweep.
    pub stop: f64,
    /// The increment for each step of the sweep.
    pub step_size: f64,
}

#[derive(Debug, Clone)]
pub struct TransientAnalysis {
    pub time_step: f64,
    pub total_time: f64,
}

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
    /// A HashMap containing the complex-valued solution at a single frequency.
    Ac(HashMap<String, c64>),

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
    pub fn into_ac(self) -> HashMap<String, c64> {
        match self {
            AnalysisResult::Ac(result) => result,
            _ => panic!("Called `into_ac()` on a non-Ac result"),
        }
    }
}
