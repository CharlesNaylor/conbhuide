/*
 * Conway's game of life basic implementation. macroquad drawing is in here at the moment, too.
 */
use macroquad::prelude::*;
use macroquad::rand::gen_range;
pub struct CellMatrix {
    pub width: u16,
    pub height: u16,
    cell_size: u16,
    screen_size: Vec2,
    cells: Vec<bool>,
}
impl CellMatrix {
    pub fn new(screen_size: Vec2, cell_size: u16) -> Self {
        let width: u16 = (screen_size.x / cell_size as f32) as u16;
        let height: u16 = (screen_size.y / cell_size as f32) as u16;
        CellMatrix {
            width,
            height,
            cell_size,
            screen_size,
            cells: vec![false; (width * height) as usize],
        }
    }

    pub fn randomize(&mut self, living_fraction: Option<f32>) {
        /* Add random live cells at rate living_fraction */
        for i in 0..self.cells.len() {
            self.cells[i] = gen_range(0, (1.0 / living_fraction.unwrap_or(0.2)) as i32) == 0;
        }
    }

    fn cell_is_alive(&self, x: u16, y: u16) -> bool {
        self.cells[self.ind_for_pos(x, y)]
    }

    fn ind_for_pos(&self, x: u16, y: u16) -> usize {
        /* return cell index for a given x,y coordinate
         * (cells are stored in a 1d vector) */
        (y * self.width) as usize + x as usize
    }

    fn cell_pos_for_click(&self, screen_pos: Vec2) -> (u16, u16) {
        /* translate a click on the screen to a cell position */
        info!("Screen position {},{}", screen_pos.x, screen_pos.y,);
        (
            screen_pos.x as u16 / self.cell_size,
            screen_pos.y as u16 / self.cell_size,
        )
    }

    pub fn draw(&self) {
        for y in 0..self.height as u16 {
            for x in 0..self.width as u16 {
                self.draw_cell(x, y);
            }
        }
    }

    fn draw_cell(&self, x: u16, y: u16) {
        /* draw a rectangle for a given cell reference at the appropriate place in the image*/
        draw_rectangle(
            (x * self.cell_size).into(),
            (y * self.cell_size).into(),
            self.cell_size.into(),
            self.cell_size.into(),
            if self.cell_is_alive(x, y) {
                BLACK
            } else {
                WHITE
            },
        );
    }

    pub fn flip_cell(&mut self, mouse_position: Vec2) {
        let (x, y) = self.cell_pos_for_click(mouse_position);
        let cell_ind = self.ind_for_pos(x, y);
        self.cells[cell_ind] = !self.cells[cell_ind];
        info!(
            "Called flip_cell on {},{}, making it {}",
            x, y, self.cells[cell_ind]
        );
    }

    pub fn step(&mut self) {
        /* evolve the matrix one step */
        let mut buffer = self.cells.to_vec();
        for y in 0..self.height as i32 {
            for x in 0..self.width as i32 {
                let mut n_neighbors = 0;
                // iterate of cell neighbors
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

                        //let neighbor = [(y + j) as usize * w + (x + i) as usize];
                        //TODO: find a way to take a 2d slice of this 1d vector and sum it rather
                        //than iterating over each point. Rust must have a better matrix library
                        if self.cell_is_alive((x + i) as u16, (y + j) as u16) {
                            n_neighbors += 1;
                        }
                    }
                }

                // add new cell state to buffer
                buffer[self.ind_for_pos(x as u16, y as u16)] =
                    match (self.cell_is_alive(x as u16, y as u16), n_neighbors) {
                        (true, x) if x < 2 => false, // Rule 1: live cell with < 2 neighbors dies
                        (true, 2) | (true, 3) => true, // Rule 2: live cell with 2-3 neighbors survives
                        (true, x) if x > 3 => false,   // Rule 3: live cell with >3 neighbors dies
                        (false, 3) => true, // Rule 4: dead cell with 3 neighbors becomes alive
                        (otherwise, _) => otherwise, // remain in same state
                    };
            }
        }
        self.cells = buffer;
    }
}
