/*
 * Celtic knot drawn around the live cells of a grid (Mercat's construction).
 *
 * Every live cell is a graph vertex at its cell centre, and two live cells that share a side
 * are joined by an edge. Each edge carries one strand crossing at its midpoint. Where a live
 * cell faces a dead cell (or the border of the grid) the strand turns back around the cell.
 *
 * With s = cell size, C = cell centre:
 *   - live neighbour: crossing at the shared side's midpoint P, made of the two diagonal
 *     segments P +/- (s/4, s/4) and P +/- (s/4, -s/4)
 *   - dead neighbour: quarter arc centred on C, radius s * sqrt(2) / 4, facing that side
 *
 * Arcs and crossings end at the same points with the same diagonal tangent, so the strands
 * join up whatever the configuration: every grid state is a valid knot.
 *
 * Over/under: on a horizontal edge the NW-SE strand is on top, on a vertical edge the NE-SW
 * strand is. Following any strand, edge orientation alternates, so over/under does too.
 *
 * Animation. Seen from C, each side of a live cell is a curve r(phi) over the 90 degrees
 * facing that side: constant for an arc, and rising to a point at P for half a crossing.
 * Between two states r is interpolated, with r = 0 for a dead cell. So a strand that is
 * about to cross pushes a tip out to P, and a cell being born grows out of its centre.
 */
use crate::life::CellMatrix;
use crate::mercat::{draw_ribbon, gap_near_crossing, Curve};
use macroquad::prelude::*;
use std::f32::consts::{FRAC_PI_2, FRAC_PI_4, SQRT_2};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Side {
    /// the cell is dead: nothing to draw
    Empty,
    /// live neighbour: strands cross on the shared side
    Cross,
    /// dead neighbour or grid border: the strand turns back
    Wall,
}

/// Neighbour offsets in the order East, South, West, North (y grows downwards).
/// Side k faces the screen angle k * 90 degrees.
pub const DIRS: [(i32, i32); 4] = [(1, 0), (0, 1), (-1, 0), (0, -1)];

pub fn sides_for_cell(cells: &[bool], width: u16, height: u16, x: u16, y: u16) -> [Side; 4] {
    let alive = |x: i32, y: i32| {
        x >= 0
            && y >= 0
            && x < width as i32
            && y < height as i32
            && cells[y as usize * width as usize + x as usize]
    };
    let mut sides = [Side::Empty; 4];
    if !alive(x as i32, y as i32) {
        return sides;
    }
    for (side, (dx, dy)) in sides.iter_mut().zip(DIRS) {
        *side = if alive(x as i32 + dx, y as i32 + dy) {
            Side::Cross
        } else {
            Side::Wall
        };
    }
    sides
}

pub struct KnotStyle {
    /// strand width as a fraction of the cell size
    pub strand_width: f32,
    /// clear space either side of the upper strand at a crossing, as a fraction of cell size
    pub gap: f32,
    pub strand_color: Color,
    pub background: Color,
}
impl Default for KnotStyle {
    fn default() -> Self {
        KnotStyle {
            strand_width: 0.3,
            gap: 0.07,
            strand_color: Color::new(1.0, 0.62, 0.2, 1.0),
            background: BLACK,
        }
    }
}

/// How far each piece of strand is extended past its nominal end, in pixels, so that
/// neighbouring pieces overlap instead of leaving an antialiased seam.
const OVERLAP: f32 = 0.5;

fn draw_segment(a: Vec2, b: Vec2, width: f32, color: Color) {
    let ext = (b - a).normalize() * OVERLAP;
    let (a, b) = (a - ext, b + ext);
    draw_line(a.x, a.y, b.x, b.y, width, color);
}

fn draw_quarter_arc(center: Vec2, radius: f32, mid_angle: f32, width: f32, color: Color) {
    const STEPS: usize = 12;
    let overlap = OVERLAP / radius;
    let start = mid_angle - FRAC_PI_4 - overlap;
    let sweep = FRAC_PI_2 + 2.0 * overlap;
    let (r_in, r_out) = (radius - width / 2.0, radius + width / 2.0);
    let mut prev: Option<(Vec2, Vec2)> = None;
    for i in 0..=STEPS {
        let angle = start + sweep * i as f32 / STEPS as f32;
        let dir = vec2(angle.cos(), angle.sin());
        let (p_in, p_out) = (center + dir * r_in, center + dir * r_out);
        if let Some((q_in, q_out)) = prev {
            draw_triangle(q_in, q_out, p_out, color);
            draw_triangle(q_in, p_out, p_in, color);
        }
        prev = Some((p_in, p_out));
    }
}

