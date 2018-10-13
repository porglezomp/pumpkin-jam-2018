use ggez::{graphics::Image, Context, GameResult};

const JOIN_PATH: &str = "/join.png";
const LEAVES_PATH: &str = "/leaves.png";

pub struct Images {
    pub leaves: Image,
    pub join: Image,
}

impl Images {
    pub fn new(ctx: &mut Context) -> GameResult<Self> {
        Ok(Images {
            leaves: Image::new(ctx, LEAVES_PATH)?,
            join: Image::new(ctx, JOIN_PATH)?,
        })
    }
}
