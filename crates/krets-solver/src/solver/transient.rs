use std::collections::HashMap;

use crate::prelude::*;
use faer::{
    Mat,
    prelude::Solve,
    sparse::{SparseColMat, Triplet},
};
use krets_parser::{
    analyses::TransientAnalysis,
    circuit::Circuit,
    elements::{Element, Identifiable, Stampable},
};

pub fn solve(
    circuit: &mut Circuit,
    config: &SolverConfig,
    transient_analysis: TransientAnalysis,
) -> Result<Vec<HashMap<String, f64>>> {
    let index_map = &circuit.index_map;
    let size = index_map.len();
    let mut all_results = Vec::new();

    Ok(all_results)
}
