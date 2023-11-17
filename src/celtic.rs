/*
* Port of pure JS implementation of Celtic Knots at https://w-shadow.com/celtic-knots/ to
* macroquad
*
* Could consider using the JS directly, but met with some dependency issues going that route.
* Meanwhile, we're using pre-rendered textures because I found out too late that Macroquad doesn't
* expose anything more than lines, rectangles, and circles
*
*/
use macroquad::prelude::*;
use phf::{phf_map, Map};
use std::f32::consts::PI;

static TILE_LOCS: Map<&'static str, (u16, u16)> = phf_map! {
    "corner" => (0, 0),
    "vertical_line" => (7,1),
    "straight_cross" => (0,2),
    "curved_cross" => (6,8),
    "curved_cross_under" => (8,0),
};

#[derive(Clone, Debug)]
pub enum Cut {
    Open,
    Horizontal,
    Vertical,
    Cross,
}

#[derive(Clone, Debug)]
pub enum Offset {
    Even,
    Odd,
}

#[derive(Debug)]
pub struct Tile {
    pub bottom_cut: Cut,
    pub top_cut: Cut,
    pub row_offset: Offset,
    pub col_offset: Offset,
}

fn draw_tile(
    texture: &Texture2D,
    top_left: Vec2,
    loc: (u16, u16),
    rotation: f32,
    tile_size: u16,
    flip_x: bool,
    flip_y: bool,
) {
    draw_texture_ex(
        texture,
        top_left.x,
        top_left.y,
        WHITE,
        DrawTextureParams {
            source: Some(Rect::new(
                (loc.0 * tile_size).into(),
                (loc.1 * tile_size).into(),
                tile_size.into(),
                tile_size.into(),
            )),
            rotation,
            flip_x,
            flip_y,
            ..Default::default()
        },
    );
}

