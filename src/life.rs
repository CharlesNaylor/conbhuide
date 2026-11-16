/*
 * Life-like cellular automaton on a bounded grid.
 *
 * The rule is a pair of bitmasks (bit n set = "applies with n live neighbours"), so any
 * B/S rule can be used. Drawing of the raw cells lives here; the knot view is in knot.rs.
 */
use macroquad::prelude::*;
use macroquad::rand::gen_range;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rule {
    pub name: &'static str,
    pub birth: u16,
    pub survive: u16,
    /// fraction of live cells that makes a good random start for this rule
    pub seed_density: f32,
}

const fn mask(counts: &[u8]) -> u16 {
    let mut m = 0u16;
    let mut i = 0;
    while i < counts.len() {
        m |= 1 << counts[i];
        i += 1;
    }
    m
}

pub const RULES: [Rule; 4] = [
    Rule {
        name: "Life B3/S23",
        birth: mask(&[3]),
        survive: mask(&[2, 3]),
        seed_density: 0.3,
    },
    Rule {
        name: "Maze B3/S12345",
        birth: mask(&[3]),
        survive: mask(&[1, 2, 3, 4, 5]),
        seed_density: 0.15,
    },
    Rule {
        name: "Day & Night B3678/S34678",
        birth: mask(&[3, 6, 7, 8]),
        survive: mask(&[3, 4, 6, 7, 8]),
        seed_density: 0.5,
    },
    Rule {
        name: "Coral B3/S45678",
        birth: mask(&[3]),
        survive: mask(&[4, 5, 6, 7, 8]),
        seed_density: 0.4,
    },
];

pub struct CellMatrix {
    pub width: u16,
    pub height: u16,
    pub cell_size: u16,
    pub frame_top_left: Vec2,
    pub rule: Rule,
    cells: Vec<bool>,
}
impl CellMatrix {
    pub fn new(screen_size: Vec2, cell_size: u16, frame_top_left: Option<Vec2>) -> Self {
        let width: u16 = (screen_size.x / cell_size as f32) as u16;
        let height: u16 = (screen_size.y / cell_size as f32) as u16;
        CellMatrix {
            width,
            height,
            cell_size,
            frame_top_left: frame_top_left.unwrap_or(vec2(0., 0.)),
            rule: RULES[0],
            cells: vec![false; width as usize * height as usize],
        }
    }

    pub fn cells(&self) -> &[bool] {
        &self.cells
    }

    pub fn clear(&mut self) {
        self.cells.iter_mut().for_each(|c| *c = false);
    }

    pub fn set(&mut self, x: u16, y: u16, alive: bool) {
        if x < self.width && y < self.height {
            let ind = self.ind_for_pos(x, y);
            self.cells[ind] = alive;
        }
    }

    pub fn randomize(&mut self, living_fraction: Option<f32>) {
        /* Add random live cells at rate living_fraction */
        let fraction = living_fraction.unwrap_or(self.rule.seed_density);
        for cell in self.cells.iter_mut() {
            *cell = gen_range(0.0, 1.0) < fraction;
        }
    }

    pub fn randomize_symmetric(&mut self, living_fraction: Option<f32>) {
        /* Randomize one quadrant and mirror it into the other three. The rules are
         * mirror-symmetric, so the pattern stays four-fold symmetric as it evolves. */
        let fraction = living_fraction.unwrap_or(self.rule.seed_density);
        for y in 0..(self.height + 1) / 2 {
            for x in 0..(self.width + 1) / 2 {
                let alive = gen_range(0.0, 1.0) < fraction;
                let (mx, my) = (self.width - 1 - x, self.height - 1 - y);
                self.set(x, y, alive);
                self.set(mx, y, alive);
                self.set(x, my, alive);
                self.set(mx, my, alive);
            }
        }
    }

    pub fn is_alive(&self, x: u16, y: u16) -> bool {
        self.cells[self.ind_for_pos(x, y)]
    }

    fn ind_for_pos(&self, x: u16, y: u16) -> usize {
        /* return cell index for a given x,y coordinate
         * (cells are stored in a 1d vector) */
        y as usize * self.width as usize + x as usize
    }

    pub fn cell_center(&self, x: u16, y: u16) -> Vec2 {
        let s = self.cell_size as f32;
        self.frame_top_left + vec2((x as f32 + 0.5) * s, (y as f32 + 0.5) * s)
    }

