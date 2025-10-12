pub use crate::error::Error;
pub type Result<T> = core::result::Result<T, Error>;

pub use crate::config::SolverConfig;
pub use crate::solver::convergence_check;
pub use crate::solver::sum_triplets;
