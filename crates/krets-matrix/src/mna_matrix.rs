use std::collections::HashMap;
use std::fmt;

use crate::Matrix;

pub struct MnaMatrix {
    pub conductance_matrix: Matrix,

    // /// The solution vector containing unknown node voltages and currents.
    // pub solution_vector: Matrix,
    /// The B vector (excitation vector) representing independent sources.
    pub excitation_vector: Matrix,

    /// Maps circuit nodes and elements to their corresponding indices in the matrix.
    pub index_map: HashMap<String, usize>,
}

impl fmt::Display for MnaMatrix {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut index_to_node: Vec<&str> = vec![""; self.conductance_matrix.rows];
        for (node, &index) in &self.index_map {
            index_to_node[index] = node;
        }

        for (row, index) in index_to_node.iter().enumerate() {
            for col in 0..self.conductance_matrix.cols {
                write!(f, "{:7.3} ", self.conductance_matrix[(row, col)])?;
            }
            write!(f, "| {index:5} ")?;
            writeln!(f, "|{:7.3}", self.excitation_vector[(row, 0)])?;
        }
        Ok(())
    }
}

impl MnaMatrix {
    pub fn solve(self) -> HashMap<String, f64> {
        let lu = self.conductance_matrix.to_sparse_col_mat().sp_lu().unwrap();

        let x = faer::linalg::solvers::Solve::solve(&lu, &self.excitation_vector.to_dense_mat());

        let mut solution_map = HashMap::new();
        for (node, &index) in &self.index_map {
            solution_map.insert(node.clone(), x[(index, 0)]);
        }

        solution_map
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use faer::Mat;

//     #[test]
//     fn test_pretty_print_matrix() {
//         let data = [
//             vec![1.0, 2.0, 3.0],
//             vec![4.0, 5.0, 6.0],
//             vec![7.0, 8.0, 9.0],
//         ];
//         let mut mat = Mat::<f64>::zeros(3, 3);
//         for (i, row) in data.iter().enumerate() {
//             for (j, &val) in row.iter().enumerate() {
//                 mat[(i, j)] = val;
//             }
//         }
//         let matrix = MNAMatrix {
//             a: mat,
//             z: Mat::zeros(3, 1),
//             nodes: vec!["V1".to_string(), "V2".to_string(), "V3".to_string()],
//         };
//         println!("{matrix}");
//     }
// }