    fn cell_pos_for_click(&self, screen_pos: Vec2) -> Option<(u16, u16)> {
        /* translate a click on the screen to a cell position, if it is on the grid */
        let pos = (screen_pos - self.frame_top_left) / self.cell_size as f32;
        if pos.x < 0.0 || pos.y < 0.0 {
            return None;
        }
        let (x, y) = (pos.x as u16, pos.y as u16);
        if x < self.width && y < self.height {
            Some((x, y))
        } else {
            None
        }
    }

    pub fn draw(&self, alive: Color, dead: Option<Color>) {
        /* draw a square per cell; dead cells are skipped when `dead` is None */
        let s = self.cell_size as f32;
        for y in 0..self.height {
            for x in 0..self.width {
                let color = if self.is_alive(x, y) {
                    alive
                } else if let Some(dead) = dead {
                    dead
                } else {
                    continue;
                };
                draw_rectangle(
                    self.frame_top_left.x + x as f32 * s,
                    self.frame_top_left.y + y as f32 * s,
                    s,
                    s,
                    color,
                );
            }
        }
    }

    pub fn flip_cell(&mut self, mouse_position: Vec2) {
        if let Some((x, y)) = self.cell_pos_for_click(mouse_position) {
            let cell_ind = self.ind_for_pos(x, y);
            self.cells[cell_ind] = !self.cells[cell_ind];
        }
    }

    pub fn step(&mut self) {
        /* evolve the matrix one step */
        let mut buffer = self.cells.to_vec();
        for y in 0..self.height as i32 {
            for x in 0..self.width as i32 {
                let mut n_neighbors = 0;
                for j in -1i32..=1 {
                    for i in -1i32..=1 {
                        // out of bounds
                        if y + j < 0
                            || y + j >= self.height as i32
                            || x + i < 0
                            || x + i >= self.width as i32
                        {
                            continue;
                        }
                        // I am not a neighbor of myself
                        if i == 0 && j == 0 {
                            continue;
                        }
                        if self.is_alive((x + i) as u16, (y + j) as u16) {
                            n_neighbors += 1;
                        }
                    }
                }

                let applies = if self.is_alive(x as u16, y as u16) {
                    self.rule.survive
                } else {
                    self.rule.birth
                };
                buffer[self.ind_for_pos(x as u16, y as u16)] = applies & (1 << n_neighbors) != 0;
            }
        }
        self.cells = buffer;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn matrix(width: u16, height: u16) -> CellMatrix {
        CellMatrix::new(vec2(width as f32 * 10.0, height as f32 * 10.0), 10, None)
    }

    fn live(m: &CellMatrix) -> Vec<(u16, u16)> {
        let mut out = vec![];
        for y in 0..m.height {
            for x in 0..m.width {
                if m.is_alive(x, y) {
                    out.push((x, y));
                }
            }
        }
        out
    }

    #[test]
    fn blinker_oscillates_under_life() {
        let mut m = matrix(5, 5);
        for x in 1..=3 {
            m.set(x, 2, true);
        }
        m.step();
        assert_eq!(live(&m), vec![(2, 1), (2, 2), (2, 3)]);
        m.step();
        assert_eq!(live(&m), vec![(1, 2), (2, 2), (3, 2)]);
    }

    #[test]
    fn lone_cell_survives_under_maze_but_not_life() {
        let mut m = matrix(5, 5);
        m.set(1, 1, true);
        m.set(2, 1, true);
        m.step();
        assert!(live(&m).is_empty());

        m.rule = RULES[1];
        m.set(1, 1, true);
        m.set(2, 1, true);
        m.step();
        assert_eq!(live(&m), vec![(1, 1), (2, 1)]);
    }

    #[test]
    fn symmetric_seed_stays_symmetric() {
        let mut m = matrix(9, 6);
        m.randomize_symmetric(Some(0.4));
        for _ in 0..5 {
            m.step();
        }
        for y in 0..m.height {
            for x in 0..m.width {
                let a = m.is_alive(x, y);
                assert_eq!(a, m.is_alive(m.width - 1 - x, y));
                assert_eq!(a, m.is_alive(x, m.height - 1 - y));
            }
        }
    }

    #[test]
    fn clicks_off_the_grid_are_ignored() {
        let mut m = matrix(3, 3);
        m.flip_cell(vec2(500.0, 500.0));
        m.flip_cell(vec2(-5.0, 5.0));
        assert!(live(&m).is_empty());
        m.flip_cell(vec2(15.0, 25.0));
        assert_eq!(live(&m), vec![(1, 2)]);
    }
}
