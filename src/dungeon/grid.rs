use std::collections::VecDeque;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Cell {
    Wall,
    Floor,
    Door,
    /// Leads to the next, deeper level.
    Stairs,
}

impl Cell {
    /// Anything that is not a wall can be walked on (a closed door opens first).
    pub fn walkable(self) -> bool {
        self != Cell::Wall
    }
}

/// One change to the grid, in square coordinates. Carvers produce a list of
/// these; generation applies them in order.
#[derive(Clone, Copy, Debug)]
pub struct Step {
    pub x: usize,
    pub y: usize,
    pub cell: Cell,
}

#[derive(Clone)]
pub struct Grid {
    width: usize,
    height: usize,
    cells: Vec<Cell>,
}

impl Grid {
    /// Takes the size in corridors; the square grid is 2n+1 in each
    /// direction so that walls occupy full squares.
    pub fn new(corridors_x: usize, corridors_y: usize) -> Self {
        let width = corridors_x * 2 + 1;
        let height = corridors_y * 2 + 1;
        Self {
            width,
            height,
            cells: vec![Cell::Wall; width * height],
        }
    }

    pub fn apply(&mut self, step: Step) {
        self.set(step.x, step.y, step.cell);
    }

    /// Breadth-first distances over walkable squares; `usize::MAX` where
    /// unreachable.
    pub fn distances_from(&self, start: (usize, usize)) -> Vec<usize> {
        let mut distance = vec![usize::MAX; self.cells.len()];
        let mut queue = VecDeque::new();
        distance[self.index(start.0, start.1)] = 0;
        queue.push_back(start);
        while let Some((x, y)) = queue.pop_front() {
            let next_distance = distance[self.index(x, y)] + 1;
            for (nx, ny) in neighbours(x, y, self.width, self.height) {
                let index = self.index(nx, ny);
                if self.walkable(nx, ny) && distance[index] == usize::MAX {
                    distance[index] = next_distance;
                    queue.push_back((nx, ny));
                }
            }
        }
        distance
    }

    /// (One of) the walkable squares farthest from `start`.
    pub fn farthest_walkable_from(&self, start: (usize, usize)) -> (usize, usize) {
        let distance = self.distances_from(start);
        let mut farthest = (start, 0);
        for y in 0..self.height {
            for x in 0..self.width {
                let d = distance[self.index(x, y)];
                if d != usize::MAX && d > farthest.1 {
                    farthest = ((x, y), d);
                }
            }
        }
        farthest.0
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn index(&self, x: usize, y: usize) -> usize {
        y * self.width + x
    }

    pub fn walkable(&self, x: usize, y: usize) -> bool {
        self.get(x, y).walkable()
    }

    pub fn get(&self, x: usize, y: usize) -> Cell {
        self.cells[self.index(x, y)]
    }

    pub fn set(&mut self, x: usize, y: usize, cell: Cell) {
        let i = self.index(x, y);
        self.cells[i] = cell;
    }
}

/// The up-to-four orthogonal neighbours of (x, y) inside a width x height grid.
pub fn neighbours(x: usize, y: usize, width: usize, height: usize) -> Vec<(usize, usize)> {
    let mut out = Vec::with_capacity(4);
    if x > 0 {
        out.push((x - 1, y));
    }
    if y > 0 {
        out.push((x, y - 1));
    }
    if x + 1 < width {
        out.push((x + 1, y));
    }
    if y + 1 < height {
        out.push((x, y + 1));
    }
    out
}
