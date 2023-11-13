/*
 * Main file for compiling to wasm
 */
pub mod life;
use crate::life::CellMatrix;
use macroquad::input;
use macroquad::prelude::*;
use macroquad::ui::{hash, root_ui, widgets::Window};

const CELL_SIZE: u16 = 25;

#[macroquad::main("Conbhuide")]
async fn main() {
    let screen_size = vec2(screen_width(), screen_height());
    let mut cell_matrix: CellMatrix = CellMatrix::new(screen_size, CELL_SIZE);
    cell_matrix.randomize(None);
    info!(
        "{} by {} canvas, for {} by {} cells",
        screen_height(),
        screen_width(),
        cell_matrix.height,
        cell_matrix.width
    );

    let mut running: bool = true;
    loop {
        clear_background(WHITE);
        if is_key_pressed(KeyCode::Space) {
            running = !running;
            if running {
                info!("Resumed");
            } else {
                info!("Paused");
            }
        }
        if is_mouse_button_pressed(MouseButton::Left) {
            cell_matrix.flip_cell(Vec2::from(mouse_position()));
        }
        if running {
            cell_matrix.step();
        }
        cell_matrix.draw();
        next_frame().await
    }
}
