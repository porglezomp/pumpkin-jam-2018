use ggez::{
    graphics,
    graphics::{DrawParam, Drawable, Point2},
    Context, GameResult,
};

use crate::grid;

pub type WorldCoord = f32;
pub const SCREEN_WIDTH: f32 = 960.0;
pub const SCREEN_HEIGHT: f32 = 720.0;
pub const WORLD_WIDTH: WorldCoord = grid::GRID_WIDTH as f32;
pub const WORLD_HEIGHT: WorldCoord = (grid::GRID_HEIGHT * 3) as f32;

pub fn draw_ex(ctx: &mut Context, drawable: &Drawable, param: DrawParam) -> GameResult<()> {
    let new_x = SCREEN_WIDTH / WORLD_WIDTH * param.dest.x;
    let new_y = SCREEN_HEIGHT / WORLD_HEIGHT * (WORLD_HEIGHT - param.dest.y);
    let param = DrawParam {
        dest: Point2::new(new_x, new_y),
        offset: Point2::new(0.0, 0.0),
        scale: Point2::new(SCREEN_WIDTH / WORLD_WIDTH, -SCREEN_HEIGHT / WORLD_HEIGHT),
        ..param
    };

    graphics::draw_ex(ctx, drawable, param)
}

pub fn draw(ctx: &mut Context, drawable: &Drawable, dest: Point2, rotation: f32) -> GameResult<()> {
    let param = DrawParam {
        dest: dest,
        rotation: rotation,
        ..Default::default()
    };
    draw_ex(ctx, drawable, param)
}
