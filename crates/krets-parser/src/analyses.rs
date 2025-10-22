use crate::prelude::*;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

// Add a small struct that pairs a circuit file path with an analysis to run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisSpec {
    /// Path to the circuit file (relative or absolute).
    pub circuit_path: PathBuf,
    /// The analysis to perform for the circuit.
    pub analysis: Analysis,
}

impl AnalysisSpec {
    /// Read an AnalysisSpec from a TOML file on disk.
    ///
    /// Returns Err(...) if the file cannot be read or the TOML fails to deserialize.
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let s = std::fs::read_to_string(path)?;
        let spec: AnalysisSpec = toml::from_str(&s)?;
        Ok(spec)
    }
}

/// Defines the type of analysis to be performed and its parameters.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
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
#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransientAnalysis {
    pub time_step: f64,
    pub stop_time: f64,
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

// Add a small test that parses a transient TOML block.
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parse_transient_toml() {
        let toml_str = r#"
[transient]
time_step = 1e-6
stop_time = 1e-3
"#;
        let parsed: Analysis =
            toml::from_str(toml_str).expect("failed to parse TOML into Analysis");
        match parsed {
            Analysis::Transient(t) => {
                assert_eq!(t.time_step, 1e-6);
                assert_eq!(t.stop_time, 1e-3);
            }
            other => panic!("expected Transient analysis, got {:?}", other),
        }
    }

    #[test]
    fn parse_analysis_spec_from_toml_str() {
        let toml_str = r#"
circuit_path = "any_path/krets.toml"

[analysis.transient]
time_step = 1e-6
stop_time = 1e-3
"#;
        let spec: AnalysisSpec =
            toml::from_str(toml_str).expect("failed to parse TOML into AnalysisSpec");

        assert!(spec.circuit_path.ends_with("krets.toml"));

        match spec.analysis {
            Analysis::Transient(t) => {
                assert_eq!(t.time_step, 1e-6);
                assert_eq!(t.stop_time, 1e-3);
            }
            other => panic!(
                "expected Transient analysis in AnalysisSpec, got {:?}",
                other
            ),
        }
    }
}
