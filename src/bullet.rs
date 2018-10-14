use ggez::{
    graphics::{self, DrawMode, Point2, Rect, Vector2},
    Context, GameResult,
};

use crate::draw;
use crate::grid::{Grid, Tile};
use crate::images::Images;
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

    pub fn fixed_update<'a>(
        &mut self,
        grids: &mut [Grid],
        players: impl Iterator<Item = &'a mut Player>,
    ) {
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

        for grid in grids {
            let mut tiles = Vec::new();
            grid.overlapping_tiles(self.rect(), &mut tiles);
            for (tile, x, y) in tiles {
                match tile {
                    Tile::Solid(_) => {
                        grid.damage_tile(x, y);
                        grid.damage_tile(x, y);
                        self.is_alive = false;
                    }
                    _ => (),
                }
            }
        }
    }

    pub fn draw(&self, ctx: &mut Context, _images: &Images) -> GameResult<()> {
        let points = [
            Point2::new(-BULLET_WIDTH / 2.0, -BULLET_HEIGHT / 2.0),
            Point2::new(-BULLET_WIDTH / 2.0, BULLET_HEIGHT / 2.0),
            Point2::new(BULLET_WIDTH / 2.0, BULLET_HEIGHT / 2.0),
            Point2::new(BULLET_WIDTH / 2.0, -BULLET_HEIGHT / 2.0),
        ];
        let mesh = graphics::Mesh::new_polygon(ctx, DrawMode::Fill, &points)?;
        draw::draw(ctx, &mesh, self.pos, 0.0)
    }
}
