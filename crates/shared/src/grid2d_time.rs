use serde::{Deserialize, Serialize};

use crate::grid2d::Grid2D;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Coord2T {
    pub x: usize,
    pub y: usize,
    pub t: usize,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Grid2DTime<T> {
    width: usize,
    height: usize,
    time: usize,
    data: Vec<T>,
}

impl<T: Clone> Grid2DTime<T> {
    pub fn filled(width: usize, height: usize, time: usize, value: T) -> Option<Self> {
        if width == 0 || height == 0 || time == 0 {
            return None;
        }
        let len = width.checked_mul(height)?.checked_mul(time)?;
        Some(Self {
            width,
            height,
            time,
            data: vec![value; len],
        })
    }

    pub fn get_time_slice(&self, t: usize) -> Option<Grid2D<T>> {
        if t >= self.time {
            return None;
        }
        let start = self.width.checked_mul(self.height)?.checked_mul(t)?;
        let end = start.checked_add(self.width.checked_mul(self.height)?)?;
        Grid2D::from_vec(self.width, self.height, self.data[start..end].to_vec())
    }

    pub fn set_time_slice(&mut self, t: usize, slice: &Grid2D<T>) -> bool {
        if t >= self.time || slice.width() != self.width || slice.height() != self.height {
            return false;
        }

        let Some(start) = self
            .width
            .checked_mul(self.height)
            .and_then(|s| s.checked_mul(t))
        else {
            return false;
        };
        let end = start + self.width * self.height;
        self.data[start..end].clone_from_slice(slice.data());
        true
    }

    pub fn step_copy(&mut self, from_t: usize, to_t: usize) -> bool {
        let Some(slice) = self.get_time_slice(from_t) else {
            return false;
        };
        self.set_time_slice(to_t, &slice)
    }
}

impl<T> Grid2DTime<T> {
    pub fn from_vec(width: usize, height: usize, time: usize, data: Vec<T>) -> Option<Self> {
        if width == 0 || height == 0 || time == 0 {
            return None;
        }
        let len = width.checked_mul(height)?.checked_mul(time)?;
        if data.len() != len {
            return None;
        }
        Some(Self {
            width,
            height,
            time,
            data,
        })
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn time(&self) -> usize {
        self.time
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    pub fn in_bounds(&self, x: usize, y: usize, t: usize) -> bool {
        x < self.width && y < self.height && t < self.time
    }

    pub fn index_of(&self, x: usize, y: usize, t: usize) -> Option<usize> {
        if !self.in_bounds(x, y, t) {
            return None;
        }
        t.checked_mul(self.height)
            .and_then(|v| v.checked_add(y))
            .and_then(|v| v.checked_mul(self.width))
            .and_then(|v| v.checked_add(x))
    }

    pub fn get(&self, x: usize, y: usize, t: usize) -> Option<&T> {
        self.index_of(x, y, t).and_then(|idx| self.data.get(idx))
    }

    pub fn get_mut(&mut self, x: usize, y: usize, t: usize) -> Option<&mut T> {
        self.index_of(x, y, t)
            .and_then(|idx| self.data.get_mut(idx))
    }

    pub fn set(&mut self, x: usize, y: usize, t: usize, value: T) -> bool {
        if let Some(idx) = self.index_of(x, y, t) {
            self.data[idx] = value;
            true
        } else {
            false
        }
    }

    pub fn data(&self) -> &[T] {
        &self.data
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn index_works() {
        let g = Grid2DTime::<i32>::filled(4, 3, 2, 0).unwrap();
        assert_eq!(g.index_of(1, 2, 1), Some(21));
    }

    #[test]
    fn slice_copy_works() {
        let mut g = Grid2DTime::<i32>::filled(2, 2, 2, 0).unwrap();
        assert!(g.set(1, 1, 0, 9));
        assert!(g.step_copy(0, 1));
        assert_eq!(g.get(1, 1, 1), Some(&9));
    }
}

