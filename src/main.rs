/*
 * Main file for compiling to wasm
 */
pub mod celtic;
pub mod edge;
pub mod life;
use crate::edge::TileMatrix;
use crate::life::CellMatrix;
use macroquad::input;
use macroquad::prelude::*;
use macroquad::ui::{hash, root_ui, widgets::Window};

const CELL_SIZE: u16 = 25;

#[macroquad::main("Conbhuide")]
async fn main() {
    let texture: Texture2D = load_texture("img/knots.png").await.unwrap();
    let screen_size = vec2(screen_width(), screen_height());
    // let mut cell_matrix: CellMatrix = CellMatrix::new(screen_size, CELL_SIZE);
    // cell_matrix.randomize(None);
    // info!(
    //     "{} by {} canvas, for {} by {} cells",
    //     screen_height(),
    //     screen_width(),
    //     cell_matrix.height,
    //     cell_matrix.width
    // );
    let mut tile_matrix: TileMatrix = TileMatrix::new(screen_size, CELL_SIZE, texture);
    info!("TileMatrix: width {}, height {}",tile_matrix.width, tile_matrix.height);

    let mut running: bool = true;
    let mut show_edges: bool = true;
    loop {
        //clear_background(WHITE);
        if is_key_pressed(KeyCode::Space) {
            running = !running;
            if running {
                info!("Resumed");
            } else {
                info!("Paused");
            }
        }
        if is_key_pressed(KeyCode::E) {
            show_edges = !show_edges;
            if show_edges {
                info!("Show edges");
            } else {
                info!("Hide edges");
            }
        }

        if is_mouse_button_pressed(MouseButton::Left) {
            //cell_matrix.flip_cell(Vec2::from(mouse_position()));
            tile_matrix.flip_node(Vec2::from(mouse_position()));
        }
        /*
        if running {
            cell_matrix.step();
        }
        cell_matrix.draw();
        */
        tile_matrix.draw_tiles();
        if show_edges {
            tile_matrix.draw_edges();
        }
        next_frame().await
    }
}
