use std::{
    collections::HashMap,
    fmt,
    ops::{Index, IndexMut},
};

use faer::{
    Mat, c64,
    sparse::{SparseColMat, Triplet},
};

pub struct ComplexMatrix {
    pub rows: usize,
    pub cols: usize,
    pub data: HashMap<(usize, usize), c64>,
}

impl ComplexMatrix {
    pub fn new(rows: usize, cols: usize) -> Self {
        Self {
            rows,
            cols,
            data: HashMap::new(),
        }
    }

    pub fn insert(&mut self, row: usize, col: usize, value: c64) {
        self.data.insert((row, col), value);
    }

    pub fn get(&self, row: usize, col: usize) -> Option<&c64> {
        self.data.get(&(row, col))
    }

    pub fn get_mut(&mut self, row: usize, col: usize) -> Option<&mut c64> {
        self.data.get_mut(&(row, col))
    }

    pub fn to_sparse_col_mat(&self) -> SparseColMat<usize, c64> {
        let triplets: Vec<Triplet<usize, usize, c64>> = self
            .data
            .iter()
            .map(|(&(r, c), &v)| Triplet::new(r, c, v))
            .collect();
        SparseColMat::try_new_from_triplets(self.rows, self.cols, &triplets).unwrap()
    }

    pub fn to_dense_mat(&self) -> Mat<c64> {
        Mat::from_fn(self.rows, self.cols, |row, col| {
            *self
                .data
                .get(&(row, col))
                .unwrap_or(&c64 { re: 0.0, im: 0.0 })
        })
    }

    pub fn new_empty(rows: usize, cols: usize) -> Self {
        let mut data = HashMap::new();
        for row in 0..rows {
            for col in 0..cols {
                data.insert((row, col), c64 { re: 0.0, im: 0.0 });
            }
        }
        Self { rows, cols, data }
    }
}

impl Index<(usize, usize)> for ComplexMatrix {
    type Output = c64;

    fn index(&self, index: (usize, usize)) -> &Self::Output {
        self.get(index.0, index.1).expect("Index out of bounds")
    }
}

impl IndexMut<(usize, usize)> for ComplexMatrix {
    fn index_mut(&mut self, index: (usize, usize)) -> &mut Self::Output {
        self.get_mut(index.0, index.1).expect("Index out of bounds")
    }
}

impl fmt::Display for ComplexMatrix {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for row in 0..self.rows {
            for col in 0..self.cols {
                let value = self.get(row, col).unwrap_or(&c64 { re: 0.0, im: 0.0 });
                write!(f, "{value:7.3} ")?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

impl fmt::Debug for ComplexMatrix {
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
        let mut matrix = ComplexMatrix::new_empty(2, 2);
        matrix[(0, 0)] = c64 { re: 10.0, im: 0.0 };
        matrix[(1, 0)] = c64 { re: 2.0, im: 0.0 };
        matrix[(0, 1)] = c64 { re: 2.0, im: 0.0 };
        matrix[(1, 1)] = c64 { re: 10.0, im: 0.0 };

        // Convert to SparseColMat
        let a = matrix.to_sparse_col_mat();

        println!("{matrix:?}");
        println!("{matrix}");

        // Create a dense vector 'b'
        let b = mat![[c64 { re: 15.0, im: 0.0 }], [c64 { re: -3.0, im: 0.0 }]];

        let lu = a.sp_lu().unwrap();
        let x = faer::linalg::solvers::Solve::solve(&lu, &b);

        // Print the solution
        println!("Solution x = {x:?}");

        // Check the solution (you can add more assertions as needed)
        assert!(x[(0, 0)].re > 0.0);
        assert!(x[(1, 0)].re < 0.0);
    }
}
