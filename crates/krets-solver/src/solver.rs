use crate::prelude::*;
use faer::c64;
use faer::sparse::SparseColMat;
use faer::sparse::Triplet;
use faer::{Mat, linalg::solvers::Solve};
use krets_parser::{analyses::DcAnalysis, circuit::Circuit, elements::Element};
use std::collections::HashMap;

pub struct Solver {
    circuit: Circuit,
}

impl Solver {
    pub const fn new(circuit: Circuit) -> Self {
        Self { circuit }
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

            for element in elements {
                // Skip capacitors, as they are not included in DC analysis
                if matches!(element, Element::Capacitor(_)) {
                    continue;
                }

                // Skip the DC sweep element for now, we will handle it separately
                if element.identifier() == dc_sweep_element.identifier() {
                    continue;
                }

                g_stamps.extend(element.conductance_matrix_dc_stamp(index_map));
                e_stamps.extend(element.excitation_vector_dc_stamp(index_map));
            }

            dc_sweep_element.set_value(current);
            g_stamps.extend(dc_sweep_element.conductance_matrix_dc_stamp(index_map));
            e_stamps.extend(dc_sweep_element.excitation_vector_dc_stamp(index_map));
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
            let mut result = HashMap::new();
            for (node, &idx) in index_map.iter() {
                result.insert(node.clone(), x[(idx, 0)]);
            }
            result.insert(dc_sweep_element.identifier(), current);

            dbg!(&result);

            results.push(result);
        }
        Ok(results)
    }

    pub fn solve_op(&self) -> HashMap<String, f64> {
        let index_map = &self.circuit.index_map;
        let elements = &self.circuit.elements;

        let mut g_stamps = Vec::new();
        let mut e_stamps = Vec::new();

        for element in elements {
            if !matches!(element, Element::Capacitor(_)) {
                g_stamps.extend(element.conductance_matrix_dc_stamp(index_map));
                e_stamps.extend(element.excitation_vector_dc_stamp(index_map));
            }
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

        index_map
            .iter()
            .map(|(node, &idx)| (node.clone(), x[(idx, 0)]))
            .collect()
    }

    pub fn solve_ac(self, frequency: f64) -> HashMap<String, c64> {
        let index_map = &self.circuit.index_map;

        let elements = &self.circuit.elements;

        let mut g_stamps = Vec::new();
        let mut e_stamps = Vec::new();

        for element in elements {
            g_stamps.extend(element.conductance_matrix_ac_stamp(index_map, frequency));
            e_stamps.extend(element.excitation_vector_ac_stamp(index_map, frequency));
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

        dbg!(&x);

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
