use ggez::{
    graphics::{self, Color, Image},
    Context, GameResult,
};

pub struct Images {
    pub tiles: Image,
    pub join: Image,
    pub ready: Image,
    pub player: Image,
    pub heart: Image,
}

impl Images {
    pub fn new(ctx: &mut Context) -> GameResult<Self> {
        graphics::set_default_filter(ctx, graphics::FilterMode::Nearest);
        Ok(Images {
            tiles: Image::new(ctx, "/tiles.png")?,
            join: Image::new(ctx, "/join.png")?,
            ready: Image::new(ctx, "/ready.png")?,
            player: Image::solid(ctx, 16, Color::new(1.0, 1.0, 1.0, 1.0))?,
            heart: Image::new(ctx, "/heart.png")?,
        })
    }
}
