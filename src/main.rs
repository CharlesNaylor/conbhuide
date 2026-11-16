/*
 * Conbhuide: a Life-like cellular automaton whose live regions are drawn as Celtic knotwork.
 *
 *   cargo run                                   interactive
 *   cargo run -- --snapshot out.png [what] [n]  render one frame to a PNG and exit
 *       what: "shapes" (hand-placed test figures, the default) or a rule name prefix
 *             such as "life", "maze", "day", "coral" (symmetric random seed)
 *       n:    generations to run before the picture is taken (default 30)
 */
pub mod knot;
pub mod life;
use crate::knot::{draw_knot, KnotStyle};
use crate::life::{CellMatrix, RULES};
use macroquad::prelude::*;
use macroquad::ui::{hash, root_ui};

const CELL_SIZE: u16 = 24;
/// reseed after this many steps without change (still life, period-2 oscillator or empty)
const STAGNANT_STEPS: u32 = 12;
const FRAME_TOP_LEFT: Vec2 = vec2(0., 44.);

fn window_conf() -> Conf {
    Conf {
        window_title: "Conbhuide".to_owned(),
        window_width: 960,
        window_height: 684,
        sample_count: 4,
        ..Default::default()
    }
}

struct Snapshot {
    path: String,
    what: String,
    steps: u32,
}

fn snapshot_from_args() -> Option<Snapshot> {
    let args: Vec<String> = std::env::args().collect();
    let at = args.iter().position(|a| a == "--snapshot")?;
    Some(Snapshot {
        path: args.get(at + 1)?.clone(),
        what: args.get(at + 2).cloned().unwrap_or("shapes".to_owned()),
        steps: args.get(at + 3).and_then(|s| s.parse().ok()).unwrap_or(30),
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

fn test_shapes(cells: &mut CellMatrix) {
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

fn draw_scene(cells: &CellMatrix, style: &KnotStyle, show_knot: bool, show_cells_under_knot: bool) {
    clear_background(style.background);
    if show_knot {
        if show_cells_under_knot {
            cells.draw(Color::new(0.2, 0.2, 0.25, 1.0), None);
        }
        draw_knot(cells, style);
    } else {
        cells.draw(BLACK, Some(WHITE));
    }
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

#[macroquad::main(window_conf)]
async fn main() {
    let grid_size = vec2(screen_width(), screen_height()) - FRAME_TOP_LEFT;
    let mut cells: CellMatrix = CellMatrix::new(grid_size, CELL_SIZE, Some(FRAME_TOP_LEFT));
    let style = KnotStyle::default();

    let mut running: bool = true;
    let mut show_knot: bool = true;
    let mut show_cells_under_knot: bool = false;
    let mut rule_ind: usize = 0;
    let mut fps: f32 = 4.0;
    let mut step_time: f64 = 0.0;
    let mut previous: Vec<bool> = cells.cells().to_vec();
    let mut stagnant: u32 = 0;

    let snapshot = snapshot_from_args();
    let mut frames_drawn: u32 = 0;
    if let Some(snap) = &snapshot {
        running = false;
        macroquad::rand::srand(42);
        if snap.what == "shapes" {
            test_shapes(&mut cells);
        } else {
            let what = snap.what.to_lowercase();
            rule_ind = RULES
                .iter()
                .position(|r| r.name.to_lowercase().starts_with(&what))
                .unwrap_or(0);
            cells.rule = RULES[rule_ind];
            cells.randomize_symmetric(None);
            for _ in 0..snap.steps {
                cells.step();
            }
        }
    } else {
        macroquad::rand::srand(macroquad::miniquad::date::now() as u64);
        cells.randomize_symmetric(None);
    }

    loop {
        if let Some(snap) = &snapshot {
            // let the window settle for a couple of frames first
            if frames_drawn >= 3 {
                save_png(&snap.path, || draw_scene(&cells, &style, true, false));
                break;
            }
        }

        // ui
        let mut toggle_view = is_key_pressed(KeyCode::C);
        let mut toggle_running = is_key_pressed(KeyCode::Space);
        let mut next_rule = is_key_pressed(KeyCode::N);
        let mut randomize = is_key_pressed(KeyCode::R);
        let mut randomize_symmetric = is_key_pressed(KeyCode::S);
        toggle_view |= root_ui().button(vec2(4., 2.), if show_knot { "Cells" } else { "Knot" });
        toggle_running |= root_ui().button(vec2(54., 2.), if running { "Pause" } else { "Play" });
        randomize |= root_ui().button(vec2(104., 2.), "Random");
        randomize_symmetric |= root_ui().button(vec2(164., 2.), "Symmetric");
        next_rule |= root_ui().button(vec2(244., 2.), format!("Rule: {}", RULES[rule_ind].name).as_str());
        root_ui().slider(hash!(), "FPS", 0.5..30.0, &mut fps);

        if toggle_view {
            show_knot = !show_knot;
        }
        if toggle_running {
            running = !running;
        }
        if next_rule {
            rule_ind = (rule_ind + 1) % RULES.len();
            cells.rule = RULES[rule_ind];
        }
        if randomize {
            cells.randomize(None);
        }
        if randomize_symmetric {
            cells.randomize_symmetric(None);
        }
        if is_key_pressed(KeyCode::G) {
            show_cells_under_knot = !show_cells_under_knot;
        }
        if is_key_pressed(KeyCode::P) {
            save_png("conbhuide.png", || {
                draw_scene(&cells, &style, show_knot, show_cells_under_knot)
            });
        }
        if is_mouse_button_pressed(MouseButton::Left) {
            let mouse_pos = Vec2::from(mouse_position());
            if mouse_pos.y > FRAME_TOP_LEFT.y {
                cells.flip_cell(mouse_pos);
            }
        }

        // simulation
        if running && get_time() > step_time + (1.0 / fps) as f64 {
            let before = cells.cells().to_vec();
            cells.step();
            step_time = get_time();
            // a display should keep moving: start again once the pattern has settled
            if cells.cells() == &before[..] || cells.cells() == &previous[..] {
                stagnant += 1;
            } else {
                stagnant = 0;
            }
            previous = before;
            if stagnant >= STAGNANT_STEPS {
                cells.randomize_symmetric(None);
                stagnant = 0;
            }
        }

        // drawing
        draw_scene(&cells, &style, show_knot, show_cells_under_knot);

        frames_drawn += 1;
        next_frame().await
    }
}