fn draw_crossing(p: Vec2, quarter: f32, horizontal_edge: bool, width: f32, gap: f32, style: &KnotStyle) {
    let nw_se = vec2(quarter, quarter);
    let ne_sw = vec2(quarter, -quarter);
    let (over, under) = if horizontal_edge {
        (nw_se, ne_sw)
    } else {
        (ne_sw, nw_se)
    };
    draw_segment(p - under, p + under, width, style.strand_color);
    // cut the lower strand back either side of the upper one
    let along = over.normalize() * (width / 2.0 + 1.0);
    draw_line(
        p.x - along.x,
        p.y - along.y,
        p.x + along.x,
        p.y + along.y,
        width + 2.0 * gap,
        style.background,
    );
    draw_segment(p - over, p + over, width, style.strand_color);
}

/// Draw side `k` of the live cell centred on `center` as it stands.
fn draw_side(center: Vec2, s: f32, k: usize, side: Side, style: &KnotStyle) {
    let width = s * style.strand_width;
    match side {
        Side::Empty => {}
        Side::Wall => {
            draw_quarter_arc(center, s * SQRT_2 / 4.0, k as f32 * FRAC_PI_2, width, style.strand_color);
        }
        Side::Cross => {
            // each edge is shared by two cells: draw it from the west/north one
            if k < 2 {
                let (dx, dy) = DIRS[k];
                let p = center + vec2(dx as f32, dy as f32) * (s / 2.0);
                draw_crossing(p, s / 4.0, k == 0, width, s * style.gap, style);
            }
        }
    }
}

pub fn draw_knot(matrix: &CellMatrix, style: &KnotStyle) {
    let s = matrix.cell_size as f32;
    for y in 0..matrix.height {
        for x in 0..matrix.width {
            let sides = sides_for_cell(matrix.cells(), matrix.width, matrix.height, x, y);
            let center = matrix.cell_center(x, y);
            for (k, side) in sides.iter().enumerate() {
                draw_side(center, s, k, *side, style);
            }
        }
    }
}

/// Distance of the strand from the cell centre, `phi` radians round from the middle of a side.
fn side_radius(side: Side, s: f32, phi: f32) -> f32 {
    match side {
        Side::Empty => 0.0,
        Side::Wall => s * SQRT_2 / 4.0,
        Side::Cross => (s / 2.0) / (phi.cos() + phi.abs().sin()),
    }
}

/// Side `k` of a cell, part of the way (`t`) from being `from` to being `to`.
fn draw_side_morph(center: Vec2, s: f32, k: usize, from: Side, to: Side, t: f32, width: f32, color: Color) {
    const STEPS: usize = 8;
    let mid_angle = k as f32 * FRAC_PI_2;
    let point = |phi: f32| {
        let (r0, r1) = (side_radius(from, s, phi), side_radius(to, s, phi));
        let angle = mid_angle + phi;
        center + vec2(angle.cos(), angle.sin()) * (r0 + (r1 - r0) * t)
    };
    let strand = |start: f32, end: f32| {
        let pts = (0..=STEPS)
            .map(|i| point(start + (end - start) * i as f32 / STEPS as f32))
            .collect();
        let curve = Curve::through(pts, None, None);
        draw_ribbon(&curve, curve.pts.len(), width, color);
    };
    if from == Side::Cross || to == Side::Cross {
        // the strand comes to a point in the middle of the side: round it off
        strand(-FRAC_PI_4, 0.0);
        strand(0.0, FRAC_PI_4);
        let tip = point(0.0);
        draw_circle(tip.x, tip.y, width / 2.0, color);
    } else {
        strand(-FRAC_PI_4, FRAC_PI_4);
    }
}

