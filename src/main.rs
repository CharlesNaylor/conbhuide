/*
 * Main file for compiling to wasm
 */
use macroquad::prelude::*;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum CellState {
    Alive,
    Dead,
}

#[macroquad::main("Conbhuide")]
async fn main() {
    let w = screen_width() as usize;
    let h = screen_height() as usize;
    info!("{} by {} canvas", w, h);

    let mut cells = vec![CellState::Dead; w * h];
    let mut buffer = vec![CellState::Dead; w * h];

    let mut image = Image::gen_image_color(w as u16, h as u16, BLACK);

    // populate random living cells in initial state
    for cell in cells.iter_mut() {
        if rand::gen_range(0, 5) == 0 {
            *cell = CellState::Alive;
        }
    }
    let texture = Texture2D::from_image(&image);
    loop {
        clear_background(WHITE);
        let w = image.width();
        let h = image.height();

        for y in 0..h as i32 {
            for x in 0..w as i32 {
                let mut n_neighbors = 0;
                // iterate of cell neighbors
                for j in -1i32..=1 {
                    for i in -1i32..=1 {
                        // out of bounds
                        if y + j < 0 || y + j >= h as i32 || x + i < 0 || x + i >= w as i32 {
                            continue;
                        }
                        // I am not a neighbor of myself
                        if i == 0 && j == 0 {
                            continue;
                        }

                        let neighbor = cells[(y + j) as usize * w + (x + i) as usize];
                        if neighbor == CellState::Alive {
                            n_neighbors += 1;
                        }
                    }
                }

                // add new cell state to buffer
                let current_cell = cells[y as usize * w + x as usize];
                buffer[y as usize * w + x as usize] = match (current_cell, n_neighbors) {
                    (CellState::Alive, x) if x < 2 => CellState::Dead, // Rule 1: live cell with < 2 neighbors dies
                    (CellState::Alive, 2) | (CellState::Alive, 3) => CellState::Alive, // Rule 2: live cell with 2-3 neighbors survives
                    (CellState::Alive, x) if x > 3 => CellState::Dead, // Rule 3: live cell with >3 neighbors dies
                    (CellState::Dead, 3) => CellState::Alive, // Rule 4: dead cell with 3 neighbors becomes alive
                    (otherwise, _) => otherwise,              // remain in same state
                };
            }
        }

        for i in 0..buffer.len() {
            cells[i] = buffer[i];

            image.set_pixel(
                (i % w) as u32,
                (i / w) as u32,
                match buffer[i as usize] {
                    CellState::Alive => BLACK,
                    CellState::Dead => WHITE,
                },
            );
        }

        texture.update(&image);
        draw_texture(&texture, 0., 0., WHITE);
        next_frame().await
    }
}
