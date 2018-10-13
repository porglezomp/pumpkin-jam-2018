use ggez::{
    graphics::{self, Drawable, Point2, Vector2},
    Context, GameResult,
};

pub struct Player {
    mesh: graphics::Mesh,
    pos: Point2,
    vel: Vector2,
    acc: Vector2,
}

impl Player {
    pub fn new(ctx: &mut Context) -> GameResult<Self> {
        let mesh = graphics::Mesh::new_polygon(
            ctx,
            graphics::DrawMode::Fill,
            &[
                Point2::new(-0.5, -0.5),
                Point2::new(-0.5, 0.5),
                Point2::new(0.5, 0.5),
                Point2::new(0.5, -0.5),
            ],
        )?;
        Ok(Player {
            mesh,
            pos: Point2::new(0.0, 0.0),
            vel: Vector2::new(0.0, 0.0),
            acc: Vector2::new(0.0, 0.0),
        })
    }

    pub fn update(&mut self) {
        self.vel += crate::DT * self.acc;
        self.pos += crate::DT * self.vel;
        self.acc = Vector2::new(0.0, 0.0);
    }

    pub fn draw(&self, ctx: &mut Context) -> GameResult<()> {
        self.mesh.draw(ctx, crate::draw_pos(self.pos), 0.0)?;
        Ok(())
    }
}