/// The knot part of the way from the cells `previous` (at `t` = 0) to the matrix's current
/// cells (at `t` = 1).
pub fn draw_knot_morph(matrix: &CellMatrix, previous: &[bool], t: f32, style: &KnotStyle) {
    let s = matrix.cell_size as f32;
    let sides = |cells: &[bool], x: u16, y: u16| sides_for_cell(cells, matrix.width, matrix.height, x, y);
    // a cell being born or dying is also drawn thinner
    let width = |x: u16, y: u16| {
        s * style.strand_width
            * match (sides(previous, x, y)[0], sides(matrix.cells(), x, y)[0]) {
                (Side::Empty, _) => t,
                (_, Side::Empty) => 1.0 - t,
                _ => 1.0,
            }
    };
    for y in 0..matrix.height {
        for x in 0..matrix.width {
            let (from, to) = (sides(previous, x, y), sides(matrix.cells(), x, y));
            let center = matrix.cell_center(x, y);
            for k in 0..4 {
                if from[k] == to[k] {
                    draw_side(center, s, k, to[k], style);
                } else {
                    draw_side_morph(center, s, k, from[k], to[k], t, width(x, y), style.strand_color);
                }
            }
        }
    }
    // Where a crossing is forming or coming apart, the two tips are all but touching near
    // that end of the move: lay the crossing over them there, with its gap opening or closing.
    for y in 0..matrix.height {
        for x in 0..matrix.width {
            let (from, to) = (sides(previous, x, y), sides(matrix.cells(), x, y));
            for k in 0..2 {
                let gap = match (from[k], to[k]) {
                    (Side::Cross, Side::Cross) => 0.0,
                    (Side::Cross, _) => gap_near_crossing(1.0 - t),
                    (_, Side::Cross) => gap_near_crossing(t),
                    _ => 0.0,
                };
                if gap > 0.0 {
                    let (dx, dy) = DIRS[k];
                    let p = matrix.cell_center(x, y) + vec2(dx as f32, dy as f32) * (s / 2.0);
                    let (nx, ny) = (x + dx as u16, y + dy as u16);
                    let width = width(x, y).min(width(nx, ny));
                    draw_crossing(p, s / 4.0, k == 0, width, s * style.gap * gap, style);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn grid(width: u16, height: u16, live: &[(u16, u16)]) -> Vec<bool> {
        let mut cells = vec![false; width as usize * height as usize];
        for &(x, y) in live {
            cells[y as usize * width as usize + x as usize] = true;
        }
        cells
    }

    fn count(cells: &[bool], width: u16, height: u16, wanted: Side) -> usize {
        let mut n = 0;
        for y in 0..height {
            for x in 0..width {
                n += sides_for_cell(cells, width, height, x, y)
                    .iter()
                    .filter(|s| **s == wanted)
                    .count();
            }
        }
        n
    }

    #[test]
    fn dead_cell_draws_nothing() {
        let cells = grid(3, 3, &[(0, 0)]);
        assert_eq!(sides_for_cell(&cells, 3, 3, 1, 1), [Side::Empty; 4]);
    }

    #[test]
    fn lone_cell_is_a_ring() {
        let cells = grid(3, 3, &[(1, 1)]);
        assert_eq!(sides_for_cell(&cells, 3, 3, 1, 1), [Side::Wall; 4]);
    }

    #[test]
    fn horizontal_pair_crosses_once() {
        let cells = grid(4, 3, &[(1, 1), (2, 1)]);
        // E, S, W, N
        assert_eq!(
            sides_for_cell(&cells, 4, 3, 1, 1),
            [Side::Cross, Side::Wall, Side::Wall, Side::Wall]
        );
        assert_eq!(
            sides_for_cell(&cells, 4, 3, 2, 1),
            [Side::Wall, Side::Wall, Side::Cross, Side::Wall]
        );
        // each edge is seen from both of its cells
        assert_eq!(count(&cells, 4, 3, Side::Cross), 2);
        assert_eq!(count(&cells, 4, 3, Side::Wall), 6);
    }

    #[test]
    fn block_has_four_edges() {
        let cells = grid(4, 4, &[(1, 1), (2, 1), (1, 2), (2, 2)]);
        assert_eq!(count(&cells, 4, 4, Side::Cross), 8);
        assert_eq!(count(&cells, 4, 4, Side::Wall), 8);
    }

    #[test]
    fn diagonal_neighbours_are_not_joined() {
        let cells = grid(3, 3, &[(0, 0), (1, 1)]);
        assert_eq!(count(&cells, 3, 3, Side::Cross), 0);
    }

    #[test]
    fn grid_border_is_a_wall() {
        let cells = grid(2, 1, &[(0, 0), (1, 0)]);
        assert_eq!(
            sides_for_cell(&cells, 2, 1, 1, 0),
            [Side::Wall, Side::Wall, Side::Cross, Side::Wall]
        );
    }

    #[test]
    fn arcs_and_crossings_meet_at_the_corners_of_a_side() {
        let s = 24.0;
        for phi in [-FRAC_PI_4, FRAC_PI_4] {
            let (wall, cross) = (side_radius(Side::Wall, s, phi), side_radius(Side::Cross, s, phi));
            assert!((wall - cross).abs() < 1e-4);
        }
        // and a crossing reaches the middle of the side
        assert!((side_radius(Side::Cross, s, 0.0) - s / 2.0).abs() < 1e-4);
    }
}
