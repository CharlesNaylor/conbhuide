/*
 * Network automaton: the state lives on the edges of a graph rather than on cells.
 *
 * The substrate is a planar triangular mesh. Each of its edges is either active or not, and
 * the active edges form the graph that the knot is drawn around (see mercat.rs). A node with
 * no active edges simply is not there, so nodes and edges both come and go, while planarity
 * is guaranteed by the substrate. (This is the "restricted network automaton" of Smith,
 * Onnela, Lee, Fricker & Johnson, 2011.)
 *
 * The rule is totalistic: an edge looks at how many of the edges that share one of its two
 * end nodes are active (0..=10 on this mesh). `birth` and `survive` are bitmasks over that
 * count, as in life.rs.
 */
use macroquad::prelude::*;
use macroquad::rand::gen_range;
use std::collections::HashMap;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NetRule {
    pub name: &'static str,
    pub birth: u16,
    pub survive: u16,
    /// fraction of active edges that makes a good random start for this rule
    pub seed_density: f32,
}

pub const fn mask(counts: &[u8]) -> u16 {
    let mut m = 0u16;
    let mut i = 0;
    while i < counts.len() {
        m |= 1 << counts[i];
        i += 1;
    }
    m
}

/// Found by sampling random rules for ones that neither die out, fill the mesh, nor fall
/// into a short cycle.
pub const NET_RULES: [NetRule; 4] = [
    NetRule {
        name: "Islands B4,8,9/S0,1,2,3,6,8",
        birth: mask(&[4, 8, 9]),
        survive: mask(&[0, 1, 2, 3, 6, 8]),
        seed_density: 0.3,
    },
    NetRule {
        name: "Lace B3/S2,3,4",
        birth: mask(&[3]),
        survive: mask(&[2, 3, 4]),
        seed_density: 0.3,
    },
    NetRule {
        name: "Weave B4,7,8/S1,2,4,5,6,10",
        birth: mask(&[4, 7, 8]),
        survive: mask(&[1, 2, 4, 5, 6, 10]),
        seed_density: 0.3,
    },
    NetRule {
        name: "Bloom B1,8/S2,4,8,9,10",
        birth: mask(&[1, 8]),
        survive: mask(&[2, 4, 8, 9, 10]),
        seed_density: 0.15,
    },
];

pub struct Network {
    /// node positions on screen (home position plus jitter)
    pub nodes: Vec<Vec2>,
    home: Vec<Vec2>,
    /// substrate edges as (lower node id, higher node id)
    pub edges: Vec<(usize, usize)>,
    pub active: Vec<bool>,
    /// substrate edge ids around each node, in order of angle
    pub incident: Vec<Vec<usize>>,
    /// for each edge, the other edges that share one of its end nodes
    adjacent: Vec<Vec<usize>>,
    edge_ids: HashMap<(usize, usize), usize>,
    /// left-right and top-bottom mirror image of each node
    mirror_x: Vec<usize>,
    mirror_y: Vec<usize>,
    row_start: Vec<usize>,
    pub spacing: f32,
    pub rule: NetRule,
}

impl Network {
    /// Triangular mesh filling `area`, with nodes `spacing` apart.
    pub fn triangular(area: Rect, spacing: f32) -> Self {
        let row_height = spacing * 3f32.sqrt() / 2.0;
        let cols = (((area.w - spacing) / spacing) as usize + 1).max(2);
        let mut rows = (((area.h - spacing) / row_height) as usize + 1).max(1);
        if rows % 2 == 0 {
            rows -= 1; // odd, so the mesh is its own top-bottom mirror image
        }
        let x0 = area.x + (area.w - (cols - 1) as f32 * spacing) / 2.0;
        let y0 = area.y + (area.h - (rows - 1) as f32 * row_height) / 2.0;

        let row_len = |j: usize| if j % 2 == 0 { cols } else { cols - 1 };
        let mut home = vec![];
        let mut row_start = vec![];
        let mut mirror_x = vec![];
        let mut mirror_y = vec![];
        for j in 0..rows {
            row_start.push(home.len());
            let offset = if j % 2 == 0 { 0.0 } else { spacing / 2.0 };
            for i in 0..row_len(j) {
                home.push(vec2(x0 + offset + i as f32 * spacing, y0 + j as f32 * row_height));
            }
        }
        for j in 0..rows {
            for i in 0..row_len(j) {
                mirror_x.push(row_start[j] + row_len(j) - 1 - i);
                mirror_y.push(row_start[rows - 1 - j] + i);
            }
        }

        let mut edges = vec![];
        for j in 0..rows {
            let n = row_len(j);
            for i in 0..n {
                let here = row_start[j] + i;
                if i + 1 < n {
                    edges.push((here, here + 1));
                }
                if j + 1 < rows {
                    let below = row_start[j + 1];
                    if j % 2 == 0 {
                        // the row below is shorter and shifted right by half a step
                        if i >= 1 {
                            edges.push((here, below + i - 1));
                        }
                        if i < cols - 1 {
                            edges.push((here, below + i));
                        }
                    } else {
                        edges.push((here, below + i));
                        edges.push((here, below + i + 1));
                    }
                }
            }
        }

        let mut incident = vec![vec![]; home.len()];
        let mut edge_ids = HashMap::new();
        for (id, &(a, b)) in edges.iter().enumerate() {
            incident[a].push(id);
            incident[b].push(id);
            edge_ids.insert((a, b), id);
        }
        for (node, list) in incident.iter_mut().enumerate() {
            let angle = |id: &usize| {
                let (a, b) = edges[*id];
                let d = home[if a == node { b } else { a }] - home[node];
                d.y.atan2(d.x)
            };
            list.sort_by(|p, q| angle(p).partial_cmp(&angle(q)).unwrap());
        }
        let adjacent = edges
            .iter()
            .enumerate()
            .map(|(id, &(a, b))| {
                incident[a]
                    .iter()
                    .chain(incident[b].iter())
                    .copied()
                    .filter(|other| *other != id)
                    .collect()
            })
            .collect();

        Network {
            nodes: home.clone(),
            home,
            active: vec![false; edges.len()],
            edges,
            incident,
            adjacent,
            edge_ids,
            mirror_x,
            mirror_y,
            row_start,
            spacing,
            rule: NET_RULES[0],
        }
    }

