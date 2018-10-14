use ggez::{
    event,
    graphics::{Color, DrawParam, Point2, Rect, Vector2},
    Context, GameResult,
};

use crate::bullet::Bullet;

use crate::draw;
use crate::grid;
use crate::images::Images;

pub const PLAYER_MAX_HEALTH: u8 = 3;
pub const TEAM_COLORS: [Color; 4] = [
    Color {
        r: 0.25,
        g: 0.7,
        b: 1.0,
        a: 1.0,
    },
    Color {
        r: 0.8,
        g: 0.2,
        b: 0.2,
        a: 1.0,
    },
    Color {
        r: 0.3,
        g: 1.0,
        b: 0.5,
        a: 1.0,
    },
    Color {
        r: 1.0,
        g: 0.9,
        b: 0.25,
        a: 1.0,
    },
];

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
    pub controls: Controls,
    pub control_state: ControlState,
    pos: Point2,
    vel: Vector2,
    acc: Vector2,
    pub health: u8,
    pub cooldown: f32,
    pub alive: bool,
}

impl Player {
    pub fn new(team: Team, controls: Controls) -> Self {
        Player {
            team,
            controls,
            control_state: ControlState::default(),
            pos: Point2::new(0.0, 0.0),
            vel: Vector2::new(0.0, 0.0),
            acc: Vector2::new(0.0, 0.0),
            health: PLAYER_MAX_HEALTH,
            cooldown: 0.0,
            alive: false,
        }
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
    /// Respawn the player in a grid, picked at random. Returns true if it succesfuly
    /// respawns the player, and false if it cannot find a place to spawn
    #[must_use]
    pub fn respawn(&mut self, grid: &grid::Grid) -> bool {
        let grid_coords = grid::find_spawn_location(grid.module);
        if grid_coords == None {
            return false;
        }
        // make the player spawns on top of the center of the block
        let offset = Vector2::new(grid::TILE_SIZE as f32 / 2.0, grid::TILE_SIZE as f32);
        self.pos = grid.to_world_coords(grid_coords.unwrap()) + offset;
        self.alive = true;
        self.health = PLAYER_MAX_HEALTH;
        true
    }

    pub fn update(&mut self, bullets: &mut Vec<Bullet>) {
        if !self.alive {
            return;
        }

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

        if self.health == 0 {
            self.alive = false;
        }
    }

    pub fn fixed_update(&mut self) {
        if !self.alive {
            return;
        }

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

    pub fn draw(&self, ctx: &mut Context, images: &Images) -> GameResult<()> {
        if !self.alive {
            return Ok(());
        }
        draw::draw_sprite(
            ctx,
            &images.player,
            DrawParam {
                dest: self.pos,
                color: Some(Color::new(1.0, 1.0, 1.0, 1.0)),
                ..Default::default()
            },
        )?;
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
