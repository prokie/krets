pub mod mna_matrix;

use faer::mat::Mat;
use faer::sparse::SparseColMat;
use std::collections::HashMap;
use std::fmt;
use std::ops::{Index, IndexMut};

pub struct Matrix {
    rows: usize,
    cols: usize,
    data: HashMap<(usize, usize), f64>,
}
use faer::sparse::Triplet;

impl Matrix {
    pub fn new(rows: usize, cols: usize) -> Self {
        Self {
            rows,
            cols,
            data: HashMap::new(),
        }
    }

    pub fn insert(&mut self, row: usize, col: usize, value: f64) {
        self.data.insert((row, col), value);
    }

    pub fn get(&self, row: usize, col: usize) -> Option<&f64> {
        self.data.get(&(row, col))
    }

    pub fn get_mut(&mut self, row: usize, col: usize) -> Option<&mut f64> {
        self.data.get_mut(&(row, col))
    }

    pub fn to_sparse_col_mat(&self) -> SparseColMat<usize, f64> {
        let triplets: Vec<Triplet<usize, usize, f64>> = self
            .data
            .iter()
            .map(|(&(r, c), &v)| Triplet::new(r, c, v))
            .collect();
        SparseColMat::try_new_from_triplets(self.rows, self.cols, &triplets).unwrap()
    }

    pub fn to_dense_mat(&self) -> Mat<f64> {
        Mat::from_fn(self.rows, self.cols, |row, col| {
            *self.data.get(&(row, col)).unwrap_or(&0.0)
        })
    }
    // pub fn to_dense_mat(&self) -> Mat<f64> {
    //     let mut mat = Mat::<f64>::zeros(self.rows, self.cols);
    //     for ((row, col), &value) in &self.data {
    //         mat[(row, col)] = value;
    //     }
    //     mat
    // }

    pub fn new_empty(rows: usize, cols: usize) -> Self {
        let mut data = HashMap::new();
        for row in 0..rows {
            for col in 0..cols {
                data.insert((row, col), 0.0);
            }
        }
        Self { rows, cols, data }
    }
}

impl Index<(usize, usize)> for Matrix {
    type Output = f64;

    fn index(&self, index: (usize, usize)) -> &Self::Output {
        self.get(index.0, index.1).expect("Index out of bounds")
    }
}

impl IndexMut<(usize, usize)> for Matrix {
    fn index_mut(&mut self, index: (usize, usize)) -> &mut Self::Output {
        self.get_mut(index.0, index.1).expect("Index out of bounds")
    }
}

impl fmt::Display for Matrix {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for row in 0..self.rows {
            for col in 0..self.cols {
                let value = self.get(row, col).unwrap_or(&0.0);
                write!(f, "{value:7.3} ")?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

impl fmt::Debug for Matrix {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self}")
    }
}

#[cfg(test)]
mod tests {
    use faer::mat;

    use super::*;

    #[test]
    fn test_solve_sparse_matrix() {
        // Create a Matrix instance and insert values
        let mut matrix = Matrix::new_empty(2, 2);
        matrix[(0, 0)] = 10.0;
        matrix[(1, 0)] = 2.0;
        matrix[(0, 1)] = 2.0;
        matrix[(1, 1)] = 10.0;

        // Convert to SparseColMat
        let a = matrix.to_sparse_col_mat();

        println!("{matrix:?}");
        println!("{matrix}");

        // Create a dense vector 'b'
        let b = mat![[15.0], [-3.0f64]];

        let lu = a.sp_lu().unwrap();
        let x = faer::linalg::solvers::Solve::solve(&lu, &b);

        // Print the solution
        println!("Solution x = {x:?}");

        // Check the solution (you can add more assertions as needed)
        assert!(x[(0, 0)] > 0.0);
        assert!(x[(1, 0)] < 0.0);
    }
}
