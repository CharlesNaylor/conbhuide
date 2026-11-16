/*
 * Conbhuide: automata whose patterns are drawn as Celtic knotwork.
 *
 * Two automata are available:
 *   - Network: edges of a triangular mesh switch on and off (network.rs), and the knot is
 *     drawn around the resulting graph (mercat.rs)
 *   - Life: a Life-like cellular automaton (life.rs) whose live regions are drawn as
 *     knotwork panels (knot.rs)
 *
 *   cargo run                                   interactive
 *   cargo run -- --snapshot out.png [what] [n]  render one frame to a PNG and exit
 *       what: "net" or "net:<rule name prefix>"  network automaton, symmetric random seed
 *             "netshapes", "netfull"             hand-placed network test figures
 *             "shapes"                           hand-placed Life test figures
 *             a Life rule name prefix such as "life", "maze", "day", "coral"
 *       n:    generations to run before the picture is taken (default 30)
 *       add --jitter to displace the mesh nodes at random
 */
pub mod knot;
pub mod life;
pub mod mercat;
pub mod network;
use crate::knot::{draw_knot, KnotStyle};
use crate::life::{CellMatrix, RULES};
use crate::mercat::draw_network_knot;
use crate::network::{Network, NET_RULES};
use macroquad::prelude::*;
use macroquad::ui::{hash, root_ui};

const CELL_SIZE: u16 = 24;
const NODE_SPACING: f32 = 44.0;
const JITTER: f32 = 0.14;
const FRAME_TOP_LEFT: Vec2 = vec2(0., 44.);
const FAINT: Color = Color::new(0.2, 0.2, 0.25, 1.0);

fn window_conf() -> Conf {
    Conf {
        window_title: "Conbhuide".to_owned(),
        window_width: 960,
        window_height: 684,
        sample_count: 4,
        ..Default::default()
    }
}

#[derive(Clone, Copy, PartialEq)]
enum Mode {
    Network,
    Life,
}

struct App {
    mode: Mode,
    cells: CellMatrix,
    net: Network,
    life_rule: usize,
    net_rule: usize,
    jitter: bool,
    /// false shows the raw automaton (cells or graph) instead of the knot
    show_knot: bool,
    /// show the raw automaton faintly under the knot
    show_under: bool,
    life_style: KnotStyle,
    net_style: KnotStyle,
}

impl App {
    fn new() -> Self {
        let size = vec2(screen_width(), screen_height()) - FRAME_TOP_LEFT;
        let area = Rect::new(FRAME_TOP_LEFT.x, FRAME_TOP_LEFT.y, size.x, size.y);
        App {
            mode: Mode::Network,
            cells: CellMatrix::new(size, CELL_SIZE, Some(FRAME_TOP_LEFT)),
            net: Network::triangular(area, NODE_SPACING),
            life_rule: 0,
            net_rule: 0,
            jitter: false,
            show_knot: true,
            show_under: false,
            life_style: KnotStyle::default(),
            net_style: KnotStyle {
                strand_width: 0.2,
                gap: 0.05,
                ..KnotStyle::default()
            },
        }
    }

    fn state(&self) -> Vec<bool> {
        match self.mode {
            Mode::Network => self.net.active.clone(),
            Mode::Life => self.cells.cells().to_vec(),
        }
    }

    fn step(&mut self) {
        match self.mode {
            Mode::Network => self.net.step(),
            Mode::Life => self.cells.step(),
        }
    }

    fn randomize(&mut self, symmetric: bool) {
        match (self.mode, symmetric) {
            (Mode::Network, true) => self.net.randomize_symmetric(None),
            (Mode::Network, false) => self.net.randomize(None),
            (Mode::Life, true) => self.cells.randomize_symmetric(None),
            (Mode::Life, false) => self.cells.randomize(None),
        }
    }

    fn clear(&mut self) {
        match self.mode {
            Mode::Network => self.net.clear(),
            Mode::Life => self.cells.clear(),
        }
    }

    fn set_rule(&mut self, index: usize) {
        match self.mode {
            Mode::Network => {
                self.net_rule = index % NET_RULES.len();
                self.net.rule = NET_RULES[self.net_rule];
            }
            Mode::Life => {
                self.life_rule = index % RULES.len();
                self.cells.rule = RULES[self.life_rule];
            }
        }
    }

    fn next_rule(&mut self) {
        match self.mode {
            Mode::Network => self.set_rule(self.net_rule + 1),
            Mode::Life => self.set_rule(self.life_rule + 1),
        }
    }

