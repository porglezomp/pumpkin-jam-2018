use ggez::{
    event,
    graphics::{self, Drawable, Point2, Vector2},
    Context, GameResult,
};

#[derive(PartialEq, Eq)]
pub enum Axis {
    Buttons(Button, Button),
    Analog(i32, event::Axis),
}

#[derive(PartialEq, Eq)]
pub enum Button {
    Keyboard(event::Keycode),
    Controller(i32, event::Button),
}

pub struct Controls {
    pub lr: Axis,
    pub jump: Button,
    pub shoot: Button,
}

#[derive(Copy, Clone, Debug, Default)]
pub struct ControlState {
    pub lr: f32,
    pub jump: bool,
    pub shoot: bool,
    pub l_pressed: bool,
    pub r_pressed: bool,
}

pub struct Player {
    mesh: graphics::Mesh,
    pub controls: Controls,
    pub control_state: ControlState,
    pos: Point2,
    vel: Vector2,
    acc: Vector2,
}

impl Player {
    pub fn new(ctx: &mut Context, controls: Controls) -> GameResult<Self> {
        let mesh = graphics::Mesh::new_polygon(
            ctx,
            graphics::DrawMode::Fill,
            &[
                Point2::new(-0.5, -2.0),
                Point2::new(-0.5, 0.0),
                Point2::new(0.5, 0.0),
                Point2::new(0.5, -2.0),
            ],
        )?;

        Ok(Player {
            mesh,
            controls,
            control_state: ControlState::default(),
            pos: Point2::new(0.0, 0.0),
            vel: Vector2::new(0.0, 0.0),
            acc: Vector2::new(0.0, 0.0),
        })
    }

    fn controls(&mut self) {
        if let Axis::Buttons(..) = self.controls.lr {
            self.control_state.lr = 0.0;
            if self.control_state.l_pressed {
                self.control_state.lr -= 1.0;
            }
            if self.control_state.r_pressed {
                self.control_state.lr += 1.0;
            }
        }
    }

    pub fn update(&mut self) {
        self.controls();

        let grounded = self.pos.y <= 0.0;

        if grounded && self.control_state.jump {
            self.acc.y = 13.0 / crate::DT;
        }

        self.acc.x += self.control_state.lr / crate::DT;
    }

    pub fn fixed_update(&mut self) {
        self.vel += crate::DT * self.acc;
        self.vel.x *= 0.95;
        self.vel.y *= 0.995;
        self.pos += crate::DT * self.vel;

        let grounded = self.pos.y <= 0.0;

        // Ground
        if grounded {
            self.pos.y = 0.0;
            self.vel.y = self.vel.y.max(0.0);
        }

        self.acc = Vector2::new(0.0, -20.0);
    }

    pub fn draw(&self, ctx: &mut Context) -> GameResult<()> {
        self.mesh.draw(ctx, crate::draw_pos(self.pos), 0.0)?;
        Ok(())
    }
}
