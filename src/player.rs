use ggez::{
    event,
    graphics::{self, Color, Drawable, Point2, Rect, Vector2},
    Context, GameResult,
};

use crate::bullet::Bullet;

use crate::draw;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct Team(pub u8);

#[derive(Debug, PartialEq, Eq)]
pub enum Axis {
    Buttons(Button, Button),
    Analog(i32, event::Axis),
}

#[derive(Debug, PartialEq, Eq)]
pub enum Button {
    Keyboard(event::Keycode),
    Controller(i32, event::Button),
}

#[derive(Debug)]
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

#[derive(Debug)]
pub struct Player {
    pub team: Team,
    mesh: graphics::Mesh,
    pub controls: Controls,
    pub control_state: ControlState,
    pos: Point2,
    vel: Vector2,
    acc: Vector2,
    pub health: u8,
    pub cooldown: f32,
}

impl Player {
    pub fn new(ctx: &mut Context, team: Team, controls: Controls) -> GameResult<Self> {
        let mesh = graphics::Mesh::new_polygon(
            ctx,
            graphics::DrawMode::Fill,
            &[
                Point2::new(-0.5, 2.0),
                Point2::new(-0.5, 0.0),
                Point2::new(0.5, 0.0),
                Point2::new(0.5, 2.0),
            ],
        )?;

        Ok(Player {
            team,
            mesh,
            controls,
            control_state: ControlState::default(),
            pos: Point2::new(0.0, 0.0),
            vel: Vector2::new(0.0, 0.0),
            acc: Vector2::new(0.0, 0.0),
            health: 3,
            cooldown: 0.0,
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

    pub fn update(&mut self, bullets: &mut Vec<Bullet>) {
        self.controls();

        let grounded = self.pos.y <= 0.0;

        if grounded && self.control_state.jump {
            self.acc.y = 13.0 / crate::DT;
        }

        if self.control_state.shoot && self.cooldown <= 0.0 {
            let bullet = if self.control_state.lr > 0.0 {
                Bullet::new(
                    self.pos + Vector2::new(0.6, 1.0),
                    Vector2::new(30.0, 0.0),
                    self.team,
                )
            } else {
                Bullet::new(
                    self.pos + Vector2::new(-0.6, 1.0),
                    Vector2::new(-30.0, 0.0),
                    self.team,
                )
            };
            bullets.push(bullet);
            self.cooldown = 0.3;
        }

        self.acc.x += self.control_state.lr / crate::DT;
    }

    pub fn fixed_update(&mut self) {
        self.cooldown = 0.0f32.max(self.cooldown - crate::DT);
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
        graphics::set_color(
            ctx,
            Color {
                r: 1.0,
                g: 1.0,
                b: 1.0,
                a: 1.0,
            },
        )?;
        draw::draw(ctx, &self.mesh, self.pos, 0.0)?;
        Ok(())
    }

    pub fn damage(&mut self) {
        self.health = self.health.saturating_sub(1);
        println!("{:?} has {} health", self.team, self.health);
    }

    pub fn rect(&self) -> Rect {
        Rect {
            x: self.pos.x - 0.5,
            y: self.pos.y,
            w: 1.0,
            h: 2.0,
        }
    }
}
