extern crate rand;

use std::fs;
use std::path;

use ggez::graphics::{Color, DrawMode, Point2, Rect, Vector2};
use ggez::{graphics, timer, Context, GameResult};

use rand::{thread_rng, Rng};

const MODULES_PATH: &str = "./resources/modules.txt";

struct MainState {
    // Grids are stored from lowest visually to highest
    grids: Vec<Grid>,
    modules: Vec<[[Tile; GRID_WIDTH]; GRID_HEIGHT]>,
}

impl MainState {
    fn new(_ctx: &mut Context) -> GameResult<MainState> {
        let modules = parse_modules_file(&path::Path::new(MODULES_PATH)).unwrap();
        Ok(MainState {
            grids: vec![
                Grid::new_from_module((GRID_HEIGHT * 0) as f32, modules[1].clone()),
                Grid::new_from_module((GRID_HEIGHT * 1) as f32, modules[0].clone()),
                Grid::new_from_module(
                    (GRID_HEIGHT * 2) as f32,
                    rand::thread_rng().choose(&modules).unwrap().clone(),
                ),
            ],
            modules: modules,
        })
    }
}

impl ggez::event::EventHandler for MainState {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        const DESIRED_FPS: u32 = 60;
        // variable update
        while timer::check_update_time(ctx, DESIRED_FPS) {
            // fixed update
            for grid in &mut self.grids {
                grid.update();
            }
            self.grids[0].damage_tile(
                thread_rng().gen_range(0, GRID_WIDTH),
                thread_rng().gen_range(0, GRID_HEIGHT),
            );
            // self.grids[0].damage_tile(
            //     thread_rng().gen_range(0, GRID_WIDTH),
            //     thread_rng().gen_range(0, GRID_HEIGHT),
            // );
            // self.grids[0].damage_tile(
            //     thread_rng().gen_range(0, GRID_WIDTH),
            //     thread_rng().gen_range(0, GRID_HEIGHT),
            // );
            // self.grids[1].damage_tile(
            //     thread_rng().gen_range(0, GRID_WIDTH),
            //     thread_rng().gen_range(0, GRID_HEIGHT),
            // );
            self.grids[1].damage_tile(
                thread_rng().gen_range(0, GRID_WIDTH),
                thread_rng().gen_range(0, GRID_HEIGHT),
            );
            self.grids[2].damage_tile(
                thread_rng().gen_range(0, GRID_WIDTH),
                thread_rng().gen_range(0, GRID_HEIGHT),
            );
        }

        if self.grids[0].state == GridState::Dead {
            self.grids.remove(0);
            self.grids.push(Grid::new_from_module(
                GRID_HEIGHT as f32 * 3.0,
                rand::thread_rng().choose(&self.modules).unwrap().clone(),
            ));
            for (i, grid) in self.grids.iter_mut().enumerate() {
                let new_height = (i * GRID_HEIGHT) as f32;
                grid.state = GridState::Falling(new_height);
            }
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx);
        for grid in &mut self.grids {
            grid.draw(ctx)?;
        }
        graphics::present(ctx);
        Ok(())
    }
}

type WorldCoord = f32;
type GridCoord = usize;

pub const WORLD_WIDTH: WorldCoord = GRID_WIDTH as f32;
pub const WORLD_HEIGHT: WorldCoord = (GRID_HEIGHT * 3) as f32;
pub const GRID_WIDTH: GridCoord = 32;
pub const GRID_HEIGHT: GridCoord = 8;
pub const TILE_SIZE: WorldCoord = 1.0f32;
pub const TILE_MAX_HEALTH: usize = 5;

/// A grid contains the collidable tiles that our dynamic objects interact with
///
struct Grid {
    grid: [[Tile; GRID_WIDTH]; GRID_HEIGHT],
    world_offset: (WorldCoord, WorldCoord),
    state: GridState,
    total_tiles: usize, // Number of tiles alive at the start
    tiles_alive: usize, // Number of tiles still currently alive
}

impl Grid {
    fn new(height: WorldCoord) -> Grid {
        Grid::new_from_module(height, [[Tile::Solid(5); GRID_WIDTH]; GRID_HEIGHT])
    }

    fn new_from_module(height: WorldCoord, module: [[Tile; GRID_WIDTH]; GRID_HEIGHT]) -> Grid {
        let total_tiles = total_tiles(module);
        Grid {
            grid: module,
            world_offset: (0.0, height),
            state: GridState::Alive,
            total_tiles: total_tiles,
            tiles_alive: total_tiles,
        }
    }

