use ggez::{
    graphics::{self, Color, Image},
    Context, GameResult,
};

const JOIN_PATH: &str = "/join.png";
// const LEAVES_PATH: &str = "/leaves.png";

pub struct Images {
    pub leaves: Image,
    pub join: Image,
    pub player: Image,
    pub start_flag: Image,
    pub leave_flag: Image,
}

impl Images {
    pub fn new(ctx: &mut Context) -> GameResult<Self> {
        graphics::set_default_filter(ctx, graphics::FilterMode::Nearest);
        Ok(Images {
            leaves: Image::solid(ctx, 16, Color::new(1.0, 1.0, 1.0, 1.0))?,
            join: Image::new(ctx, JOIN_PATH)?,
            player: Image::solid(ctx, 16, Color::new(1.0, 1.0, 1.0, 1.0))?,
            start_flag: Image::solid(ctx, 16, Color::new(0.0, 1.0, 0.0, 1.0))?,
            leave_flag: Image::solid(ctx, 16, Color::new(1.0, 0.0, 1.0, 1.0))?,
        })
    }
}
