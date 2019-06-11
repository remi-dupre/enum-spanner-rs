use std::fmt;
use std::iter;
use std::ops::{Index, Mul};

use super::tools::iter_complement;

/// Naive representation of a matrix as a single consecutive chunk of memory.
pub struct Matrix<T> {
    height: usize,
    width:  usize,
    data:   Vec<T>,
}

// Custom trait for matrices that can be right-multiplied by a column vector.
pub trait ColMul<U> {
    fn col_mul(&self, column: &Vec<U>) -> Vec<U>;
}

impl<'a, T> Matrix<T>
where
    T: Copy + Default,
{
    /// Create a matrix filled with one value.
    pub fn new(height: usize, width: usize, value: T) -> Matrix<T> {
        Matrix {
            width,
            height,
            data: vec![value; width * height],
        }
    }

    pub fn get_height(&self) -> usize {
        self.height
    }

    pub fn get_width(&self) -> usize {
        self.width
    }

    /// Mutable access to an element of the matrix.
    pub fn at(&mut self, row: usize, col: usize) -> &mut T {
        let index = self.data_index(row, col);
        &mut self.data[index]
    }

    /// Get an iterator over a column of the matrix.
    pub fn iter_col(&self, col: usize) -> impl Iterator<Item = &T> {
        debug_assert!(col < self.width);
        self.data
            .iter()
            .skip(col)
            .step_by(self.width)
            .take(self.height)
    }

    /// Get an iterator over a row of the matrix.
    pub fn iter_row(&self, row: usize) -> impl Iterator<Item = &T> {
        debug_assert!(row < self.height);
        self.data.iter().skip(row * self.width).take(self.width)
    }

    /// Truncate rows and columns from a matrix.
    pub fn truncate<U, V>(&self, del_rows: U, del_cols: V) -> Matrix<T>
    where
        U: Iterator<Item = usize>,
        V: Iterator<Item = usize>,
    {
        let kept_rows = iter_complement(0, self.height, del_rows);
        let kept_cols = iter_complement(0, self.width, del_cols);
        self.submatrix_sorted(kept_rows, kept_cols)
    }

    pub fn del_row(&self, row: usize) -> Matrix<T> {
        self.truncate(iter::once(row), iter::empty())
    }

    pub fn del_col(&self, col: usize) -> Matrix<T> {
        self.truncate(iter::empty(), iter::once(col))
    }

    /// Get the index of a cell in the data vector.
    fn data_index(&self, row: usize, col: usize) -> usize {
        debug_assert!(col < self.width);
        debug_assert!(row < self.height);
        col + (row * self.width)
    }

    /// Get a submatrix containing only some rows and columns given as sorted
    /// iterator.
    fn submatrix_sorted<U, V>(&self, rows: U, cols: V) -> Matrix<T>
    where
        U: Iterator<Item = usize> + Clone,
        V: Iterator<Item = usize> + Clone,
        T: Clone,
    {
        let mut all_data_iter = self.data.iter();
        let indices = rows
            .clone()
            .map(|row| cols.clone().map(move |col| self.data_index(row, col)))
            .flatten();
        let data: Vec<_> = indices
            .scan(0, |expected_index, index| {
                let val = all_data_iter.nth(index - *expected_index);
                *expected_index = index + 1;
                val.cloned()
            })
            .collect();

        Matrix {
            width: cols.count(),
            height: rows.count(),
            data,
        }
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

//  ____              _
// | __ )  ___   ___ | | ___  __ _ _ __
// |  _ \ / _ \ / _ \| |/ _ \/ _` | '_ \
// | |_) | (_) | (_) | |  __/ (_| | | | |
// |____/ \___/ \___/|_|\___|\__,_|_| |_|
//  __  __       _        _
// |  \/  | __ _| |_ _ __(_)_  __
// | |\/| |/ _` | __| '__| \ \/ /
// | |  | | (_| | |_| |  | |>  <
// |_|  |_|\__,_|\__|_|  |_/_/\_\
//

impl Mul for &Matrix<bool> {
    type Output = Matrix<bool>;

    fn mul(self, other: &Matrix<bool>) -> Matrix<bool> {
        let data = (0..self.height)
            .map(|row| {
                (0..other.width).map(move |col| {
                    let row_iter = self.iter_row(row);
                    let col_iter = other.iter_col(col);
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

//  ____       _
// |  _ \  ___| |__  _   _  __ _
// | | | |/ _ \ '_ \| | | |/ _` |
// | |_| |  __/ |_) | |_| | (_| |
// |____/ \___|_.__/ \__,_|\__, |
//                         |___/

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
