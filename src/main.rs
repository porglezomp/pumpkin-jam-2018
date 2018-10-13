extern crate rand;

use std::path;

use ggez::graphics::{Color, DrawMode, Point2, Rect, Vector2};
use ggez::{graphics, timer, Context, GameResult};

use rand::{thread_rng, Rng};

struct MainState {
    grid: Grid,
}

impl MainState {
    fn new(_ctx: &mut Context) -> GameResult<MainState> {
        Ok(MainState {
            grid: Default::default(),
        })
    }
}

impl ggez::event::EventHandler for MainState {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        const DESIRED_FPS: u32 = 60;
        // variable update
        while timer::check_update_time(ctx, DESIRED_FPS) {
            // fixed update
        }
        self.grid.damage_tile(
            thread_rng().gen_range(0, GRID_WIDTH),
            thread_rng().gen_range(0, GRID_HEIGHT),
        );
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx);

        self.grid.draw(ctx)?;

        graphics::present(ctx);
        Ok(())
    }
}

type GridCoord = usize;
type WorldCoord = f32;

pub const GRID_WIDTH: GridCoord = 32;
pub const GRID_HEIGHT: GridCoord = 8;
pub const TILE_SIZE: WorldCoord = 1.0f32;
pub const TILE_MAX_HEALTH: usize = 5;

/// A grid contains the collidable tiles that our dynamic objects interact with
///
struct Grid {
    grid: [[Tile; GRID_WIDTH]; GRID_HEIGHT],
    world_offset: (WorldCoord, WorldCoord),
}

impl Grid {
    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        for (j, row) in self.grid.iter().enumerate() {
            for (i, tile) in row.iter().enumerate() {
                // Tile is dead, don't need to render
                if tile.health == 0 {
                    continue;
                }

                let x_pos = self.world_offset.0 + TILE_SIZE * i as f32;
                let y_pos = self.world_offset.1 + TILE_SIZE * j as f32;
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
        self.grid[y][x].health = self.grid[y][x].health.saturating_sub(1);
    }
}

impl Default for Grid {
    fn default() -> Grid {
        Grid {
            grid: [[Tile { health: 5 }; GRID_WIDTH]; GRID_HEIGHT],
            world_offset: (0.0, 12.0),
        }
    }
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
            w: GRID_WIDTH as f32,
            h: (GRID_HEIGHT * 3) as f32,
        },
    ).expect("Couldn't set screen coordinates?!");
    let state = &mut MainState::new(ctx).unwrap();
    if let Err(e) = ggez::event::run(ctx, state) {
        println!("Error encountered: {}", e);
    }
}
