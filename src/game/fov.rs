use crate::dungeon::grid::Grid;

/// Squares visible from `origin` within `radius`: a square is seen when the
/// straight line to it crosses nothing that blocks sight. The blocking square
/// itself is seen, so walls and closed doors light up at the edge of view.
pub fn compute(grid: &Grid, origin: (usize, usize), radius: usize) -> Vec<bool> {
    let (w, h) = (grid.width(), grid.height());
    let mut visible = vec![false; w * h];
    let (ox, oy) = (origin.0 as isize, origin.1 as isize);
    let r = radius as isize;

    for ty in (oy - r).max(0)..=(oy + r).min(h as isize - 1) {
        for tx in (ox - r).max(0)..=(ox + r).min(w as isize - 1) {
            let (dx, dy) = (tx - ox, ty - oy);
            if dx * dx + dy * dy > r * r {
                continue;
            }
            if line_is_clear(grid, (ox, oy), (tx, ty)) {
                visible[grid.index(tx as usize, ty as usize)] = true;
            }
        }
    }
    visible
}

/// Bresenham from `from` to `to`; true when no square strictly between the
/// two ends blocks sight.
fn line_is_clear(grid: &Grid, from: (isize, isize), to: (isize, isize)) -> bool {
    let (mut x, mut y) = from;
    let dx = (to.0 - x).abs();
    let dy = -(to.1 - y).abs();
    let sx = if x < to.0 { 1 } else { -1 };
    let sy = if y < to.1 { 1 } else { -1 };
    let mut err = dx + dy;
    loop {
        if (x, y) == to {
            return true;
        }
        if (x, y) != from && grid.get(x as usize, y as usize).blocks_sight() {
            return false;
        }
        let e2 = 2 * err;
        if e2 >= dy {
            err += dy;
            x += sx;
        }
        if e2 <= dx {
            err += dx;
            y += sy;
        }
    }
}
