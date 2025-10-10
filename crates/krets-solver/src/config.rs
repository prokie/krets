/// Configuration structure for controlling solver parameters across different simulation types

#[derive(Clone, Debug)]
pub struct SolverConfig {
    /// Tolerance for convergence based on relative error
    pub relative_tolerance: f64,

    /// Absolute tolerance for node currents during simulation
    pub current_absolute_tolerance: f64,

    /// Absolute tolerance for node voltages (in volts)
    pub voltage_absolute_tolerance: f64,

    /// Maximum number of iterations before solver aborts
    pub maximum_iterations: usize,

    /// Minimum resistance to consider; resistors below this value are set to this minimum.
    /// This prevents numerical issues with extremely small resistances.
    pub minimum_resistance: f64, // Note: Changed from `pub` since it's an internal parameter

    /// Minimum conductance (inverse of resistance) considered by the solver
    pub minimum_conductance: f64,
}

/// Default configuration for the solver, providing reasonable defaults for all parameters.
impl Default for SolverConfig {
    fn default() -> Self {
        SolverConfig {
            relative_tolerance: 0.001,
            current_absolute_tolerance: 1e-12,
            voltage_absolute_tolerance: 1e-6,
            maximum_iterations: 300,
            minimum_resistance: 1e-3,
            minimum_conductance: 1e-12,
        }
    }
}
