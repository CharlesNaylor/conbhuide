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
use macroquad::ui::{hash, root_ui, widgets::Window, widgets::Button};
use std::time;

const CELL_SIZE: u16 = 25;
const FRAME_TOP_LEFT: Vec2 = vec2(0., 40.);

#[macroquad::main("Conbhuide")]
async fn main() {
    let texture: Texture2D = load_texture("img/knots.png").await.unwrap();
    let screen_size = vec2(screen_width(), screen_height()-FRAME_TOP_LEFT.y);
    let mut cell_matrix: CellMatrix = CellMatrix::new(screen_size, CELL_SIZE, Some(FRAME_TOP_LEFT));
    cell_matrix.randomize(None);
    info!(
        "{} by {} canvas, for {} by {} cells",
        screen_height(),
        screen_width(),
        cell_matrix.height,
        cell_matrix.width
    );
    let mut tile_matrix: TileMatrix = TileMatrix::new(screen_size, CELL_SIZE, texture, Some(FRAME_TOP_LEFT));
    info!("TileMatrix: width {}, height {}",tile_matrix.width, tile_matrix.height);

    let mut running: bool = true;
    let mut show_edges: bool = true;
    let mut is_conway: bool = true;
    let mut fps: f32 = 10.0;
    let mut step_time: f64 = 0.0;
    loop {
        // setup ui
        if root_ui().button(None, "Celtic") {
            is_conway = !is_conway;
        };
        if root_ui().button(vec2(50.,0.), ">||") {
            running = !running;
        };
        root_ui().slider(hash!(), "FPS", 0.1..30.0, &mut fps);
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
        if is_key_pressed(KeyCode::C) {
            is_conway = !is_conway;
        }

        if is_conway {
            if is_mouse_button_pressed(MouseButton::Left) {
                //cell_matrix.flip_cell(Vec2::from(mouse_position()));
                let mouse_pos = Vec2::from(mouse_position());
                if (mouse_pos.x > FRAME_TOP_LEFT.x) & (mouse_pos.y > FRAME_TOP_LEFT.y) {
                    tile_matrix.flip_edge(mouse_pos);
                }
            }
            if is_mouse_button_pressed(MouseButton::Right) {
                let mouse_pos = Vec2::from(mouse_position());
                let (tile_x, tile_y) = tile_matrix.tile_pos_for_click(mouse_pos);
                info!("clicked on tile {}, {}:\n\t{:?}",tile_x, tile_y, tile_matrix.tile_for_pos(tile_x, tile_y)); 
            }
            if is_key_pressed(KeyCode::D) {
                info!("Edges:");
                for edge in &tile_matrix.edges {
                    info!("({}, {}), ({}, {})", edge.0.0, edge.0.1, edge.1.0, edge.1.1);
                }
            }
            tile_matrix.draw_tiles();
            if show_edges {
                tile_matrix.draw_edges();
            }
        } else {
            if running {
                if get_time() > (step_time + ((1.0/fps) as f64)) {
                    cell_matrix.step();
                    step_time = get_time();
                }
            }
            if is_mouse_button_pressed(MouseButton::Left) {
                //cell_matrix.flip_cell(Vec2::from(mouse_position()));
                let mouse_pos = Vec2::from(mouse_position());
                if (mouse_pos.x > FRAME_TOP_LEFT.x) & (mouse_pos.y > FRAME_TOP_LEFT.y) {
                    cell_matrix.flip_cell(mouse_pos);
                }
            }
            cell_matrix.draw();
        }
        next_frame().await
    }
}
