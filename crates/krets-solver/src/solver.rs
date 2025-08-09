use crate::prelude::*;
use faer::c64;
use faer::sparse::SparseColMat;
use faer::sparse::Triplet;
use faer::{Mat, linalg::solvers::Solve};
use krets_parser::{analyses::DcAnalysis, circuit::Circuit, elements::Element};
use std::collections::HashMap;

pub struct Solver {
    circuit: Circuit,
    relative_tolerance: f64,
    current_absolute_tolerance: f64,
    voltage_absolute_tolerance: f64,
    maximum_iterations: usize,

    /// Minimum resistance to consider in the solver. All resistors with a value below this will be converted to this value.
    /// This is to avoid numerical issues with very small resistances.
    minimum_resistance: f64,

    minimum_conductance: f64,
}

impl Solver {
    pub const fn new(circuit: Circuit) -> Self {
        Self {
            circuit,
            relative_tolerance: 0.001,
            current_absolute_tolerance: 1e-12,
            voltage_absolute_tolerance: 1e-6,
            maximum_iterations: 300,
            minimum_resistance: 1e-3,
            minimum_conductance: 1e-12,
        }
    }

    pub fn solve_dc(self, dc_analysis: DcAnalysis) -> Result<Vec<HashMap<String, f64>>> {
        let index_map = &self.circuit.index_map;
        let elements = &self.circuit.elements;

        let mut dc_sweep_element = self
            .circuit
            .elements
            .iter()
            .find(|x| x.identifier() == dc_analysis.element)
            .cloned()
            .ok_or_else(|| Error::ElementNotFound(dc_analysis.element.clone()))?;

        let mut results: Vec<HashMap<String, f64>> = vec![];

        let mut current = dc_analysis.start;

        while current <= dc_analysis.stop {
            let mut g_stamps = Vec::new();
            let mut e_stamps = Vec::new();
            let mut result = results.last().cloned().unwrap_or_default();

            for element in elements {
                // Skip capacitors, as they are not included in DC analysis
                if matches!(element, Element::Capacitor(_)) {
                    continue;
                }

                // Skip the DC sweep element for now, we will handle it separately
                if element.identifier() == dc_sweep_element.identifier() {
                    continue;
                }

                g_stamps.extend(element.conductance_matrix_dc_stamp(
                    index_map,
                    results.last().unwrap_or(&HashMap::new()),
                ));
                e_stamps.extend(element.excitation_vector_dc_stamp(index_map, &result));
            }

            dc_sweep_element.set_value(current);
            g_stamps.extend(dc_sweep_element.conductance_matrix_dc_stamp(index_map, &result));
            e_stamps.extend(dc_sweep_element.excitation_vector_dc_stamp(index_map, &result));
            current += dc_analysis.step_size;

            let size = index_map.len();
            let lu = SparseColMat::try_new_from_triplets(size, size, &g_stamps)
                .expect("Failed to build sparse matrix")
                .sp_lu()
                .expect("LU decomposition failed");
            let mut b = Mat::zeros(size, 1);
            for &Triplet { row, col, val } in &e_stamps {
                b[(row, col)] = val;
            }
            let x = lu.solve(&b);

            for (node, &idx) in index_map.iter() {
                result.insert(node.clone(), x[(idx, 0)]);
            }
            result.insert(dc_sweep_element.identifier(), current);

            results.push(result);
        }
        Ok(results)
    }

    pub fn solve_op(&self) -> HashMap<String, f64> {
        let index_map = &self.circuit.index_map;
        let elements = &self.circuit.elements;
        let mut result = HashMap::new();
        let mut previous_result = result.clone();
        let max_iterations = 300;

        for iter in 0..max_iterations {
            let mut g_stamps = Vec::new();
            let mut e_stamps = Vec::new();

            for element in elements {
                if !matches!(element, Element::Capacitor(_)) {
                    g_stamps
                        .extend(element.conductance_matrix_dc_stamp(index_map, &previous_result));
                    e_stamps
                        .extend(element.excitation_vector_dc_stamp(index_map, &previous_result));
                }
            }

            let g_stamps = sum_triplets(&g_stamps);
            let e_stamps = sum_triplets(&e_stamps);

            let size = index_map.len();

            let lu = SparseColMat::try_new_from_triplets(size, size, &g_stamps)
                .expect("Failed to build sparse matrix")
                .sp_lu()
                .expect("LU decomposition failed");

            let mut b = Mat::zeros(size, 1);
            for &Triplet { row, col, val } in &e_stamps {
                b[(row, col)] = val;
            }

            let x = lu.solve(&b);

            result = index_map
                .iter()
                .map(|(node, &idx)| (node.clone(), x[(idx, 0)]))
                .collect();

            if !previous_result.is_empty()
                && result
                    .iter()
                    .all(|(node, &value)| (value - previous_result[node]).abs() < 1e-12)
            {
                println!("Converged after {} iterations", iter + 1);
                break;
            }
            previous_result = result.clone();

            if iter == max_iterations - 1 {
                println!("Warning: Maximum iterations reached without convergence.");
            }
        }

        result
    }

    pub fn solve_ac(self, frequency: f64) -> HashMap<String, c64> {
        let index_map = &self.circuit.index_map;
        let result = HashMap::new();
        let elements = &self.circuit.elements;

        let mut g_stamps = Vec::new();
        let mut e_stamps = Vec::new();

        for element in elements {
            g_stamps.extend(element.conductance_matrix_ac_stamp(index_map, frequency, &result));
            e_stamps.extend(element.excitation_vector_ac_stamp(index_map, frequency, &result));
        }

        let size = index_map.len();
        let lu = SparseColMat::try_new_from_triplets(size, size, &g_stamps)
            .expect("Failed to build sparse matrix")
            .sp_lu()
            .expect("LU decomposition failed");

        let mut b = Mat::zeros(size, 1);
        for &Triplet { row, col, val } in &e_stamps {
            b[(row, col)] = val;
        }
        let x = lu.solve(&b);

        let mut solution_map = HashMap::new();
        for (node, &index) in index_map {
            solution_map.insert(node.clone(), x[(index, 0)]);
        }

        solution_map.insert(
            "frequency".to_string(),
            c64 {
                re: frequency,
                im: 0.0,
            },
        );

        solution_map
    }
}

fn sum_triplets(triplets: &[Triplet<usize, usize, f64>]) -> Vec<Triplet<usize, usize, f64>> {
    let mut map: HashMap<(usize, usize), f64> = HashMap::new();
    for triplet in triplets {
        let key = (triplet.row, triplet.col);
        *map.entry(key).or_insert(0.0) += triplet.val;
    }
    map.into_iter()
        .map(|((row, col), val)| Triplet { row, col, val })
        .collect()
}

#[allow(dead_code)]
fn print_triplet_matrix(triplets: &[Triplet<usize, usize, f64>], rows: usize, cols: usize) {
    // Build a dense matrix initialized to 0.0
    let mut mat = vec![vec![0.0; cols]; rows];
    for triplet in triplets {
        if triplet.row < rows && triplet.col < cols {
            mat[triplet.row][triplet.col] += triplet.val;
        }
    }
    // Print the matrix
    println!("Matrix ({} x {}):", rows, cols);
    (0..rows).for_each(|row| {
        for col in 0..cols {
            print!("{:>12.5e} ", mat[row][col]);
        }
        println!();
    });
}
