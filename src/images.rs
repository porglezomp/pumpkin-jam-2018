use ggez::{
    graphics::{self, Color, Image},
    Context, GameResult,
};

const JOIN_PATH: &str = "/join.png";
// const LEAVES_PATH: &str = "/leaves.png";

pub struct Images {
    pub leaves: Image,
    pub join: Image,
}

impl Images {
    pub fn new(ctx: &mut Context) -> GameResult<Self> {
        graphics::set_default_filter(ctx, graphics::FilterMode::Nearest);
        Ok(Images {
            leaves: Image::solid(ctx, 16, Color::new(1.0, 1.0, 1.0, 1.0))?,
            join: Image::new(ctx, JOIN_PATH)?,
        })
    }
}
