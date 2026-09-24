//! Level generation: rooms, corridors carved around them by a maze
//! algorithm, doors joining everything into one, dead ends filled back in.
//! Adapted from the dungeon mode of the rust_maze project.

pub mod carvers;
mod disjoint_sets;
pub mod grid;

use rand::seq::IndexedRandom;
use rand::{Rng, RngExt};

use carvers::{Algorithm, Lattice};
use disjoint_sets::DisjointSets;
use grid::{Cell, Grid, Step, neighbours};

/// A room in corridor coordinates: corridors x..x+width by y..y+height.
#[derive(Clone, Copy, Debug)]
pub struct Room {
    x: usize,
    y: usize,
    width: usize,
    height: usize,
}

impl Room {
    /// True when the rooms touch or overlap, counting one corridor of gap so
    /// that a carved corridor always separates two rooms.
    fn too_close(&self, other: &Room) -> bool {
        self.x < other.x + other.width + 1
            && other.x < self.x + self.width + 1
            && self.y < other.y + other.height + 1
            && other.y < self.y + self.height + 1
    }

    /// The square coordinates of the room's corners, inclusive.
    fn bounds(&self) -> (usize, usize, usize, usize) {
        (
            self.x * 2 + 1,
            self.y * 2 + 1,
            (self.x + self.width - 1) * 2 + 1,
            (self.y + self.height - 1) * 2 + 1,
        )
    }

    /// Every square the room covers, the walls between its corridors included.
    pub fn squares(&self) -> Vec<(usize, usize)> {
        let (x0, y0, x1, y1) = self.bounds();
        let mut out = Vec::new();
        for y in y0..=y1 {
            for x in x0..=x1 {
                out.push((x, y));
            }
        }
        out
    }

    pub fn centre(&self) -> (usize, usize) {
        let (x0, y0, x1, y1) = self.bounds();
        ((x0 + x1) / 2, (y0 + y1) / 2)
    }
}

/// A finished level. The player starts in the first room; the stairs are on
/// the square farthest from the start.
pub struct Dungeon {
    pub grid: Grid,
    pub rooms: Vec<Room>,
    pub start: (usize, usize),
}

/// Rooms first, then the chosen algorithm fills the space around them with
/// corridors, then doors join every region into one, then corridors that
/// lead nowhere are filled back in.
pub fn generate(
    algorithm: Algorithm,
    corridors_x: usize,
    corridors_y: usize,
    rng: &mut impl Rng,
) -> Dungeon {
    let mut grid = Grid::new(corridors_x, corridors_y);
    let mut lattice = Lattice::open(corridors_x, corridors_y);

    let rooms = place_rooms(corridors_x, corridors_y, rng);
    for room in &rooms {
        for y in room.y..room.y + room.height {
            for x in room.x..room.x + room.width {
                lattice.block(x, y);
            }
        }
        for (x, y) in room.squares() {
            grid.set(x, y, Cell::Floor);
        }
    }

    for step in algorithm.carve(&lattice, rng) {
        grid.apply(step);
    }
    connect_regions(&mut grid, rng);
    prune_dead_ends(&mut grid, &rooms);

    let start = rooms.first().map(Room::centre).unwrap_or((1, 1));
    let stairs = grid.farthest_walkable_from(start);
    grid.set(stairs.0, stairs.1, Cell::Stairs);

    Dungeon { grid, rooms, start }
}

/// Random rectangles, kept only when they don't come too close to an
/// earlier one. The number of attempts scales with the area.
fn place_rooms(corridors_x: usize, corridors_y: usize, rng: &mut impl Rng) -> Vec<Room> {
    let max_side = (corridors_x.min(corridors_y) / 2).clamp(2, 4);
    let attempts = corridors_x * corridors_y / 4;
    let mut rooms: Vec<Room> = Vec::new();
    for _ in 0..attempts {
        let width = rng.random_range(2..=max_side);
        let height = rng.random_range(2..=max_side);
        let room = Room {
            x: rng.random_range(0..=corridors_x - width),
            y: rng.random_range(0..=corridors_y - height),
            width,
            height,
        };
        if rooms.iter().all(|other| !room.too_close(other)) {
            rooms.push(room);
        }
    }
    rooms
}

const NO_REGION: usize = usize::MAX;

