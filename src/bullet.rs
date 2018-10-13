use ggez::{
    graphics::{self, Point2, Rect, Vector2},
    Context, GameResult,
};

use crate::player::{Player, Team};

const BULLET_WIDTH: f32 = 0.2;
const BULLET_HEIGHT: f32 = 0.2;

#[derive(Debug)]
pub struct Bullet {
    pub pos: Point2,
    pub vel: Vector2,
    pub team: Team,
    pub is_alive: bool,
}

impl Bullet {
    pub fn new(pos: Point2, vel: Vector2, team: Team) -> Self {
        Bullet {
            pos,
            vel,
            team,
            is_alive: true,
        }
    }

    pub fn rect(&self) -> Rect {
        Rect {
            x: self.pos.x - BULLET_WIDTH / 2.0,
            y: self.pos.y + BULLET_HEIGHT / 2.0,
            w: BULLET_WIDTH,
            h: BULLET_HEIGHT,
        }
    }

    pub fn fixed_update(&mut self, players: &mut Vec<Player>) {
        self.pos += crate::DT * self.vel;

        if self.pos.x < -1.0 || self.pos.x > 33.0 {
            self.is_alive = false;
        }

        for player in players {
            if self.rect().overlaps(&player.rect()) && self.team != player.team {
                player.damage();
                self.is_alive = false;
            }
        }
    }

    pub fn draw(&self, ctx: &mut Context) -> GameResult<()> {
        let rect = self.rect();

        graphics::rectangle(
            ctx,
            graphics::DrawMode::Fill,
            Rect {
                y: 24.0 - rect.y,
                ..rect
            },
        )?;
        Ok(())
    }
}
