use std::fmt;
use std::ops::{Index, Mul};

/// Naive representation of a matrix as a single consecutive chunk of memory.
pub struct Matrix<T> {
    height: usize,
    width: usize,
    data: Vec<T>,
}

// Custom trait for matrices that can be right-multiplied by a column vector.
pub trait ColMul<U> {
    fn col_mul(&self, column: &Vec<U>) -> Vec<U>;
}

impl<'a, T> Matrix<T>
where
    T: Copy + Default,
{
    pub fn new(height: usize, width: usize, value: T) -> Matrix<T> {
        Matrix {
            width,
            height,
            data: vec![value; width * height],
        }
    }

    pub fn at(&mut self, row: usize, col: usize) -> &mut T {
        let index = self.data_index(row, col);
        &mut self.data[index]
    }

    pub fn iter_col(&self, col: usize) -> impl Iterator<Item = &T> {
        assert!(col < self.width);
        self.data
            .iter()
            .skip(col)
            .step_by(self.width)
            .take(self.height)
    }

    pub fn iter_row(&self, row: usize) -> impl Iterator<Item = &T> {
        assert!(row < self.height);
        self.data.iter().skip(row * self.width).take(self.width)
    }

    fn data_index(&self, row: usize, col: usize) -> usize {
        assert!(col < self.width);
        assert!(row < self.height);
        col + (row * self.width)
    }
}

impl<T> Index<(usize, usize)> for Matrix<T>
where
    T: Copy + Default,
{
    type Output = T;

    fn index(&self, (row, col): (usize, usize)) -> &T {
        &self.data[self.data_index(row, col)]
    }
}

//  ____              _                    __  __       _        _
// | __ )  ___   ___ | | ___  __ _ _ __   |  \/  | __ _| |_ _ __(_) ___ ___  ___
// |  _ \ / _ \ / _ \| |/ _ \/ _` | '_ \  | |\/| |/ _` | __| '__| |/ __/ _ \/ __|
// | |_) | (_) | (_) | |  __/ (_| | | | | | |  | | (_| | |_| |  | | (_|  __/\__ \
// |____/ \___/ \___/|_|\___|\__,_|_| |_| |_|  |_|\__,_|\__|_|  |_|\___\___||___/
//

impl Mul for &Matrix<bool> {
    type Output = Matrix<bool>;

    fn mul(self, other: &Matrix<bool>) -> Matrix<bool> {
        let data = (0..self.height)
            .map(|row| {
                (0..other.width).map(move |col| {
                    let row_iter = self.iter_row(row);
                    let col_iter = self.iter_col(col);
                    row_iter.zip(col_iter).any(|(&x, &y)| x && y)
                })
            })
            .flatten()
            .collect();

        Matrix {
            width: other.width,
            height: self.height,
            data,
        }
    }
}

impl ColMul<bool> for Matrix<bool> {
    fn col_mul(&self, column: &Vec<bool>) -> Vec<bool> {
        (0..self.height)
            .map(|row| {
                let row_iter = self.iter_row(row);
                let col_iter = column.iter();
                row_iter.zip(col_iter).any(|(&x, &y)| x && y)
            })
            .collect()
    }
}

impl fmt::Debug for Matrix<bool> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let content = (0..self.height)
            .map(|row| {
                let row = self
                    .iter_row(row)
                    .map(|x| match x {
                        true => "T",
                        false => "F",
                    })
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("[{}]", row)
            })
            .collect::<Vec<_>>()
            .join(",\n ");

        write!(f, "[{}]\n", content)
    }
}