    fn rule_name(&self) -> &'static str {
        match self.mode {
            Mode::Network => NET_RULES[self.net_rule].name,
            Mode::Life => RULES[self.life_rule].name,
        }
    }

    fn set_jitter(&mut self, on: bool) {
        self.jitter = on;
        self.net.set_jitter(if on { JITTER } else { 0.0 });
    }

    fn flip(&mut self, pos: Vec2) {
        match self.mode {
            Mode::Network => self.net.flip_nearest_edge(pos),
            Mode::Life => self.cells.flip_cell(pos),
        }
    }

    fn draw(&self) {
        clear_background(BLACK);
        match (self.mode, self.show_knot) {
            (Mode::Network, true) => {
                if self.show_under {
                    self.net.draw_graph(FAINT, Some(Color::new(0.1, 0.1, 0.12, 1.0)));
                }
                draw_network_knot(&self.net, &self.net_style);
            }
            (Mode::Network, false) => self.net.draw_graph(WHITE, Some(FAINT)),
            (Mode::Life, true) => {
                if self.show_under {
                    self.cells.draw(FAINT, None);
                }
                draw_knot(&self.cells, &self.life_style);
            }
            (Mode::Life, false) => self.cells.draw(BLACK, Some(WHITE)),
        }
    }
}

struct Snapshot {
    path: String,
    what: String,
    steps: u32,
    jitter: bool,
}

fn snapshot_from_args() -> Option<Snapshot> {
    let mut args: Vec<String> = std::env::args().collect();
    let jitter = args.iter().any(|a| a == "--jitter");
    args.retain(|a| a != "--jitter");
    let at = args.iter().position(|a| a == "--snapshot")?;
    Some(Snapshot {
        path: args.get(at + 1)?.clone(),
        what: args.get(at + 2).cloned().unwrap_or("net".to_owned()),
        steps: args.get(at + 3).and_then(|s| s.parse().ok()).unwrap_or(30),
        jitter,
    })
}

fn stamp(cells: &mut CellMatrix, origin: (u16, u16), rows: &[&str]) {
    for (y, row) in rows.iter().enumerate() {
        for (x, ch) in row.chars().enumerate() {
            if ch == '#' {
                cells.set(origin.0 + x as u16, origin.1 + y as u16, true);
            }
        }
    }
}

fn life_test_shapes(cells: &mut CellMatrix) {
    /* small figures whose knots are easy to check by eye */
    cells.clear();
    stamp(cells, (1, 1), &["#"]);
    stamp(cells, (3, 1), &["##"]);
    stamp(cells, (6, 1), &["#", "#", "#"]);
    stamp(cells, (8, 1), &["##", "##"]);
    stamp(cells, (11, 1), &["#.", "##"]);
    stamp(cells, (14, 1), &[".#.", "###", ".#."]);
    stamp(cells, (18, 1), &["###", "#.#", "###"]);
    stamp(cells, (1, 6), &["###", "###", "###"]);
    stamp(cells, (5, 6), &["######", "######", "######", "######"]);
    stamp(cells, (12, 6), &["#####", "#...#", "#.#.#", "#...#", "#####"]);
    stamp(cells, (18, 6), &["#..", "##.", ".##", "..#"]);
}

fn net_test_shapes(net: &mut Network) {
    /* small graphs whose knots are easy to check by eye */
    net.clear();
    // one edge, a path, a triangle
    net.activate_among(&[net.node_id(0, 0), net.node_id(1, 0)]);
    net.activate_among(&[net.node_id(3, 0), net.node_id(4, 0)]);
    net.activate_among(&[net.node_id(4, 0), net.node_id(5, 0)]);
    net.activate_among(&[net.node_id(5, 0), net.node_id(6, 0)]);
    net.activate_among(&[net.node_id(8, 0), net.node_id(9, 0), net.node_id(8, 1)]);
    // a bent path
    net.activate_among(&[net.node_id(11, 0), net.node_id(12, 0)]);
    net.activate_among(&[net.node_id(12, 0), net.node_id(12, 1)]);
    net.activate_among(&[net.node_id(12, 1), net.node_id(14, 2)]);
    net.activate_among(&[net.node_id(12, 1), net.node_id(13, 2)]);
    // a star, a hexagon, a wheel
    let hub = net.node_id(2, 4);
    for spoke in net.neighbors(hub) {
        net.activate_among(&[hub, spoke]);
    }
    let ring = net.neighbors(net.node_id(6, 4));
    net.activate_among(&ring);
    let mut wheel = net.neighbors(net.node_id(10, 4));
    wheel.push(net.node_id(10, 4));
    net.activate_among(&wheel);
    // a solid patch and a larger ring
    let center = net.node_id(4, 9);
    let mut patch = vec![center];
    for n in net.neighbors(center) {
        patch.push(n);
        patch.extend(net.neighbors(n));
    }
    net.activate_among(&patch);
    let center = net.node_id(11, 9);
    let inner = net.neighbors(center);
    let mut outer = vec![];
    for n in &inner {
        outer.extend(net.neighbors(*n));
    }
    outer.retain(|n| *n != center && !inner.contains(n));
    net.activate_among(&outer);
}

