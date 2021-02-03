use graphics;
use graphics::{clear, rectangle};
use piston::window::WindowSettings;
use piston_window::{PistonWindow, Transformed, UpdateEvent, Window, AdvancedWindow, Key, Button, PressEvent};
use rand::prelude::*;

use rand::thread_rng;

const CELL_COLOR: [f32; 4] = [0.0, 0.0, 0.0, 1.0];
const BG_COLOR: [f32; 4] = [1.0, 1.0, 1.0, 1.0];

const CELL_WIDTH: usize = 100;
const CELL_HEIGHT: usize = 100;
const CELL_SCALE: f64 = 0.75;
const SCREEN_SCALE: f64 = 8.0;

const SNAPSHOT_LIMIT: usize = usize::MAX - 1;

const SEED_BOUNDING_BOX: usize = CELL_WIDTH / 2;

const SEED_SPAWN_RATE: f64 = 0.45 / 4.2;

const CELL_TICK_RATE: f64 = 60.0; // Tick rate in Hz

type CellGrid = [bool; CELL_WIDTH * CELL_HEIGHT];

fn get_x_y(i: usize) -> (usize, usize) {
    (i % CELL_WIDTH, i / CELL_HEIGHT)
}

fn get_idx(x: usize, y: usize) -> usize {
    y * CELL_HEIGHT + x
}

fn cell_generation_tick(mut cells: CellGrid) -> CellGrid {
    for i in 0..cells.len() {
        let (x, y) = get_x_y(i);
        let mut live_count = 0;

        let (x_a, x_a_o) = x.overflowing_sub(1);
        let (x_b, x_b_o) = x.overflowing_add(1);
        let (y_a, y_a_o) = y.overflowing_sub(1);
        let (y_b, y_b_o) = y.overflowing_add(1);

        let mut v: Vec<usize> = vec![];

        if !x_a_o {
            v.push(get_idx(x_a, y));
        }
        if !x_b_o {
            v.push(get_idx(x_b, y));
        }
        if !y_a_o {
            v.push(get_idx(x, y_a));
        }
        if !y_b_o {
            v.push(get_idx(x, y_b));
        }
        if !x_a_o && !y_a_o {
            v.push(get_idx(x_a, y_a));
        }
        if !x_b_o && !y_a_o {
            v.push(get_idx(x_b, y_a));
        }
        if !x_a_o && !y_b_o {
            v.push(get_idx(x_a, y_b));
        }
        if !x_b_o && !y_b_o {
            v.push(get_idx(x_b, y_b));
        }

        for idx in v {
            if idx < cells.len() {
                if cells[idx] {
                    live_count += 1;
                }
            }
        }

        let state = cells[i];
        if state {
            if live_count < 2 {
                cells[i] = false
            } else if live_count > 3 {
                cells[i] = false
            }
        } else {
            if live_count == 3 {
                cells[i] = true;
            }
        }
    }
    cells
}

fn seed_cells(mut rng: ThreadRng) -> CellGrid {
    let mut cells: CellGrid = [false; CELL_WIDTH * CELL_HEIGHT];
    for i in 0..cells.len() {
        let (x, y) = get_x_y(i);
        if (CELL_WIDTH / 2).wrapping_sub(x) < SEED_BOUNDING_BOX
            && (CELL_HEIGHT / 2).wrapping_sub(y) < SEED_BOUNDING_BOX
        {
            cells[i] = rng.gen_bool(SEED_SPAWN_RATE);
        }
    }
    cells
}

fn get_next_skip_index(dir: isize, i: usize, max: usize) -> usize {
    let tv = dir + i as isize;
    if tv < 0 { 0 }
    else if tv > max as isize { max }
    else { tv as usize }
}

fn main() {
    let rng = thread_rng();
    let mut window: PistonWindow = WindowSettings::new(
        "Cells",
        (
            CELL_WIDTH as f64 * SCREEN_SCALE,
            CELL_HEIGHT as f64 * SCREEN_SCALE,
        ),
    )
    .exit_on_esc(true)
    .build()
    .unwrap_or_else(|e| panic!("Failed to build PistonWindow {}", e));
    let win_size = window.window.size();
    let pos_x_m: f64 = win_size.width / CELL_WIDTH as f64;
    let pos_y_m: f64 = win_size.height / CELL_HEIGHT as f64;
    let rect = rectangle::square(0.0, 0.0, pos_x_m * CELL_SCALE);
    let mut ft: f64 = 0.0;
    let mut snapshots: Vec<[bool; CELL_HEIGHT * CELL_WIDTH]> = vec![];
    let mut should_play: bool = false;
    let mut skip_index: usize = 0;
    snapshots.push(seed_cells(rng.clone()));
    while let Some(e) = window.next() {
        if let Some(args) = e.update_args() {
            ft += args.dt;
            if ft >= (1000.0 / CELL_TICK_RATE / 1000.0) && should_play {
                ft = 0.0;
                if skip_index < snapshots.len() - 1 {
                    skip_index += 1;
                } else {
                    // update
                    {
                        snapshots.push(cell_generation_tick(snapshots.last().copied().expect("NO SNAPSHOT")));
                        if snapshots.len() > SNAPSHOT_LIMIT {
                            snapshots.remove(0);
                        }
                        skip_index = snapshots.len() - 1; // Set the skip index since we should be *playing*
                    }
                }
            }
            window.set_title(format!("Cells - {}", { if should_play { "PLAY" } else { "PAUSED" } }));
        } else if let Some(Button::Keyboard(k)) = e.press_args() {
            match k {
                Key::Space => {
                    should_play = !should_play;
                },
                Key::Right => {
                    if !should_play {
                        // Don't move through while playing
                        let old = skip_index;
                        skip_index = get_next_skip_index(1, skip_index, snapshots.len() - 1);
                        if old == skip_index {
                            snapshots.push(cell_generation_tick(snapshots.last().copied().expect("NO SNAPSHOT")));
                            if snapshots.len() > SNAPSHOT_LIMIT {
                                snapshots.remove(0);
                            }
                        }
                    }
                },
                Key::Left => {
                    if !should_play {
                        // Don't move through while playing
                        skip_index = get_next_skip_index(-1, skip_index, snapshots.len() - 1);
                    }
                },
                _ => {}
            }
        }
        window.draw_2d(&e, |_c, g, _d| {
            let cells = snapshots.get(skip_index).expect("NO SNAPSHOT?");
            clear(BG_COLOR, g);
            for i in 0..cells.len() {
                let (x, y) = get_x_y(i);
                if cells[i] {
                    rectangle(
                        CELL_COLOR,
                        rect,
                        _c.transform.trans(x as f64 * pos_x_m, y as f64 * pos_y_m),
                        g,
                    );
                }
            }
        });
    }
}
