use graphics;
use graphics::{clear, rectangle};
use piston::window::WindowSettings;
use piston_window::{PistonWindow, Transformed, UpdateEvent, Window, AdvancedWindow, Key, Button, PressEvent};
use rand::prelude::*;

use rand::thread_rng;
use std::slice::IterMut;

const CELL_COLOR: [f32; 4] = [0.0, 0.0, 0.0, 1.0];
const BG_COLOR: [f32; 4] = [1.0, 1.0, 1.0, 1.0];

const CELL_WIDTH: usize = 100;
const CELL_HEIGHT: usize = 100;
const CELL_SCALE: f64 = 1.0;
const SCREEN_SCALE: f64 = 8.0;

const SNAPSHOT_LIMIT: usize = 10000;

const SEED_BOUNDING_BOX: usize = (CELL_WIDTH + CELL_HEIGHT) / 2 / 5;

const SEED_SPAWN_RATE: f64 = 1.0 / 4.2;

const CELL_TICK_RATE: f64 = 60.0; // Tick rate in Hz

type CellGrid = [bool; CELL_WIDTH * CELL_HEIGHT];

#[derive(Clone)]
struct CellState {
    pub x: usize,
    pub y: usize,
    pub alive: bool,
    pub neighbors_alive: u8,
}

type CellRule = Box<dyn FnMut(&CellGrid, CellState) -> bool>;

struct CellRules {
    rules: Vec<CellRule>,
}

impl CellRules where  {
    pub fn new() -> Self {
        CellRules { rules: vec![] }
    }
    pub fn add_rule<F: 'static>(&mut self, f: F) where F: FnMut(&CellGrid, CellState) -> bool {
        self.rules.push(Box::new(f));
    }
    pub fn iter_rules(&mut self) -> IterMut<'_, CellRule> {
        self.rules.iter_mut()
    }
}

fn get_x_y(i: usize) -> (usize, usize) {
    (i % CELL_WIDTH, i / CELL_HEIGHT)
}

fn get_idx(x: usize, y: usize) -> usize {
    y * CELL_HEIGHT + x
}

fn cell_generation_tick(mut cells: CellGrid, rules: &mut CellRules) -> CellGrid {
    for i in 0..cells.len() {
        let (x, y) = get_x_y(i);
        let mut live_count: u8 = 0;

        for _x in -1..2 as isize {
            for _y in -1..2 as isize {
                if !(_x == 0 && _y == 0) {
                    let x_m = _x + x as isize;
                    let y_m = _y + y as isize;
                    if x_m >= 0 && x_m < CELL_WIDTH as isize && y_m >= 0 && y_m < CELL_HEIGHT as isize {
                        // within range
                        let idx = get_idx(x_m as usize, y_m as usize);
                        if cells[idx] { live_count += 1; }
                    }
                }
            }
        }

        let mut state = CellState {
            x,
            y,
            alive: cells[i],
            neighbors_alive: live_count,
        };

        for rule in rules.iter_rules() {
            state.alive = rule(&cells, state.clone());
        }

        cells[i] = state.alive;
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
    else if tv as usize > max { max }
    else { tv as usize }
}

fn conway_rules(_grid: &CellGrid, cell: CellState) -> bool {
    // Conway rules.
    if cell.alive && cell.neighbors_alive > 3 { false }
    else if cell.alive && cell.neighbors_alive < 2 { false }
    else if !cell.alive {
        if cell.neighbors_alive == 3 { true }
        else { cell.alive }
    } else {
        cell.alive
    }
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
    let mut cell_rules = CellRules::new();

    // ADD RULES HERE
        cell_rules.add_rule(conway_rules);
    //

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
                        snapshots.push(cell_generation_tick(snapshots.last().copied().expect("NO SNAPSHOT"), &mut cell_rules));
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
                        skip_index = get_next_skip_index(1, skip_index, SNAPSHOT_LIMIT - 1);
                        if skip_index >= snapshots.len() || skip_index == snapshots.len() - 1 {
                            snapshots.push(cell_generation_tick(snapshots.last().copied().expect("NO SNAPSHOT"), &mut cell_rules));
                            if snapshots.len() > SNAPSHOT_LIMIT {
                                snapshots.remove(0);
                            }
                        }
                    }
                },
                Key::Up => {
                    // skip to end
                    skip_index = snapshots.len() - 1;
                },
                Key::Down => {
                    // skip to start
                    skip_index = 0;
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