    pub fn other_end(&self, edge: usize, node: usize) -> usize {
        let (a, b) = self.edges[edge];
        if a == node {
            b
        } else {
            a
        }
    }

    /// Node `i` of row `j` (odd rows are one node shorter than even rows).
    pub fn node_id(&self, i: usize, j: usize) -> usize {
        self.row_start[j] + i
    }

    pub fn edge_between(&self, a: usize, b: usize) -> Option<usize> {
        self.edge_ids.get(&(a.min(b), a.max(b))).copied()
    }

    pub fn clear(&mut self) {
        self.active.iter_mut().for_each(|e| *e = false);
    }

    pub fn fill(&mut self) {
        self.active.iter_mut().for_each(|e| *e = true);
    }

    /// Activate every substrate edge whose two ends are both in `nodes`.
    pub fn activate_among(&mut self, nodes: &[usize]) {
        for (id, (a, b)) in self.edges.iter().enumerate() {
            if nodes.contains(a) && nodes.contains(b) {
                self.active[id] = true;
            }
        }
    }

    pub fn neighbors(&self, node: usize) -> Vec<usize> {
        self.incident[node].iter().map(|e| self.other_end(*e, node)).collect()
    }

    fn mirror_edge(&self, edge: usize, map: &[usize]) -> usize {
        let (a, b) = self.edges[edge];
        self.edge_between(map[a], map[b]).unwrap()
    }

    pub fn randomize(&mut self, density: Option<f32>) {
        let density = density.unwrap_or(self.rule.seed_density);
        for e in self.active.iter_mut() {
            *e = gen_range(0.0, 1.0) < density;
        }
    }

    /// Random state with left-right and top-bottom mirror symmetry. The rule only looks at
    /// the mesh's connectivity, so the symmetry survives as the automaton runs.
    pub fn randomize_symmetric(&mut self, density: Option<f32>) {
        let density = density.unwrap_or(self.rule.seed_density);
        let random: Vec<bool> = (0..self.edges.len()).map(|_| gen_range(0.0, 1.0) < density).collect();
        for id in 0..self.edges.len() {
            let mx = self.mirror_edge(id, &self.mirror_x);
            let my = self.mirror_edge(id, &self.mirror_y);
            let mxy = self.mirror_edge(mx, &self.mirror_y);
            self.active[id] = random[id.min(mx).min(my).min(mxy)];
        }
    }

    /// Displace every node at random by up to `amount` * spacing (keep it below about 0.25
    /// so that no triangle of the mesh can flip). The displacement is mirror-symmetric.
    pub fn set_jitter(&mut self, amount: f32) {
        let mut done = vec![false; self.nodes.len()];
        for n in 0..self.nodes.len() {
            if done[n] {
                continue;
            }
            let (mx, my) = (self.mirror_x[n], self.mirror_y[n]);
            let mxy = self.mirror_y[mx];
            let angle = gen_range(0.0, std::f32::consts::TAU);
            let mut d = vec2(angle.cos(), angle.sin()) * gen_range(0.0, 1.0f32).sqrt() * amount * self.spacing;
            if mx == n {
                d.x = 0.0;
            }
            if my == n {
                d.y = 0.0;
            }
            for (node, sign) in [(n, vec2(1., 1.)), (mx, vec2(-1., 1.)), (my, vec2(1., -1.)), (mxy, vec2(-1., -1.))] {
                self.nodes[node] = self.home[node] + d * sign;
                done[node] = true;
            }
        }
    }

