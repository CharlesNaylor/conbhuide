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

pub struct TileMatrix {
    pub width: u16,
    pub height: u16,
    tile_size: u16,
    nodes: Vec<bool>,
    edges: Vec<Cut>,
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
            edges: vec![Cut::Cross; (width * height) as usize],
            texture,
        }
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

    fn node_ind_for_tile_pos(&self, x: u16, y: u16) -> usize {
        /* return node index for a given x,y coordinate
         * (cells are stored in a 1d vector)
         * recall there are half as many horizontal nodes
         * as there are tiles
         */
        ((y*(self.width/2 + 1)) + (x/2)) as usize
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

    fn tile_pos_for_click(&self, screen_pos: Vec2) -> (u16, u16) {
        /* translate a click on the screen to a tile position */
        info!("Screen position {},{}", screen_pos.x, screen_pos.y,);
        (
            screen_pos.x as u16 / self.tile_size,
            screen_pos.y as u16 / self.tile_size,
        )
    }

    fn nearest_node_to_click(&self, screen_pos: Vec2) -> (u16, u16) {
        /* nearest node to click */
        let y: u16 = (screen_pos.y / self.tile_size as f32).round() as u16;
        let x: u16 = if y % 2 == 0 {
            (screen_pos.x / (self.tile_size*2) as f32).round() as u16
        } else {
            ((screen_pos.x-self.tile_size as f32) / (self.tile_size*2) as f32).round() as u16
        };
        info!("Nearest node to click {},{} is {}, {}", screen_pos.x, screen_pos.y, x, y);
        (x, y)
    }

    pub fn flip_node(&mut self, mouse_position: Vec2) {
        let (x, y) = self.nearest_node_to_click(mouse_position);
        let node_ind = self.node_ind_for_pos(x, y);
        self.nodes[node_ind] = !self.nodes[node_ind];
        info!(
            "Called flip_node on {},{}, index {}, making it {}",
            x, y, node_ind, self.nodes[node_ind]
        );
    }

    pub fn draw_tiles(&self) {
        // draw all the tiles
        for x in 0..self.width {
            for y in 0..self.height {
                let tile: Tile = self.tile_for_pos(x, y);
                let top_left: Vec2 = self.loc_for_node(x, y);
                draw_expr_for_tile(&self.texture, tile, top_left, self.tile_size);
            }
        }
    }

    fn tile_for_pos(&self, x: u16, y: u16) -> Tile {
        /* instantiate a tile based on information about nearby edges */
        // note these are odd and even as if things were 1-indexed
        let row_odd: bool = (y + 1) % 2 == 1;
        let col_odd: bool = (x + 1) % 2 == 1;
        let row_offset: Offset = if row_odd { Offset::Odd } else { Offset::Even };
        let col_offset: Offset = if col_odd { Offset::Odd } else { Offset::Even };
        let first_cut: Cut;
        let second_cut: Cut;
        (first_cut, second_cut) = match (col_odd, row_odd) {
            (true, ..) => (Cut::Open, Cut::Open),     // y+1, x/2  &  y, x/2
            (false, true) => (Cut::Open, Cut::Open),  // y+1, (x+1)/2  &  y, (x-1)/2
            (false, false) => (Cut::Open, Cut::Open), // y+1, (x-1)/2  &  y, (x+1)/2
        };
        Tile {
            first_cut,
            second_cut,
            row_offset,
            col_offset,
        }
    }

    pub fn calc_edges(&mut self) {
        /* calculate edges based on nodes */
        for x in 0..self.width {
            for y in 0..self.height {
                let ind = self.ind_for_pos(x, y); // necessary b/c of something about borrowing
                                                  // muts
                self.edges[ind] = Cut::Cross; // TODO: add logic
            }
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
                if self.nodes[self.node_ind_for_pos(x, y)] {
                    draw_circle(
                        node_loc.x,
                        node_loc.y,
                        1.5 * self.spacing() as f32,
                        node_color,
                    );
                } else {
                    draw_circle(node_loc.x, node_loc.y, self.spacing().into(), node_color);
                }
                //edges
                for (next_x, next_y) in [(x + 1, y), (x, y + 2)] {
                    // node_ind_for_pos uses tiles. we're iterating over nodes here
                    if (self.nodes[self.node_ind_for_pos(x, y)])
                        && (self.nodes[self.node_ind_for_pos(next_x, next_y)])
                    {
                        let node_loc_end: Vec2 = self.loc_for_node(next_x, next_y);
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
        }
    }
}
