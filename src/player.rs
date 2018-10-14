use ggez::{
    event,
    graphics::{DrawParam, Point2, Rect, Vector2},
    Context, GameResult,
};

use crate::bullet::Bullet;
use crate::sound::{Sound, SoundEffect};

use crate::collide;
use crate::config::{PLAYER, TEAM};
use crate::draw;
use crate::grid;
use crate::images::Images;
use crate::math;

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
    pub jump: bool,            // Updated every jump event (edge up and edge down)
    pub this_jump_frame: bool, // Updated every frame
    pub last_jump_frame: bool, // Updated every frame
    pub shoot: bool,
    pub l_pressed: bool,
    pub r_pressed: bool,
    pub facing: f32,
}

#[derive(Debug)]
pub struct Player {
    pub team: Team,
    pub controls: Controls,
    pub control_state: ControlState,
    pos: Point2,
    vel: Vector2,
    acc: Vector2,
    health: u8,
    pub lives: u8,
    pub cooldown: f32,
    pub alive: bool,
    pub grounded: bool,
    pub frames_since_grounded: u8,
    jump: JumpState,
    pub ready: bool,
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
            health: PLAYER.max_health,
            lives: PLAYER.max_lives,
            cooldown: 0.0,
            alive: false,
            frames_since_grounded: 0,
            grounded: true,
            ready: false,
            jump: JumpState::Double,
        }
    }

    pub fn health(&self) -> u8 {
        self.health
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
        self.vel = Vector2::new(0.0, 0.0);
        self.acc = Vector2::new(0.0, 0.0);
        self.alive = true;
        self.health = PLAYER.max_health;
        true
    }

    pub fn update(&mut self, ctx: &mut Context, bullets: &mut Vec<Bullet>, sounds: &mut Sound) {
        if !self.alive {
            return;
        }

        self.control_state.last_jump_frame = self.control_state.this_jump_frame;
        self.control_state.this_jump_frame = self.control_state.jump;
        self.controls();

        use self::JumpState::*;
        // Want to jump (rising jump edge)
        if !self.control_state.last_jump_frame && self.control_state.this_jump_frame {
            self.jump = match self.jump {
                Double => {
                    self.acc.y = PLAYER.jump_power / crate::DT;
                    self.grounded = false;
                    sounds.play_sound(ctx, SoundEffect::Jump);
                    Single
                }
                Single => {
                    self.acc.y = PLAYER.second_jump_power / crate::DT;
                    self.grounded = false;
                    sounds.play_sound(ctx, SoundEffect::SecondJump);
                    None
                }
                None => None,
            }
        }

        // Transition from air to grounded
        if self.grounded && self.frames_since_grounded > 3 {
            self.jump = JumpState::Double;
            sounds.play_sound(ctx, SoundEffect::Land);
        }

        // Transition from grounded to air
        if !self.grounded && self.frames_since_grounded > 3 {
            if self.jump == JumpState::Double {
                self.jump = JumpState::Single;
            }
        }

        if self.control_state.shoot && self.cooldown <= 0.0 {
            bullets.push(Bullet::new(
                self.pos + Vector2::new(PLAYER.width / 2.0, 0.0) + Vector2::new(
                    self.control_state.facing * PLAYER.shoot_offset_x,
                    PLAYER.shoot_offset_y,
                ),
                Vector2::new(self.control_state.facing * 30.0, 0.0),
                self.team,
            ));
            self.cooldown = 0.3;
            sounds.play_sound(ctx, SoundEffect::Shoot);
        }

        self.acc.x += self.control_state.lr / crate::DT;

        if self.pos.y < -1.0 {
            self.damage();
        }
    }

    pub fn fixed_update(&mut self, grids: &[grid::Grid]) {
        if !self.alive {
            return;
        }

        self.cooldown = 0.0f32.max(self.cooldown - crate::DT);
        self.vel += crate::DT * self.acc;
        self.vel.x *= 0.95;
        self.vel.y *= 0.995;

        // Collision resolution, this is done by move the player along the y-axis,
        // and moving them them back so that they do not collide with the wall,
        // and then repeating this process for the x-axis.
        let mut colliders = Vec::with_capacity(8);
        let mut next_pos = self.pos;

        // Resolve Vertically
        next_pos.y += self.vel.y * crate::DT;

        let next_rect = math::rect_from_point(next_pos, PLAYER.width, PLAYER.height);
        collide::get_overlapping_tiles(grids, next_rect, &mut colliders);
        let (res_disp_y, res_vel_y) =
            collide::resolve_colliders_vert(next_rect, self.vel, &colliders);
        next_pos.y += res_disp_y;
        self.vel = res_vel_y;

        self.frames_since_grounded = if self.grounded {
            0
        } else {
            self.frames_since_grounded.saturating_add(1)
        };
        // If the displacement was vertical that means we have been pushed up
        // out of the ground, which means we are probably grounded.
        self.grounded = res_disp_y > 0.0;

        // Resolve Horizontally
        next_pos.x += crate::DT * self.vel.x;

        let next_rect = math::rect_from_point(next_pos, PLAYER.width, PLAYER.height);
        colliders.clear();
        collide::get_overlapping_tiles(grids, next_rect, &mut colliders);
        let (res_disp_x, res_vel_x) =
            collide::resolve_colliders_horiz(next_rect, self.vel, &colliders);
        next_pos.x += res_disp_x;
        self.vel = res_vel_x;

        self.pos = next_pos;

        // Don't let the player escape!
        if self.pos.y + PLAYER.height > draw::WORLD_HEIGHT {
            self.pos.y = draw::WORLD_HEIGHT - PLAYER.height;
        }
        self.pos.x = math::clamp(0.0, draw::WORLD_WIDTH - PLAYER.width, self.pos.x);
        // Gravity
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
                color: Some(TEAM.colors[self.team.0 as usize].into()),
                ..Default::default()
            },
        )?;
        Ok(())
    }

    pub fn kill(&mut self) {
        assert_eq!(self.health, 0);
        if self.alive {
            self.lives = self.lives.saturating_sub(1);
        }
        self.alive = false;
    }

    pub fn damage(&mut self) {
        self.health = self.health.saturating_sub(1);
        if self.health == 0 {
            self.kill();
        }
    }

    pub fn rect(&self) -> Rect {
        Rect {
            x: self.pos.x - 0.5,
            y: self.pos.y,
            w: PLAYER.width,
            h: PLAYER.height,
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
enum JumpState {
    Double,
    Single,
    None,
}
