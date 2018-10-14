use ggez::{
    graphics,
    graphics::{spritebatch::SpriteBatch, DrawParam, Drawable, Image, Point2},
    Context, GameResult,
};

use crate::grid;

pub type WorldCoord = f32;
pub const SCREEN_WIDTH: f32 = 768.0;
pub const SCREEN_HEIGHT: f32 = 576.0;
pub const WORLD_WIDTH: WorldCoord = grid::GRID_WIDTH as f32;
pub const WORLD_HEIGHT: WorldCoord = (grid::GRID_HEIGHT * 3) as f32;
const SCALE_X: f32 = SCREEN_WIDTH / WORLD_WIDTH;
const SCALE_Y: f32 = SCREEN_HEIGHT / WORLD_HEIGHT;
const PIX: f32 = 1.5;

pub struct Batch {
    batch: SpriteBatch,
}

impl Batch {
    pub fn new(image: Image) -> Self {
        Batch {
            batch: SpriteBatch::new(image),
        }
    }

    pub fn add(&mut self, param: DrawParam) {
        let param = DrawParam {
            offset: Point2::new(0.0, 0.5),
            scale: Point2::new(
                PIX / SCALE_X * param.scale.x,
                -PIX / SCALE_Y * param.scale.y,
            ),
            ..param
        };
        self.batch.add(param);
    }

    pub fn draw(&mut self, ctx: &mut Context, param: DrawParam) -> GameResult<()> {
        draw_ex(ctx, &self.batch, param)
    }
}

pub fn draw_sprite(ctx: &mut Context, image: &Image, param: DrawParam) -> GameResult<()> {
    let param = DrawParam {
        offset: Point2::new(0.0, 0.5),
        scale: Point2::new(PIX / SCALE_X, -PIX / SCALE_Y),
        ..param
    };
    draw_ex(ctx, image, param)
}

pub fn draw_ex(ctx: &mut Context, drawable: &Drawable, param: DrawParam) -> GameResult<()> {
    let new_x = SCALE_X * param.dest.x;
    let new_y = SCREEN_HEIGHT - SCALE_Y * param.dest.y;
    let param = DrawParam {
        dest: Point2::new(new_x, new_y),
        scale: Point2::new(SCALE_X * param.scale.x, -SCALE_Y * param.scale.y),
        ..param
    };

    graphics::draw_ex(ctx, drawable, param)
}

pub fn draw(ctx: &mut Context, drawable: &Drawable, dest: Point2, rotation: f32) -> GameResult<()> {
    let param = DrawParam {
        dest,
        rotation,
        ..Default::default()
    };
    draw_ex(ctx, drawable, param)
}
