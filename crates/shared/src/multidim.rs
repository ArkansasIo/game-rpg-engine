#[derive(Clone, Debug, PartialEq)]
pub struct MultiDim<T, const N: usize> {
    dims: [usize; N],
    strides: [usize; N],
    data: Vec<T>,
}

pub type MultiDim2<T> = MultiDim<T, 2>;
pub type MultiDim3<T> = MultiDim<T, 3>;
pub type MultiDim4<T> = MultiDim<T, 4>;

pub type Dims2 = [usize; 2];
pub type Dims3 = [usize; 3];
pub type Dims4 = [usize; 4];

impl<T: Clone, const N: usize> MultiDim<T, N> {
    pub fn filled(dims: [usize; N], value: T) -> Option<Self> {
        let (strides, len) = Self::build_strides_and_len(dims)?;
        Some(Self {
            dims,
            strides,
            data: vec![value; len],
        })
    }
}

impl<T, const N: usize> MultiDim<T, N> {
    pub fn from_vec(dims: [usize; N], data: Vec<T>) -> Option<Self> {
        let (strides, len) = Self::build_strides_and_len(dims)?;
        if data.len() != len {
            return None;
        }
        Some(Self {
            dims,
            strides,
            data,
        })
    }

    pub fn dims(&self) -> [usize; N] {
        self.dims
    }

    pub fn strides(&self) -> [usize; N] {
        self.strides
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

    pub fn index_of(&self, coords: [usize; N]) -> Option<usize> {
        let mut index = 0usize;
        for axis in 0..N {
            if coords[axis] >= self.dims[axis] {
                return None;
            }
            index = index.saturating_add(coords[axis].saturating_mul(self.strides[axis]));
        }
        Some(index)
    }

    pub fn coords_of(&self, index: usize) -> Option<[usize; N]> {
        if index >= self.data.len() {
            return None;
        }

        let mut remaining = index;
        let mut coords = [0usize; N];
        for axis in 0..N {
            let stride = self.strides[axis];
            if stride == 0 {
                return None;
            }
            coords[axis] = remaining / stride;
            remaining %= stride;
        }
        Some(coords)
    }

    pub fn get(&self, coords: [usize; N]) -> Option<&T> {
        self.index_of(coords).and_then(|idx| self.data.get(idx))
    }

    pub fn get_mut(&mut self, coords: [usize; N]) -> Option<&mut T> {
        self.index_of(coords).and_then(|idx| self.data.get_mut(idx))
    }

    pub fn set(&mut self, coords: [usize; N], value: T) -> bool {
        if let Some(idx) = self.index_of(coords) {
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

    fn build_strides_and_len(dims: [usize; N]) -> Option<([usize; N], usize)> {
        if N == 0 || dims.contains(&0) {
            return None;
        }

        let mut strides = [0usize; N];
        strides[N - 1] = 1;

        for axis in (0..N - 1).rev() {
            strides[axis] = strides[axis + 1].checked_mul(dims[axis + 1])?;
        }

        let len = dims
            .iter()
            .try_fold(1usize, |acc, dim| acc.checked_mul(*dim))?;

        Some((strides, len))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn index_roundtrip_3d() {
        let grid = MultiDim3::<i32>::filled([3, 4, 5], 0).unwrap();
        let idx = grid.index_of([2, 1, 3]).unwrap();
        assert_eq!(idx, 48);
        assert_eq!(grid.coords_of(idx), Some([2, 1, 3]));
    }

    #[test]
    fn get_set_works() {
        let mut grid = MultiDim2::<i32>::filled([2, 3], 1).unwrap();
        assert_eq!(grid.get([1, 2]), Some(&1));
        assert!(grid.set([1, 2], 99));
        assert_eq!(grid.get([1, 2]), Some(&99));
        assert!(!grid.set([2, 0], 3));
    }

    #[test]
    fn from_vec_validates_size() {
        let ok = MultiDim2::from_vec([2, 2], vec![1, 2, 3, 4]);
        let bad = MultiDim2::from_vec([2, 2], vec![1, 2, 3]);
        assert!(ok.is_some());
        assert!(bad.is_none());
    }
}