    fn update(&mut self) {
        if self.percent_tiles_alive() < 0.15 {
            self.state = GridState::Dead;
        }

        if let GridState::Falling(goal_height) = self.state {
            self.world_offset.1 = goal_height;
            self.state = GridState::Alive;
        }
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        use self::Tile::*;
        for (j, row) in self.grid.iter().enumerate() {
            for (i, tile) in row.iter().enumerate() {
                match *tile {
                    Air => continue,
                    Solid(health) => {
                        // Tile is dead, don't need to render
                        if health == 0 {
                            continue;
                        }

                        let x_pos = self.world_offset.0 + TILE_SIZE * i as f32;
                        // TODO: the j + 1 is a kludge and only exists because rectangles are drawn downwards. This problem
                        // should go away once we change screen coordinates to be sensible (aka: y goes upwards)
                        let y_pos =
                            WORLD_HEIGHT - (self.world_offset.1 + TILE_SIZE * (j + 1) as f32);
                        let rect = Rect {
                            x: x_pos,
                            y: y_pos,
                            w: TILE_SIZE,
                            h: TILE_SIZE,
                        };
                        let color = color_lerp(
                            RED,
                            WHITE,
                            (health - 1) as f32 / (TILE_MAX_HEALTH - 1) as f32,
                        );
                        graphics::set_color(ctx, color)?;
                        graphics::rectangle(ctx, graphics::DrawMode::Fill, rect)?;
                    }
                }
            }
        }
        Ok(())
    }

    fn damage_tile(&mut self, x: GridCoord, y: GridCoord) {
        use self::Tile::*;
        match self.grid[y][x] {
            Solid(ref mut health) => {
                if *health == 0 {
                    return;
                }

                *health -= 1;
                if *health == 0 {
                    self.tiles_alive -= 1;
                }
            }
            Air => return,
        }
    }

    fn percent_tiles_alive(&self) -> f32 {
        self.tiles_alive as f32 / self.total_tiles as f32
    }
}

#[derive(PartialEq)]
enum GridState {
    Alive,
    Falling(WorldCoord), // Stores the target height to get to
    Dead,
}

fn total_tiles(module: [[Tile; GRID_WIDTH]; GRID_HEIGHT]) -> usize {
    let mut total_tiles = 0;
    for row in module.iter() {
        for tile in row.iter() {
            match tile {
                Tile::Air => continue,
                _ => total_tiles += 1,
            }
        }
    }
    total_tiles
}

fn parse_modules_file(path: &path::Path) -> Result<Vec<[[Tile; GRID_WIDTH]; GRID_HEIGHT]>, String> {
    let contents = &fs::read_to_string(path).unwrap();
    let mut modules_list = vec![];
    let lines: Vec<&str> = contents.lines().collect();

    for module in lines.chunks(9) {
        // last line in file, stop parsing
        if module.len() == 1 {
            assert_eq!(module[0].trim(), "");
            break;
        }
        assert_eq!(module.len(), 9);
        assert_eq!(module[8].trim(), "-");
        let mut grid = [[Tile::Air; GRID_WIDTH]; GRID_HEIGHT];
        for (i, row) in module[..8].iter().enumerate() {
            println!("{:?}", row);
            grid[7 - i] = text_to_row(row)
                .map_err(|err| format!("Could not parse {} (line: {}) Reason: {}", row, i, err))?;
        }
        modules_list.push(grid);
    }
    Ok(modules_list)
}

fn text_to_row(row: &str) -> Result<[Tile; GRID_WIDTH], String> {
    let mut tiles = [Tile::Air; GRID_WIDTH];
    for (i, character) in row.trim_right().chars().enumerate() {
        match character {
            '#' => tiles[i] = Tile::Solid(TILE_MAX_HEALTH),
            ' ' => tiles[i] = Tile::Air,
            _ => {
                return Err(format!(
                    "Unknown Character: {} at position {}",
                    character, i
                ));
            }
        }
    }
    Ok(tiles)
}

#[derive(Copy, Clone)]
enum Tile {
    Air,
    Solid(usize),
}

pub const WHITE: Color = Color {
    r: 1.0,
    g: 1.0,
    b: 1.0,
    a: 1.0,
};

pub const RED: Color = Color {
    r: 1.0,
    g: 0.0,
    b: 0.0,
    a: 1.0,
};

pub const GREEN: Color = Color {
    r: 0.0,
    g: 1.0,
    b: 0.0,
    a: 1.0,
};

pub const TRANSPARENT: Color = Color {
    r: 0.0,
    g: 0.0,
    b: 0.0,
    a: 0.0,
};

// todo : make this not stupid
pub fn color_lerp(a: Color, b: Color, t: f32) -> Color {
    fn f32_lerp(a: f32, b: f32, t: f32) -> f32 {
        a + (b - a) * t
    }

    Color::new(
        f32_lerp(a.r, b.r, t),
        f32_lerp(a.g, b.g, t),
        f32_lerp(a.b, b.b, t),
        f32_lerp(a.a, b.a, t),
    )
}

fn main() {
    let c = ggez::conf::Conf::new();
    let ctx = &mut Context::load_from_conf("fall", "acgames", c).unwrap();

    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let mut path = path::PathBuf::from(manifest_dir);
    path.push("resources");
    ctx.filesystem.mount(&path, true);
    graphics::set_screen_coordinates(
        ctx,
        Rect {
            x: 0.0,
            y: 0.0,
            w: WORLD_WIDTH,
            h: WORLD_HEIGHT,
        },
    )
    .expect("Couldn't set screen coordinates?!");
    let state = &mut MainState::new(ctx).unwrap();
    if let Err(e) = ggez::event::run(ctx, state) {
        println!("Error encountered: {}", e);
    }
}