fn save_png(path: &str, draw: impl Fn()) {
    /* Draw into an offscreen target and write it out. (Reading the window back is not
     * reliable: after a buffer swap its contents are undefined on some platforms.) */
    #[cfg(not(target_arch = "wasm32"))]
    {
        let (w, h) = (screen_width(), screen_height());
        let target = render_target(w as u32, h as u32);
        let mut camera = Camera2D::from_display_rect(Rect::new(0., 0., w, h));
        camera.render_target = Some(target.clone());
        set_camera(&camera);
        draw();
        set_default_camera();
        target.texture.get_texture_data().export_png(path);
        info!("Saved {}", path);
    }
    #[cfg(target_arch = "wasm32")]
    {
        let _ = (path, draw);
    }
}

fn prepare_snapshot(app: &mut App, snap: &Snapshot) {
    macroquad::rand::srand(42);
    if snap.jitter {
        app.set_jitter(true);
    }
    let what = snap.what.to_lowercase();
    let rule_at = |names: Vec<&str>, prefix: &str| {
        names
            .iter()
            .position(|n| n.to_lowercase().starts_with(prefix))
            .unwrap_or(0)
    };
    match what.as_str() {
        "shapes" => {
            app.mode = Mode::Life;
            life_test_shapes(&mut app.cells);
        }
        "netshapes" => net_test_shapes(&mut app.net),
        "netfull" => app.net.fill(),
        _ => {
            if what == "net" || what.starts_with("net:") {
                let prefix = what.strip_prefix("net:").unwrap_or("");
                app.set_rule(rule_at(NET_RULES.iter().map(|r| r.name).collect(), prefix));
            } else {
                app.mode = Mode::Life;
                app.set_rule(rule_at(RULES.iter().map(|r| r.name).collect(), &what));
            }
            app.randomize(true);
            for _ in 0..snap.steps {
                app.step();
            }
        }
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut app = App::new();
    let mut running: bool = true;
    let mut fps: f32 = 3.0;
    let mut step_time: f64 = 0.0;

    let snapshot = snapshot_from_args();
    let mut frames_drawn: u32 = 0;
    if let Some(snap) = &snapshot {
        running = false;
        prepare_snapshot(&mut app, snap);
    } else {
        macroquad::rand::srand(macroquad::miniquad::date::now() as u64);
        app.set_rule(0);
        app.randomize(true);
    }

    loop {
        if let Some(snap) = &snapshot {
            // let the window settle for a couple of frames first
            if frames_drawn >= 3 {
                save_png(&snap.path, || app.draw());
                break;
            }
        }

        // ui
        let mut toggle_view = is_key_pressed(KeyCode::C);
        let mut toggle_running = is_key_pressed(KeyCode::Space);
        let mut next_rule = is_key_pressed(KeyCode::N);
        let mut randomize = is_key_pressed(KeyCode::R);
        let mut randomize_symmetric = is_key_pressed(KeyCode::S);
        let mut switch_mode = is_key_pressed(KeyCode::M);
        let mut clear = is_key_pressed(KeyCode::X);
        let raw_name = if app.mode == Mode::Network { "Graph" } else { "Cells" };
        toggle_view |= root_ui().button(vec2(4., 2.), if app.show_knot { raw_name } else { "Knot" });
        toggle_running |= root_ui().button(vec2(58., 2.), if running { "Pause" } else { "Play" });
        randomize |= root_ui().button(vec2(110., 2.), "Random");
        randomize_symmetric |= root_ui().button(vec2(172., 2.), "Symmetric");
        clear |= root_ui().button(vec2(254., 2.), "Clear");
        switch_mode |= root_ui().button(
            vec2(306., 2.),
            if app.mode == Mode::Network { "Mode: Network" } else { "Mode: Life" },
        );
        next_rule |= root_ui().button(vec2(416., 2.), format!("Rule: {}", app.rule_name()).as_str());
        root_ui().slider(hash!(), "FPS", 0.5..30.0, &mut fps);

        if toggle_view {
            app.show_knot = !app.show_knot;
        }
        if toggle_running {
            running = !running;
        }
        if switch_mode {
            app.mode = if app.mode == Mode::Network { Mode::Life } else { Mode::Network };
            if !app.state().iter().any(|on| *on) {
                app.randomize(true);
            }
        }
        if next_rule {
            app.next_rule();
        }
        if clear {
            // pause as well, so the blank board stays blank until something is drawn on it
            app.clear();
            running = false;
        }
        if randomize {
            app.randomize(false);
        }
        if randomize_symmetric {
            app.randomize(true);
        }
        if is_key_pressed(KeyCode::J) {
            app.set_jitter(!app.jitter);
        }
        if is_key_pressed(KeyCode::G) {
            app.show_under = !app.show_under;
        }
        if is_key_pressed(KeyCode::P) {
            save_png("conbhuide.png", || app.draw());
        }
        if is_mouse_button_pressed(MouseButton::Left) {
            let mouse_pos = Vec2::from(mouse_position());
            if mouse_pos.y > FRAME_TOP_LEFT.y {
                app.flip(mouse_pos);
            }
        }

        // simulation
        if running && get_time() > step_time + (1.0 / fps) as f64 {
            app.step();
            step_time = get_time();
        }

        app.draw();

        frames_drawn += 1;
        next_frame().await
    }
}