/// Labels every walkable square with the number of its connected region.
fn label_regions(grid: &Grid) -> (Vec<usize>, usize) {
    let (w, h) = (grid.width(), grid.height());
    let mut regions = vec![NO_REGION; w * h];
    let mut count = 0;
    for y in 0..h {
        for x in 0..w {
            if !grid.walkable(x, y) || regions[y * w + x] != NO_REGION {
                continue;
            }
            regions[y * w + x] = count;
            let mut stack = vec![(x, y)];
            while let Some((cx, cy)) = stack.pop() {
                for (nx, ny) in neighbours(cx, cy, w, h) {
                    if grid.walkable(nx, ny) && regions[ny * w + nx] == NO_REGION {
                        regions[ny * w + nx] = count;
                        stack.push((nx, ny));
                    }
                }
            }
            count += 1;
        }
    }
    (regions, count)
}

/// A connector is a wall square with walkable squares of two different
/// regions on opposite sides. Random connectors are opened as doors, the
/// ones made redundant dropped, until every region is joined into one.
fn connect_regions(grid: &mut Grid, rng: &mut impl Rng) {
    let (w, h) = (grid.width(), grid.height());
    let (regions, count) = label_regions(grid);
    let region_at = |x: usize, y: usize| regions[y * w + x];

    let mut connectors = Vec::new();
    for y in 1..h - 1 {
        for x in 1..w - 1 {
            if grid.walkable(x, y) {
                continue;
            }
            for ((ax, ay), (bx, by)) in [((x - 1, y), (x + 1, y)), ((x, y - 1), (x, y + 1))] {
                let (a, b) = (region_at(ax, ay), region_at(bx, by));
                if a != NO_REGION && b != NO_REGION && a != b {
                    connectors.push(((x, y), a, b));
                }
            }
        }
    }

    let mut joined = DisjointSets::new(count);
    loop {
        connectors.retain(|&(_, a, b)| joined.find(a) != joined.find(b));
        let Some(&((x, y), a, b)) = connectors.choose(rng) else {
            break;
        };
        joined.merge(a, b);
        grid.apply(Step {
            x,
            y,
            cell: Cell::Door,
        });
    }
}

/// Fills back every walkable square outside the rooms that has at most one
/// walkable neighbour, round after round, until none is left. Corridors
/// that don't join two doors vanish entirely, doors to nowhere with them.
fn prune_dead_ends(grid: &mut Grid, rooms: &[Room]) {
    let (w, h) = (grid.width(), grid.height());
    let mut in_room = vec![false; w * h];
    for room in rooms {
        for (x, y) in room.squares() {
            in_room[y * w + x] = true;
        }
    }

    loop {
        let mut dead_ends = Vec::new();
        for y in 0..h {
            for x in 0..w {
                if !grid.walkable(x, y) || in_room[y * w + x] {
                    continue;
                }
                let open = neighbours(x, y, w, h)
                    .into_iter()
                    .filter(|&(nx, ny)| grid.walkable(nx, ny))
                    .count();
                if open <= 1 {
                    dead_ends.push((x, y));
                }
            }
        }
        if dead_ends.is_empty() {
            break;
        }
        for (x, y) in dead_ends {
            grid.set(x, y, Cell::Wall);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SIZES: [(usize, usize); 3] = [(10, 8), (20, 15), (40, 30)];

    fn walkable_count(grid: &Grid) -> usize {
        let mut count = 0;
        for y in 0..grid.height() {
            for x in 0..grid.width() {
                if grid.walkable(x, y) {
                    count += 1;
                }
            }
        }
        count
    }

    #[test]
    fn every_walkable_square_is_reachable_from_the_start() {
        for algorithm in Algorithm::ALL {
            for (cx, cy) in SIZES {
                let dungeon = generate(algorithm, cx, cy, &mut rand::rng());
                let distances = dungeon.grid.distances_from(dungeon.start);
                let reachable = distances.iter().filter(|&&d| d != usize::MAX).count();
                assert_eq!(
                    reachable,
                    walkable_count(&dungeon.grid),
                    "{algorithm:?} {cx}x{cy}"
                );
            }
        }
    }

    #[test]
    fn stairs_are_away_from_the_start() {
        for algorithm in Algorithm::ALL {
            let dungeon = generate(algorithm, 20, 15, &mut rand::rng());
            let stairs = dungeon.grid.farthest_walkable_from(dungeon.start);
            assert_eq!(dungeon.grid.get(stairs.0, stairs.1), Cell::Stairs);
            assert_ne!(stairs, dungeon.start);
            assert!(!dungeon.rooms.is_empty());
        }
    }
}
