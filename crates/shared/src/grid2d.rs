use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Coord2 {
    pub x: usize,
    pub y: usize,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Grid2D<T> {
    width: usize,
    height: usize,
    data: Vec<T>,
}

impl<T: Clone> Grid2D<T> {
    pub fn filled(width: usize, height: usize, value: T) -> Option<Self> {
        if width == 0 || height == 0 {
            return None;
        }
        let len = width.checked_mul(height)?;
        Some(Self {
            width,
            height,
            data: vec![value; len],
        })
    }
}

impl<T> Grid2D<T> {
    pub fn from_vec(width: usize, height: usize, data: Vec<T>) -> Option<Self> {
        if width == 0 || height == 0 {
            return None;
        }
        let len = width.checked_mul(height)?;
        if data.len() != len {
            return None;
        }
        Some(Self {
            width,
            height,
            data,
        })
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    pub fn data(&self) -> &[T] {
        &self.data
    }

    pub fn data_mut(&mut self) -> &mut [T] {
        &mut self.data
    }

    pub fn in_bounds(&self, x: usize, y: usize) -> bool {
        x < self.width && y < self.height
    }

    pub fn index_of(&self, x: usize, y: usize) -> Option<usize> {
        if !self.in_bounds(x, y) {
            return None;
        }
        y.checked_mul(self.width).and_then(|row| row.checked_add(x))
    }

    pub fn coords_of(&self, idx: usize) -> Option<Coord2> {
        if idx >= self.data.len() {
            return None;
        }
        Some(Coord2 {
            x: idx % self.width,
            y: idx / self.width,
        })
    }

    pub fn get(&self, x: usize, y: usize) -> Option<&T> {
        self.index_of(x, y).and_then(|idx| self.data.get(idx))
    }

    pub fn get_mut(&mut self, x: usize, y: usize) -> Option<&mut T> {
        self.index_of(x, y).and_then(|idx| self.data.get_mut(idx))
    }

    pub fn set(&mut self, x: usize, y: usize, value: T) -> bool {
        if let Some(idx) = self.index_of(x, y) {
            self.data[idx] = value;
            true
        } else {
            false
        }
    }

    pub fn iter(&self) -> std::slice::Iter<'_, T> {
        self.data.iter()
    }

    pub fn iter_mut(&mut self) -> std::slice::IterMut<'_, T> {
        self.data.iter_mut()
    }

    pub fn neighbors4(&self, x: usize, y: usize) -> Vec<Coord2> {
        let mut out = Vec::with_capacity(4);
        if x + 1 < self.width {
            out.push(Coord2 { x: x + 1, y });
        }
        if x > 0 {
            out.push(Coord2 { x: x - 1, y });
        }
        if y + 1 < self.height {
            out.push(Coord2 { x, y: y + 1 });
        }
        if y > 0 {
            out.push(Coord2 { x, y: y - 1 });
        }
        out
    }

    pub fn neighbors8(&self, x: usize, y: usize) -> Vec<Coord2> {
        let mut out = Vec::with_capacity(8);
        let y0 = y.saturating_sub(1);
        let y1 = (y + 1).min(self.height - 1);
        let x0 = x.saturating_sub(1);
        let x1 = (x + 1).min(self.width - 1);

        for yy in y0..=y1 {
            for xx in x0..=x1 {
                if xx == x && yy == y {
                    continue;
                }
                out.push(Coord2 { x: xx, y: yy });
            }
        }
        out
    }
}

impl<T: Clone> Grid2D<T> {
    pub fn fill(&mut self, value: T) {
        self.data.fill(value);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn index_roundtrip() {
        let g = Grid2D::<i32>::filled(4, 3, 0).unwrap();
        let idx = g.index_of(2, 1).unwrap();
        assert_eq!(idx, 6);
        assert_eq!(g.coords_of(idx), Some(Coord2 { x: 2, y: 1 }));
    }

    #[test]
    fn neighbors_counts() {
        let g = Grid2D::<i32>::filled(3, 3, 0).unwrap();
        assert_eq!(g.neighbors4(1, 1).len(), 4);
        assert_eq!(g.neighbors8(1, 1).len(), 8);
        assert_eq!(g.neighbors8(0, 0).len(), 3);
    }
}

