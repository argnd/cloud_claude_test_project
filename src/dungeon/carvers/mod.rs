mod backtracker;
mod kruskal;
mod prim;

use rand::Rng;

use super::grid::{Cell, Step, neighbours};

/// The corridor lattice a carver works on: its size in corridors and which
/// corridors are off-limits (inside rooms). Carvers work in corridor
/// coordinates only; `floor` and `wall_between` translate to squares.
pub struct Lattice {
    width: usize,
    height: usize,
    blocked: Vec<bool>,
}

impl Lattice {
    pub fn open(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            blocked: vec![false; width * height],
        }
    }

    fn len(&self) -> usize {
        self.width * self.height
    }

    fn index(&self, x: usize, y: usize) -> usize {
        y * self.width + x
    }

    pub fn block(&mut self, x: usize, y: usize) {
        let i = self.index(x, y);
        self.blocked[i] = true;
    }

    fn is_blocked(&self, x: usize, y: usize) -> bool {
        self.blocked[self.index(x, y)]
    }

    /// Every corridor that is not blocked, row by row.
    fn corridors(&self) -> Vec<(usize, usize)> {
        let mut out = Vec::with_capacity(self.len());
        for y in 0..self.height {
            for x in 0..self.width {
                if !self.is_blocked(x, y) {
                    out.push((x, y));
                }
            }
        }
        out
    }

    /// Orthogonal neighbours that are not blocked.
    fn neighbours(&self, x: usize, y: usize) -> Vec<(usize, usize)> {
        neighbours(x, y, self.width, self.height)
            .into_iter()
            .filter(|&(nx, ny)| !self.is_blocked(nx, ny))
            .collect()
    }
}

/// The corridor carvers the player can choose from; each one gives the
/// corridors between rooms a different feel.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Algorithm {
    Backtracker,
    Prim,
    Kruskal,
}

impl Algorithm {
    #[cfg(test)]
    pub const ALL: [Algorithm; 3] = [Algorithm::Backtracker, Algorithm::Prim, Algorithm::Kruskal];


    pub fn carve(self, lattice: &Lattice, rng: &mut impl Rng) -> Vec<Step> {
        match self {
            Algorithm::Backtracker => backtracker::carve(lattice, rng),
            Algorithm::Prim => prim::carve(lattice, rng),
            Algorithm::Kruskal => kruskal::carve(lattice, rng),
        }
    }
}

/// The floor step for one corridor, in square coordinates.
fn floor(corridor_x: usize, corridor_y: usize) -> Step {
    Step {
        x: corridor_x * 2 + 1,
        y: corridor_y * 2 + 1,
        cell: Cell::Floor,
    }
}

/// The wall square between two adjacent corridors is their midpoint.
fn wall_between(ax: usize, ay: usize, bx: usize, by: usize) -> Step {
    Step {
        x: ax + bx + 1,
        y: ay + by + 1,
        cell: Cell::Floor,
    }
}