    pub fn step(&mut self) {
        let next = (0..self.edges.len())
            .map(|id| {
                let n = self.adjacent[id].iter().filter(|other| self.active[**other]).count();
                let applies = if self.active[id] { self.rule.survive } else { self.rule.birth };
                applies & (1 << n) != 0
            })
            .collect();
        self.active = next;
    }

    /// Flip the substrate edge whose midpoint is nearest to `pos`.
    pub fn flip_nearest_edge(&mut self, pos: Vec2) {
        let dist = |&(a, b): &(usize, usize)| ((self.nodes[a] + self.nodes[b]) / 2.0).distance(pos);
        let nearest = self
            .edges
            .iter()
            .enumerate()
            .min_by(|(_, p), (_, q)| dist(p).partial_cmp(&dist(q)).unwrap());
        if let Some((id, edge)) = nearest {
            if dist(edge) < self.spacing {
                self.active[id] = !self.active[id];
            }
        }
    }

    /// The substrate and the active graph as plain lines and dots.
    pub fn draw_graph(&self, active_color: Color, substrate_color: Option<Color>) {
        for (id, &(a, b)) in self.edges.iter().enumerate() {
            let color = if self.active[id] {
                active_color
            } else if let Some(c) = substrate_color {
                c
            } else {
                continue;
            };
            let thickness = if self.active[id] { 2.5 } else { 1.0 };
            draw_line(self.nodes[a].x, self.nodes[a].y, self.nodes[b].x, self.nodes[b].y, thickness, color);
        }
        for (node, list) in self.incident.iter().enumerate() {
            if list.iter().any(|e| self.active[*e]) {
                draw_circle(self.nodes[node].x, self.nodes[node].y, 3.5, active_color);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mesh() -> Network {
        Network::triangular(Rect::new(0., 0., 400., 300.), 40.)
    }

    #[test]
    fn interior_nodes_have_six_neighbours() {
        let net = mesh();
        let degrees: Vec<usize> = net.incident.iter().map(|l| l.len()).collect();
        assert_eq!(*degrees.iter().max().unwrap(), 6);
        assert!(*degrees.iter().min().unwrap() >= 2);
        // every edge is as long as the spacing
        for &(a, b) in &net.edges {
            assert!((net.nodes[a].distance(net.nodes[b]) - 40.0).abs() < 1e-3);
        }
    }

    #[test]
    fn edges_around_a_node_are_in_angular_order() {
        let net = mesh();
        for (node, list) in net.incident.iter().enumerate() {
            let angles: Vec<f32> = list
                .iter()
                .map(|e| {
                    let d = net.nodes[net.other_end(*e, node)] - net.nodes[node];
                    d.y.atan2(d.x)
                })
                .collect();
            assert!(angles.windows(2).all(|w| w[0] < w[1]));
        }
    }

    #[test]
    fn mirrors_map_edges_to_edges() {
        let net = mesh();
        for id in 0..net.edges.len() {
            let mx = net.mirror_edge(id, &net.mirror_x);
            assert_eq!(net.mirror_edge(mx, &net.mirror_x), id);
            let my = net.mirror_edge(id, &net.mirror_y);
            assert_eq!(net.mirror_edge(my, &net.mirror_y), id);
        }
    }

    #[test]
    fn symmetric_seed_stays_symmetric() {
        let mut net = mesh();
        net.rule = NetRule {
            name: "test",
            birth: mask(&[3, 4]),
            survive: mask(&[2, 3, 4, 5]),
            seed_density: 0.3,
        };
        net.randomize_symmetric(None);
        for _ in 0..6 {
            net.step();
        }
        assert!(net.active.iter().any(|a| *a));
        for id in 0..net.edges.len() {
            assert_eq!(net.active[id], net.active[net.mirror_edge(id, &net.mirror_x)]);
            assert_eq!(net.active[id], net.active[net.mirror_edge(id, &net.mirror_y)]);
        }
    }

    #[test]
    fn step_counts_edges_sharing_a_node() {
        let mut net = mesh();
        // a lone edge has no active neighbours: it survives only if S0 is set
        net.rule = NetRule { name: "test", birth: mask(&[1]), survive: 0, seed_density: 0.0 };
        let center = net.node_id(4, 2);
        let first = net.incident[center][0];
        net.active[first] = true;
        net.step();
        assert!(!net.active[first]);
        // B1: exactly the 10 edges that touched it are born
        assert_eq!(net.active.iter().filter(|a| **a).count(), 10);
    }

    #[test]
    fn jitter_keeps_the_mesh_mirror_symmetric() {
        let mut net = mesh();
        net.set_jitter(0.2);
        let center_x = (net.home[0].x + net.home[net.mirror_x[0]].x) / 2.0;
        for n in 0..net.nodes.len() {
            let m = net.nodes[net.mirror_x[n]];
            assert!((net.nodes[n].x - center_x + (m.x - center_x)).abs() < 1e-3);
            assert!((net.nodes[n].y - m.y).abs() < 1e-3);
            assert!(net.nodes[n].distance(net.home[n]) <= 0.2 * 40.0 + 1e-3);
        }
    }
}
