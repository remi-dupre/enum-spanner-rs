use std::fmt;
use std::ops::{Index, Mul};

pub struct Matrix<T> {
    height: usize,
    width: usize,
    data: Vec<T>,
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

impl Mul for &Matrix<bool> {
    type Output = Matrix<bool>;

    fn mul(self, other: &Matrix<bool>) -> Matrix<bool> {
        let data = (0..self.height)
            .map(|row| {
                (0..other.width).map(move |col| {
                    self.iter_row(row)
                        .zip(other.iter_col(col))
                        .any(|(&x, &y)| x && y)
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
