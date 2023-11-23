/*
* Set of cellular automata rules for a series of edges that would be appropriate for generating
* Celtic knots.
*
* We have a matrix, but this matrix is offset in alternating rows, i.e. the dots resemble a
* quincunx. A quincunx matrix is {(0,0), (0,1), (0,2), (1,0), (1,2)}, and represents a 2 x 2 set of tiles
*
* The "tile" going from 0,1 to 1,2 is influenced by the following edges:
*   a. (0,1), (1,1)
*   b. (0,1), (0,3)
*   c. (0,2), (1,2)
*   d. (1,0), (1,2)
*
* but it can't have crossing edges (remember the offset). This means the following pairs can't
* exist:
*    - a,d
*    - b,c
*
*  So, valid tiles have one of these sets of edges:
*    - {}
*    - a
*    - b
*    - c
*    - d
*    - a,b
*    - a,c
*    - b,d
*
*
* To make this appealing, we want to reverse the meaning of an initialized matrix. A blank "tile"
* should be initalized as {a,b,c,d}, and we need good rules to subtract edges.
*
* Otherwise, I think it will be trial and error to discover what rules generate appealing cellular
* automata. Step one is to design an engine so we can visualize the results.

  /* The 'quincunx' offset structure means we basically have
    * 2 matrices; call them 'even' and 'odd', each of which can
    * have connections with adjacent values ( x +/- 1, y +/- 1)
    * evolution rules will need to consider both matrices, though
    */

  I think it's probably easier to evolve the edges directly.
*/
use crate::celtic::{draw_expr_for_tile, Cut, Offset, Tile};
use macroquad::prelude::*;
use macroquad::rand::gen_range;
use std::collections::HashSet;
use std::cmp::max;

pub struct TileMatrix {
    pub width: u16,
    pub height: u16,
    tile_size: u16,
    nodes: Vec<bool>,
    pub edges: HashSet<((i16, i16),(i16,i16))>,
    texture: Texture2D,
}
impl TileMatrix {
    pub fn new(screen_size: Vec2, tile_size: u16, texture: Texture2D) -> Self {
        let width: u16 = (screen_size.x / tile_size as f32) as u16;
        let height: u16 = (screen_size.y / tile_size as f32) as u16;
        // to make edges, we should have 1 fewer nodes than columns of tiles,
        // and 1 more than rows
        TileMatrix {
            width,
            height,
            tile_size,
            nodes: vec![false; ((width / 2 + 1) * (height + 1)) as usize],
            edges: HashSet::new(),
            texture,
        }
    }

    pub fn draw_texture(&self) {
        draw_texture(&self.texture, 0.0, 0.0, WHITE);
    }

    pub fn spacing(&self) -> u16 {
        /* return a good value for pixel spacing based on tile_size
         * e.g., for drawing edge lines */
        self.tile_size / 10
    }

    fn ind_for_pos(&self, x: u16, y: u16) -> usize {
        /* return tile index for a given x,y coordinate
         * (cells are stored in a 1d vector) */
        (y * (self.width - 1)) as usize + x as usize
    }

    fn node_ind_for_pos(&self, x: u16, y: u16) -> usize {
        /* return node index for a given x,y coordinate
         * (cells are stored in a 1d vector)
         * recall there are half as many horizontal nodes
         * as there are tiles
         */
        ((y*(self.width/2 + 1)) + x ) as usize
    }

    pub fn loc_for_node(&self, x: u16, y: u16) -> Vec2 {
        /* return offset position of a node on screen */
        if y % 2 == 0 {
            vec2((x * self.tile_size*2).into(), (y * self.tile_size).into())
        } else {
            vec2(
                (self.tile_size  + x * self.tile_size*2).into(),
                (y * self.tile_size).into(),
            )
        }
    }

    pub fn loc_for_tile(&self, x: u16, y: u16) -> Vec2 {
        /* return position of a tile on screen */
            vec2((x * self.tile_size).into(), (y * self.tile_size).into())
    }

    pub fn tile_pos_for_click(&self, screen_pos: Vec2) -> (u16, u16) {
        /* translate a click on the screen to a tile position */
        info!("Screen position {},{}", screen_pos.x, screen_pos.y,);
        (
            screen_pos.x as u16 / self.tile_size,
            screen_pos.y as u16 / self.tile_size,
        )
    }

    fn nearest_edge_to_click(&self, screen_pos: Vec2) -> ((u16, u16), (u16, u16)) {
        /* nearest pair of nodes (constituting an edge) to click */
        let y_ft: f32 = screen_pos.y / self.tile_size as f32;
        let y_1: u16 = y_ft.round() as u16;
        let x_ft: f32 = if y_1 % 2 == 0 {
            screen_pos.x / (self.tile_size*2) as f32
        } else {
            (screen_pos.x-self.tile_size as f32) / (self.tile_size*2) as f32
        };
        let x_1: u16 = x_ft.round() as u16;

        let x_dist = x_ft - x_1 as f32;
        let y_dist = y_ft - y_1 as f32;

        let (x_2, y_2) = if x_dist >= y_dist {
            (max(0,(x_1 as i16) + if x_dist >= 0.0 {1} else {-1}) as u16,
            y_1)
        } else {
            (x_1,
            max(0,(y_1 as i16) + if y_dist >= 0.0 {2} else {-2}) as u16)
        };
        // So that's the nearest node. The second node will be whichever one
        info!("Nearest node to click {},{} is {}, {}", screen_pos.x, screen_pos.y, x_1, y_1);
        ((x_1, y_1), (x_2,y_2))
    }

