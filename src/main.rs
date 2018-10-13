extern crate rand;

use std::path;

use ggez::graphics::{Color, DrawMode, Point2, Rect, Vector2};
use ggez::{graphics, timer, Context, GameResult};

use rand::{thread_rng, Rng};

struct MainState {
    // Grids are stored from lowest visually to highest
    grids: Vec<Grid>,
}

impl MainState {
    fn new(_ctx: &mut Context) -> GameResult<MainState> {
        Ok(MainState {
            grids: vec![
                Grid::new((GRID_HEIGHT * 0) as f32),
                Grid::new((GRID_HEIGHT * 1) as f32),
                Grid::new((GRID_HEIGHT * 2) as f32),
            ],
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
        }

        if self.grids[0].state == GridState::Dead {
            self.grids.remove(0);
            self.grids.push(Grid::new(GRID_HEIGHT as f32 * 3.0));
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
        Grid {
            grid: [[Tile { health: 5 }; GRID_WIDTH]; GRID_HEIGHT],
            world_offset: (0.0, height),
            state: GridState::Alive,
            total_tiles: GRID_WIDTH * GRID_HEIGHT,
            tiles_alive: GRID_WIDTH * GRID_HEIGHT,
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
        for (j, row) in self.grid.iter().enumerate() {
            for (i, tile) in row.iter().enumerate() {
                // Tile is dead, don't need to render
                if tile.health == 0 {
                    continue;
                }

                let x_pos = self.world_offset.0 + TILE_SIZE * i as f32;
                // TODO: the j + 1 is a kludge and only exists because rectangles are drawn downwards. This problem
                // should go away once we change screen coordinates to be sensible (aka: y goes upwards)
                let y_pos = WORLD_HEIGHT - (self.world_offset.1 + TILE_SIZE * (j + 1) as f32);
                let rect = Rect {
                    x: x_pos,
                    y: y_pos,
                    w: TILE_SIZE,
                    h: TILE_SIZE,
                };
                let color = color_lerp(
                    RED,
                    WHITE,
                    (tile.health - 1) as f32 / (TILE_MAX_HEALTH - 1) as f32,
                );
                graphics::set_color(ctx, color)?;
                graphics::rectangle(ctx, graphics::DrawMode::Fill, rect)?;
            }
        }
        Ok(())
    }

    fn damage_tile(&mut self, x: GridCoord, y: GridCoord) {
        if self.grid[y][x].health == 0 {
            return;
        }

        self.grid[y][x].health -= 1;
        if self.grid[y][x].health == 0 {
            self.tiles_alive -= 1;
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

#[derive(Copy, Clone)]
struct Tile {
    health: usize,
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
