use ggez::{
    graphics::{self, Color, Image},
    Context, GameResult,
};

pub struct Images {
    pub tiles: Image,
    pub join: Image,
    pub player: Image,
    pub start_flag: Image,
    pub leave_flag: Image,
    pub heart: Image,
}

impl Images {
    pub fn new(ctx: &mut Context) -> GameResult<Self> {
        graphics::set_default_filter(ctx, graphics::FilterMode::Nearest);
        Ok(Images {
            tiles: Image::new(ctx, "/tiles.png")?,
            join: Image::new(ctx, "/join.png")?,
            player: Image::solid(ctx, 16, Color::new(1.0, 1.0, 1.0, 1.0))?,
            start_flag: Image::solid(ctx, 16, Color::new(0.0, 1.0, 0.0, 1.0))?,
            leave_flag: Image::solid(ctx, 16, Color::new(1.0, 0.0, 1.0, 1.0))?,
            heart: Image::new(ctx, "/heart.png")?,
        })
    }
}