    pub fn flip_edge(&mut self, mouse_position: Vec2) {
        // TODO: need a data structure that's indifferent to the order in which the edges are
        // stored
        let node_pair_u = self.nearest_edge_to_click(mouse_position);
        let node_pair = ((node_pair_u.0.0 as i16, node_pair_u.0.1 as i16), (node_pair_u.1.0 as i16, node_pair_u.1.1 as i16));
        let mut node_pair_rev = node_pair.clone();
        node_pair_rev = ((node_pair_rev.1.0, node_pair_rev.1.1), (node_pair_rev.0.0, node_pair_rev.0.1));

        let add_rem: &str;
        if self.edges.contains(&node_pair) {
            add_rem = "Removed";
            self.edges.remove(&node_pair);
            self.edges.remove(&node_pair_rev);
        } else {
            add_rem = "Added";
            self.edges.insert(node_pair);
            self.edges.insert(node_pair_rev);
        }
        info!("{} edge at {:?}", add_rem, node_pair);
    }

    pub fn draw_tiles(&self) {
        // draw all the tiles
        for x in 0..self.width {
            for y in 0..self.height {
                let tile: Tile = self.tile_for_pos(x, y);
                let top_left: Vec2 = self.loc_for_tile(x, y);
                draw_expr_for_tile(&self.texture, tile, top_left, self.tile_size);
            }
        }
    }

    pub fn tile_for_pos(&self, x: u16, y: u16) -> Tile {
        /* instantiate a tile based on information about nearby edges */
        // note these are odd and even as if things were 1-indexed
        let row_offset: Offset = if y % 2 == 1 { Offset::Odd } else { Offset::Even };
        let col_offset: Offset = if x % 2 == 1 { Offset::Odd } else { Offset::Even };
        Tile {
            bottom_cut: self.cut_for_tile(x, y, &row_offset, &col_offset, true),
            top_cut: self.cut_for_tile(x, y, &row_offset, &col_offset, false),
            row_offset,
            col_offset,
        }
    }

    fn cut_for_tile(&self, x: u16, y: u16, row_offset: &Offset, col_offset: &Offset, is_bottom: bool) -> Cut {
        /* get bottom-most cut on a tile */
        let n_x: i16 = (x as f32 / 2.0).round() as i16;
        let n_y: i16 = y as i16;
        let (vert_exists, hori_exists) = match (is_bottom, row_offset, col_offset) {
            (true, Offset::Even, Offset::Even) => {
                (
                    self.edges.contains(&((n_x,n_y),(n_x,n_y+2))),
                    self.edges.contains(&((n_x-1,n_y+1),(n_x,n_y+1)))
                )
            },
            (true, Offset::Odd, Offset::Even) => {
                (
                    self.edges.contains(&((n_x,n_y),(n_x,n_y+2))),
                    self.edges.contains(&((n_x,n_y+1),(n_x+1,n_y+1))),
                )
            },
            (true, Offset::Odd, Offset::Odd) => {
                (
                    self.edges.contains(&((n_x-1,n_y),(n_x-1,n_y+2))),
                    self.edges.contains(&((n_x-1,n_y+1),(n_x,n_y+1)))
                )
            },
            (true, Offset::Even, Offset::Odd) => {
                (
                    self.edges.contains(&((n_x,n_y),(n_x,n_y+2))),
                    self.edges.contains(&((n_x-1,n_y+1),(n_x,n_y+1)))
                )
            },
            (false, Offset::Even, Offset::Even) => {
                (
                    self.edges.contains(&((n_x,n_y-1), (n_x,n_y+1))),
                    self.edges.contains(&((n_x,n_y),(n_x+1,n_y)))
                )
            },
            (false, Offset::Odd, Offset::Odd) => {
                (
                    self.edges.contains(&((n_x,n_y-1),(n_x,n_y+1))),
                    self.edges.contains(&((n_x-1,n_y),(n_x,n_y)))
                )
            },
            (false, Offset::Even, Offset::Odd) => {
                (
                    self.edges.contains(&((n_x-1,n_y-1), (n_x-1,n_y+1))),
                    self.edges.contains(&((n_x-1,n_y), (n_x,n_y)))
                )
            }
            (false, Offset::Odd, Offset::Even) => {
                (
                    self.edges.contains(&((n_x,n_y-1), (n_x,n_y+1))),
                    self.edges.contains(&((n_x-1,n_y), (n_x,n_y)))
                )
            }
        };
        match (vert_exists, hori_exists) {
            (true, true) => Cut::Cross,
            (true, false) => Cut::Vertical,
            (false, true) => Cut::Horizontal,
            (false, false) => Cut::Open
        }
    }

    pub fn draw_edges(&self) {
        /*
         * draw dots for even and odd rows,
         * add edges
         */
        for y in 0..(self.height - 1) {
            let node_color: Color = if y % 2 == 0 { RED } else { BLUE };
            for x in 0..(self.width / 2 + 1) {
                let node_loc: Vec2 = self.loc_for_node(x, y);
                draw_circle(node_loc.x, node_loc.y, self.spacing().into(), node_color);
            }
        }

        //edges
        for edge in self.edges.iter() {
            let node_loc = self.loc_for_node(edge.0.0 as u16, edge.0.1 as u16);
            let node_loc_end = self.loc_for_node(edge.1.0 as u16, edge.1.1 as u16);
            draw_line(
                node_loc.x,
                node_loc.y,
                node_loc_end.x,
                node_loc_end.y,
                (self.spacing() - 1).into(),
                WHITE,
            );
        }

            }
        }