pub fn draw_expr_for_tile(texture: &Texture2D, tile: Tile, top_left: Vec2, tile_size: u16) {
    /*
     * There are 36 possible tiles in celtic knots,
     * which can be rendered using 5 drawings in various orientations
     *
     * TODO: there must be a more concise way to express this
     */
    match tile {
        Tile {
            bottom_cut: Cut::Open,
            top_cut: Cut::Open,
            row_offset: Offset::Even,
            col_offset: Offset::Even,
        } => {
            //drawStraightCross
            draw_tile(
                texture,
                top_left,
                TILE_LOCS["straight_cross"],
                0.0,
                tile_size,
                false,
                false,
            );
        }
        Tile {
            bottom_cut: Cut::Open,
            top_cut: Cut::Open,
            row_offset: Offset::Even,
            col_offset: Offset::Odd,
        } => {
            //"rotate(drawStraightCross, 90)"
            draw_tile(
                texture,
                top_left,
                TILE_LOCS["straight_cross"],
                PI / 2.0,
                tile_size,
                false,
                false,
            );
        }
        Tile {
            bottom_cut: Cut::Open,
            top_cut: Cut::Open,
            row_offset: Offset::Odd,
            col_offset: Offset::Odd,
        } => {
            //"rotate(drawStraightCross, 180)"
            draw_tile(
                texture,
                top_left,
                TILE_LOCS["straight_cross"],
                PI,
                tile_size,
                false,
                false,
            );
        }
        Tile {
            bottom_cut: Cut::Open,
            top_cut: Cut::Open,
            row_offset: Offset::Odd,
            col_offset: Offset::Even,
        } => {
            //"rotate(drawStraightCross, 270)"
            draw_tile(
                texture,
                top_left,
                TILE_LOCS["straight_cross"],
                PI * 1.5,
                tile_size,
                false,
                false,
            );
        }
        Tile {
            bottom_cut: Cut::Horizontal,
            top_cut: Cut::Vertical,
            row_offset: Offset::Odd,
            col_offset: Offset::Odd,
        }
        | Tile {
            bottom_cut: Cut::Horizontal,
            top_cut: Cut::Vertical,
            row_offset: Offset::Even,
            col_offset: Offset::Even,
        } => {
            //drawCorner NB: the corner tile I'm using is upside down
            draw_tile(
                texture,
                top_left,
                TILE_LOCS["corner"],
                PI,
                tile_size,
                false,
                false,
            );
        }
        Tile {
            bottom_cut: Cut::Horizontal,
            top_cut: Cut::Vertical,
            row_offset: Offset::Odd,
            col_offset: Offset::Even,
        }
        | Tile {
            bottom_cut: Cut::Horizontal,
            top_cut: Cut::Vertical,
            row_offset: Offset::Even,
            col_offset: Offset::Odd,
        } => {
            //"rotate(drawCorner, 90)"
            draw_tile(
                texture,
                top_left,
                TILE_LOCS["corner"],
                PI * 1.5,
                tile_size,
                false,
                false,
            );
        }
        Tile {
            bottom_cut: Cut::Vertical,
            top_cut: Cut::Horizontal,
            row_offset: Offset::Odd,
            col_offset: Offset::Odd,
        }
        | Tile {
            bottom_cut: Cut::Vertical,
            top_cut: Cut::Horizontal,
            row_offset: Offset::Even,
            col_offset: Offset::Even,
        } => {
            //"rotate(drawCorner, 180)"
            draw_tile(
                texture,
                top_left,
                TILE_LOCS["corner"],
                0.0,
                tile_size,
                false,
                false,
            );
        }
        Tile {
            bottom_cut: Cut::Vertical,
            top_cut: Cut::Horizontal,
            row_offset: Offset::Odd,
            col_offset: Offset::Even,
        }
        | Tile {
            bottom_cut: Cut::Vertical,
            top_cut: Cut::Horizontal,
            row_offset: Offset::Even,
            col_offset: Offset::Odd,
        } => {
            //"rotate(drawCorner, 270)"
            draw_tile(
                texture,
                top_left,
                TILE_LOCS["corner"],
                PI * 0.5,
                tile_size,
                false,
                false,
            );
        }
        Tile {
            bottom_cut: Cut::Horizontal,
            top_cut: Cut::Horizontal,
            ..
        } => {
            // "drawHorizontalLine"
            draw_tile(
                texture,
                top_left,
                TILE_LOCS["vertical_line"],
                PI / 2.0,
                tile_size,
                false,
                false,
            );
        }
        Tile {
            bottom_cut: Cut::Vertical,
            top_cut: Cut::Vertical,
            ..
        } => {
            // "drawVerticalLine"
            draw_tile(
                texture,
                top_left,
                TILE_LOCS["vertical_line"],
                0.0,
                tile_size,
                false,
                false,
            );
        }
        Tile {
            bottom_cut: Cut::Vertical,
            top_cut: Cut::Open,
            row_offset: Offset::Odd,
            col_offset: Offset::Odd,
        } => {
            //"drawCurvedCross"
            draw_tile(
                texture,
                top_left,
                TILE_LOCS["curved_cross"],
                PI,
                tile_size,
                false,
                false,
            );
        }
        Tile {
            bottom_cut: Cut::Vertical,
            top_cut: Cut::Open,
            row_offset: Offset::Even,
            col_offset: Offset::Even,
        } => {
            //"drawCurvedCrossUnder"
            draw_tile(
                texture,
                top_left,
                TILE_LOCS["curved_cross_under"],
                0.0,
                tile_size,
                false,
                false,
            );
        }
        Tile {
            bottom_cut: Cut::Vertical,
            top_cut: Cut::Open,
            row_offset: Offset::Odd,
            col_offset: Offset::Even,
        } => {
            //"flipHorizontally(drawCurvedCrossUnder)"
            draw_tile(
                texture,
                top_left,
                TILE_LOCS["curved_cross_under"],
                0.0,
                tile_size,
                true,
                false,
            );
        }
        Tile {
            bottom_cut: Cut::Vertical,
            top_cut: Cut::Open,
            row_offset: Offset::Even,
            col_offset: Offset::Odd,
        } => {
            //"flipHorizontally(drawCurvedCross)"
            draw_tile(
                texture,
                top_left,
                TILE_LOCS["curved_cross"],
                PI,
                tile_size,
                true,
                false,
            );
        }
        Tile {
            bottom_cut: Cut::Open,
            top_cut: Cut::Vertical,
            row_offset: Offset::Odd,
            col_offset: Offset::Odd,
        } => {
            //"rotate(drawCurvedCrossUnder, 180)"
            draw_tile(
                texture,
                top_left,
                TILE_LOCS["curved_cross_under"],
                PI,
                tile_size,
                false,
                false,
            );
        }
        Tile {
            bottom_cut: Cut::Open,
            top_cut: Cut::Vertical,
            row_offset: Offset::Odd,
            col_offset: Offset::Even,
        } => {
            //"rotate(flipHorizontally(drawCurvedCross), 180)"
            draw_tile(
                texture,
                top_left,
                TILE_LOCS["curved_cross"],
                0.0,
                tile_size,
                true,
                false,
            );
        }
        Tile {
            bottom_cut: Cut::Open,
            top_cut: Cut::Vertical,
            row_offset: Offset::Even,
            col_offset: Offset::Even,
        } => {
            //"rotate(drawCurvedCross, 180)"
            draw_tile(
                texture,
                top_left,
                TILE_LOCS["curved_cross"],
                0.0,
                tile_size,
                false,
                false,
            );
        }
        Tile {
            bottom_cut: Cut::Open,
            top_cut: Cut::Vertical,
            row_offset: Offset::Even,
            col_offset: Offset::Odd,
        } => {
            //"rotate(flipHorizontally(drawCurvedCrossUnder), 180)"
            draw_tile(
                texture,
                top_left,
                TILE_LOCS["curved_cross_under"],
                PI,
                tile_size,
                true,
                false,
            );
        }
        Tile {
            bottom_cut: Cut::Horizontal,
            top_cut: Cut::Open,
            row_offset: Offset::Odd,
            col_offset: Offset::Odd,
        } => {
            //"rotate(flipHorizontally(drawCurvedCross), 90)"
            draw_tile(
                texture,
                top_left,
                TILE_LOCS["curved_cross"],
                PI*1.5,
                tile_size,
                true,
                false,
            );
        }
        Tile {
            bottom_cut: Cut::Horizontal,
            top_cut: Cut::Open,
            row_offset: Offset::Odd,
            col_offset: Offset::Even,
        } => {
            //"rotate(drawCurvedCrossUnder, 270)"
            draw_tile(
                texture,
                top_left,
                TILE_LOCS["curved_cross_under"],
                PI * 1.5,
                tile_size,
                false,
                false,
            );
        }
        Tile {
            bottom_cut: Cut::Horizontal,
            top_cut: Cut::Open,
            row_offset: Offset::Even,
            col_offset: Offset::Even,
        } => {
            //"rotate(flipHorizontally(drawCurvedCrossUnder), 90)"
            draw_tile(
                texture,
                top_left,
                TILE_LOCS["curved_cross_under"],
                PI / 2.0,
                tile_size,
                true,
                false,
            );
        }
        Tile {
            bottom_cut: Cut::Horizontal,
            top_cut: Cut::Open,
            row_offset: Offset::Even,
            col_offset: Offset::Odd,
        } => {
            //"rotate(drawCurvedCross, 270)"
            draw_tile(
                texture,
                top_left,
                TILE_LOCS["curved_cross"],
                PI * 0.5,
                tile_size,
                false,
                false,
            );
        }
        Tile {
            bottom_cut: Cut::Open,
            top_cut: Cut::Horizontal,
            row_offset: Offset::Odd,
            col_offset: Offset::Odd,
        } => {
            //"rotate(flipHorizontally(drawCurvedCrossUnder), 270)"
            draw_tile(
                texture,
                top_left,
                TILE_LOCS["curved_cross_under"],
                PI * 1.5,
                tile_size,
                true,
                false,
            );
        }
        Tile {
            bottom_cut: Cut::Open,
            top_cut: Cut::Horizontal,
            row_offset: Offset::Odd,
            col_offset: Offset::Even,
        } => {
            //"rotate(drawCurvedCross, 90)"
            draw_tile(
                texture,
                top_left,
                TILE_LOCS["curved_cross"],
                PI * 1.5,
                tile_size,
                false,
                false,
            );
        }
        Tile {
            bottom_cut: Cut::Open,
            top_cut: Cut::Horizontal,
            row_offset: Offset::Even,
            col_offset: Offset::Even,
        } => {
            //"rotate(flipHorizontally(drawCurvedCross), 270)"
            draw_tile(
                texture,
                top_left,
                TILE_LOCS["curved_cross"],
                PI * 0.5,
                tile_size,
                true,
                false,
            );
        }
        Tile {
            bottom_cut: Cut::Open,
            top_cut: Cut::Horizontal,
            row_offset: Offset::Even,
            col_offset: Offset::Odd,
        } => {
            //"rotate(drawCurvedCrossUnder, 90)"
            draw_tile(
                texture,
                top_left,
                TILE_LOCS["curved_cross_under"],
                PI / 2.0,
                tile_size,
                false,
                false,
            );
        }
        _ => {
            // error
            draw_rectangle(
                top_left.x,
                top_left.y,
                tile_size.into(),
                tile_size.into(),
                BLACK,
            );
        }
    }
}
