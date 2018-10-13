use ggez::graphics::{Color, DrawMode, Point2, Rect, Vector2};
use ggez::{graphics, timer, Context, GameResult};
use std::path;

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
        self.grid.world_offset.0 += 0.01;
        self.grid.world_offset.1 += 0.01;
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

const GRID_WIDTH: GridCoord = 32;
const GRID_HEIGHT: GridCoord = 8;
const TILE_SIZE: WorldCoord = 1.0f32;
/// A grid contains the collidable tiles that our dynamic objects interact with
///
struct Grid {
    grid: [[Tile; GRID_WIDTH]; GRID_HEIGHT],
    world_offset: (WorldCoord, WorldCoord),
}

impl Grid {
    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        for (j, row) in self.grid.iter().enumerate() {
            for (i, _tile) in row.iter().enumerate() {
                let x_pos = self.world_offset.0 + TILE_SIZE * i as f32;
                let y_pos = self.world_offset.1 + TILE_SIZE * j as f32;
                let rect = Rect {
                    x: x_pos,
                    y: y_pos,
                    w: TILE_SIZE,
                    h: TILE_SIZE,
                };
                let color = Color {
                    r: x_pos.tan(),
                    g: y_pos.tan(),
                    b: x_pos.tan(),
                    a: 1.0,
                };
                graphics::set_color(ctx, color);
                graphics::rectangle(ctx, graphics::DrawMode::Fill, rect)?;
            }
        }
        Ok(())
    }
}

impl Default for Grid {
    fn default() -> Grid {
        Grid {
            grid: [[Tile {}; GRID_WIDTH]; GRID_HEIGHT],
            world_offset: (0.0, 12.0),
        }
    }
}

#[derive(Copy, Clone)]
struct Tile {}

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
    );
    let state = &mut MainState::new(ctx).unwrap();
    if let Err(e) = ggez::event::run(ctx, state) {
        println!("Error encountered: {}", e);
    }
}
