use std::collections::HashMap;

use crate::grid2d::{Coord2, Grid2D};

#[derive(Clone, Debug, PartialEq)]
pub struct CellDelta<T> {
    pub index: usize,
    pub before: T,
    pub after: T,
}

#[derive(Clone, Debug, PartialEq)]
pub struct HistoryStep<T> {
    pub turn: usize,
    pub deltas: Vec<CellDelta<T>>,
}

#[derive(Clone, Debug, PartialEq)]
struct PendingDelta<T> {
    before: T,
    after: T,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Grid2DHistory<T> {
    grid: Grid2D<T>,
    capacity: usize,
    turn: usize,
    steps: Vec<HistoryStep<T>>,
    redo_steps: Vec<HistoryStep<T>>,
    pending: Option<HashMap<usize, PendingDelta<T>>>,
}

impl<T: Clone + PartialEq> Grid2DHistory<T> {
    pub fn new(initial: Grid2D<T>, capacity: usize) -> Option<Self> {
        if capacity == 0 {
            return None;
        }
        Some(Self {
            grid: initial,
            capacity,
            turn: 0,
            steps: Vec::new(),
            redo_steps: Vec::new(),
            pending: None,
        })
    }

    pub fn begin_step(&mut self) -> bool {
        if self.pending.is_some() {
            return false;
        }
        self.pending = Some(HashMap::new());
        true
    }

    pub fn set(&mut self, x: usize, y: usize, value: T) -> bool {
        let Some(idx) = self.grid.index_of(x, y) else {
            return false;
        };
        let Some(pending) = self.pending.as_mut() else {
            return false;
        };

        let current = match self.grid.get(x, y) {
            Some(v) => v.clone(),
            None => return false,
        };

        if let Some(existing) = pending.get_mut(&idx) {
            if existing.before == value {
                pending.remove(&idx);
                return self.grid.set(x, y, value);
            }
            existing.after = value.clone();
            return self.grid.set(x, y, value);
        }

        if current == value {
            return true;
        }

        pending.insert(
            idx,
            PendingDelta {
                before: current,
                after: value.clone(),
            },
        );
        self.grid.set(x, y, value)
    }

    pub fn commit_step(&mut self) -> bool {
        let Some(pending) = self.pending.take() else {
            return false;
        };
        if pending.is_empty() {
            return false;
        }

        self.turn += 1;
        let mut deltas = Vec::with_capacity(pending.len());
        for (index, delta) in pending {
            deltas.push(CellDelta {
                index,
                before: delta.before,
                after: delta.after,
            });
        }
        deltas.sort_by_key(|d| d.index);

        self.steps.push(HistoryStep {
            turn: self.turn,
            deltas,
        });
        self.redo_steps.clear();

        if self.steps.len() > self.capacity {
            self.steps.remove(0);
        }
        true
    }

    pub fn cancel_step(&mut self) -> bool {
        let Some(pending) = self.pending.take() else {
            return false;
        };
        for (index, delta) in pending {
            if let Some(c) = self.coords_from_index(index) {
                let _ = self.grid.set(c.x, c.y, delta.before);
            }
        }
        true
    }

    pub fn undo(&mut self) -> bool {
        if self.pending.is_some() {
            return false;
        }
        let Some(step) = self.steps.pop() else {
            return false;
        };

        for delta in step.deltas.iter().rev() {
            if let Some(c) = self.coords_from_index(delta.index) {
                let _ = self.grid.set(c.x, c.y, delta.before.clone());
            }
        }
        self.turn = self.turn.saturating_sub(1);
        self.redo_steps.push(step);
        true
    }

    pub fn redo(&mut self) -> bool {
        if self.pending.is_some() {
            return false;
        }
        let Some(step) = self.redo_steps.pop() else {
            return false;
        };
        for delta in &step.deltas {
            if let Some(c) = self.coords_from_index(delta.index) {
                let _ = self.grid.set(c.x, c.y, delta.after.clone());
            }
        }
        self.turn += 1;
        self.steps.push(step);
        true
    }

    pub fn get(&self, x: usize, y: usize) -> Option<&T> {
        self.grid.get(x, y)
    }

    pub fn current_turn(&self) -> usize {
        self.turn
    }

    pub fn step_count(&self) -> usize {
        self.steps.len()
    }

    pub fn as_grid(&self) -> Grid2D<T> {
        self.grid.clone()
    }

    pub fn clear_history(&mut self) -> bool {
        if self.pending.is_some() {
            return false;
        }
        self.steps.clear();
        self.redo_steps.clear();
        self.turn = 0;
        true
    }

    fn coords_from_index(&self, index: usize) -> Option<Coord2> {
        self.grid.coords_of(index)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn undo_redo_delta_history() {
        let grid = Grid2D::filled(4, 4, 0i32).unwrap();
        let mut history = Grid2DHistory::new(grid, 50).unwrap();

        assert!(history.begin_step());
        assert!(history.set(1, 1, 5));
        assert!(history.set(2, 1, 7));
        assert!(history.commit_step());
        assert_eq!(history.get(1, 1), Some(&5));
        assert_eq!(history.step_count(), 1);

        assert!(history.undo());
        assert_eq!(history.get(1, 1), Some(&0));
        assert!(history.redo());
        assert_eq!(history.get(2, 1), Some(&7));
    }

    #[test]
    fn cancel_step_reverts() {
        let grid = Grid2D::filled(2, 2, 0i32).unwrap();
        let mut history = Grid2DHistory::new(grid, 10).unwrap();
        assert!(history.begin_step());
        assert!(history.set(0, 0, 9));
        assert!(history.cancel_step());
        assert_eq!(history.get(0, 0), Some(&0));
    }
}

