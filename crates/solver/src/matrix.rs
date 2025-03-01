use faer::Mat;
use std::fmt;

pub struct Matrix {
    matrix: Mat<f64>,
}

impl fmt::Display for Matrix {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for row in 0..self.matrix.nrows() {
            for col in 0..self.matrix.ncols() {
                write!(f, "{:8.3} ", self.matrix[(row, col)])?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use faer::Mat;

    #[test]
    fn test_pretty_print_matrix() {
        let data = [
            vec![1.0, 2.0, 3.0],
            vec![4.0, 5.0, 6.0],
            vec![7.0, 8.0, 9.0],
        ];
        let mut mat = Mat::<f64>::zeros(3, 3);
        for (i, row) in data.iter().enumerate() {
            for (j, &val) in row.iter().enumerate() {
                mat[(i, j)] = val;
            }
        }
        let matrix = Matrix { matrix: mat };
        println!("{matrix}");
    }
}
