use faer::Mat;
use faer::sparse::SparseColMat;
use std::fmt;

pub struct MNAMatrix {
    pub a: SparseColMat<usize, f64>,
    pub z: Mat<f64>,
    pub nodes: Vec<String>,
}

impl fmt::Display for MNAMatrix {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for row in 0..self.a.nrows() {
            for col in 0..self.a.ncols() {
                write!(f, "{:7.3} ", self.a[(row, col)])?;
            }
            write!(f, "| {:5} ", self.nodes[row])?;
            writeln!(f, "|{:7.3}", self.z[(row, 0)])?;
        }
        Ok(())
    }
}

// impl MNAMatrix {
//     pub fn solve(self) {
//         let lu = self.a.sp_lu().unwrap();
//         let x = faer::linalg::solvers::Solve::solve(&lu, &b);
//     }
// }

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
