#[macro_use]
extern crate serde_derive;
extern crate serde;

use std::path;

use ggez::{
    conf::{WindowMode, WindowSetup},
    event,
    graphics::{self, Color, DrawParam, Point2, Vector2},
    timer, Context, ContextBuilder, GameResult,
};
use rand::{thread_rng, Rng};

use crate::bullet::Bullet;
use crate::config::{GRID, MENU, PLAYER, TEAM};
use crate::grid::{Grid, GridState, Module};
use crate::images::Images;
use crate::player::{Axis, Button, Controls, Player, Team};
use crate::sound::Sound;

mod bullet;
mod collide;
mod config;
mod draw;
mod grid;
mod images;
mod math;
mod player;
mod sound;

fn joycon_controls(id: i32) -> Controls {
    Controls {
        lr: Axis::Analog(id, event::Axis::LeftX),
        jump: Button::Controller(id, event::Button::A),
        shoot: Button::Controller(id, event::Button::B),
    }
}

const WASD_CONTROLS: Controls = Controls {
    lr: Axis::Buttons(
        Button::Keyboard(event::Keycode::A),
        Button::Keyboard(event::Keycode::D),
    ),
    jump: Button::Keyboard(event::Keycode::W),
    shoot: Button::Keyboard(event::Keycode::Tab),
};

const ARROW_CONTROLS: Controls = Controls {
    lr: Axis::Buttons(
        Button::Keyboard(event::Keycode::Left),
        Button::Keyboard(event::Keycode::Right),
    ),
    jump: Button::Keyboard(event::Keycode::Up),
    shoot: Button::Keyboard(event::Keycode::Comma),
};

const DT: f32 = 1.0 / 60.0;
const MODULES_PATH: &str = "/modules.txt";

struct MainState {
    focused: bool,
    in_menu: bool,
    // Grids are stored from lowest visually to highest
    grids: Vec<Grid>,
    modules: Vec<Module>,
    players: [Option<Player>; 4],
    bullets: Vec<Bullet>,
    images: Images,
    sounds: Sound,
}

fn somes_mut<'a, T: 'a>(
    i: impl IntoIterator<Item = &'a mut Option<T>>,
) -> impl Iterator<Item = &'a mut T> {
    i.into_iter().filter_map(|x| x.as_mut())
}

impl MainState {
    fn new(ctx: &mut Context) -> GameResult<MainState> {
        let modules =
            grid::parse_modules_file(ctx, MODULES_PATH).expect("Should load the modules file");
        let grids = vec![
            Grid::new_from_module((grid::GRID_HEIGHT * 0) as f32, modules[0].clone()),
            Grid::new_from_module((grid::GRID_HEIGHT * 1) as f32, modules[0].clone()),
            Grid::new_from_module((grid::GRID_HEIGHT * 2) as f32, modules[0].clone()),
        ];

        let images = images::Images::new(ctx)?;
        let sounds = sound::Sound::new(ctx)?;
        config::load(ctx)?;

        Ok(MainState {
            focused: true,
            in_menu: true,
            grids,
            modules,
            players: [None, None, None, None],
            bullets: Vec::with_capacity(20),
            images,
            sounds,
        })
    }

    fn button(&mut self, btn: Button, pressed: bool) {
        let mut found = false;
        for player in somes_mut(&mut self.players) {
            if btn == player.controls.jump {
                player.control_state.jump = pressed;
                found = true;
            }
            if btn == player.controls.shoot {
                player.control_state.shoot = pressed;
                found = true;
            }
            if let Axis::Buttons(ref l, ref r) = player.controls.lr {
                if btn == *l {
                    if pressed {
                        player.control_state.facing = -1.0;
                    } else if player.control_state.r_pressed {
                        player.control_state.facing = 1.0;
                    }
                    player.control_state.l_pressed = pressed;
                    found = true;
                }
                if btn == *r {
                    if pressed {
                        player.control_state.facing = 1.0;
                    } else if player.control_state.l_pressed {
                        player.control_state.facing = -1.0;
                    }
                    player.control_state.r_pressed = pressed;
                    found = true;
                }
            }
        }

        if !found && pressed {
            if let Some((i, player)) = self
                .players
                .iter_mut()
                .enumerate()
                .find(|(_, x)| x.is_none())
            {
                match btn {
                    Button::Keyboard(event::Keycode::Up) => {
                        *player = Some(Player::new(Team(i as u8), ARROW_CONTROLS));
                    }
                    Button::Keyboard(event::Keycode::W) => {
                        *player = Some(Player::new(Team(i as u8), WASD_CONTROLS));
                    }
                    Button::Controller(id, event::Button::A) => {
                        *player = Some(Player::new(Team(i as u8), joycon_controls(id)));
                    }
                    _ => (),
                }
            }
        }
    }

    fn axis(&mut self, axis: event::Axis, id: i32, value: f32) {
        let axis = Axis::Analog(id, axis);
        for player in somes_mut(&mut self.players) {
            if axis == player.controls.lr {
                if value.abs() > 0.1 {
                    player.control_state.facing = value.signum();
                }
                player.control_state.lr = value;
            }
        }
    }
}

pub fn draw_pos(p: Point2) -> Point2 {
    Point2::new(p.x, 24.0 - p.y)
}

impl ggez::event::EventHandler for MainState {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        const DESIRED_FPS: u32 = 60;

        if !self.focused {
            while timer::check_update_time(ctx, DESIRED_FPS) {}
            timer::sleep(std::time::Duration::from_millis(10));
            timer::yield_now();
            return Ok(());
        }

        if self.in_menu {
            let mut ready = true;
            let mut player_count = 0;
            for player in somes_mut(&mut self.players) {
                ready &= player.ready;
                player_count += 1;
            }
            if ready && player_count >= 2 {
                self.in_menu = false;
            }
        }

        for i in 0..self.grids.len() {
            if i == 0 {
                let target = if self.grids[0].state == GridState::Dead {
                    -(grid::GRID_HEIGHT as f32)
                } else {
                    0.0
                };
                self.grids[i].fixed_update(target);
            } else {
                let offset = self.grids[i - 1].world_offset.y + grid::GRID_HEIGHT as f32;
                self.grids[i].fixed_update(offset);
            }
        }

        for player in somes_mut(&mut self.players) {
            player.update(ctx, &mut self.bullets, &mut self.sounds);
        }

        while timer::check_update_time(ctx, DESIRED_FPS) {
            // fixed update
            for player in somes_mut(&mut self.players) {
                player.fixed_update(&self.grids);

                // If the player is dead attempt to respawn them
                if !player.alive {
                    let mut indicies: Vec<_> = (0..self.grids.len()).collect();
                    rand::thread_rng().shuffle(&mut indicies);
                    for i in indicies {
                        // Avoid spawning on the lowest grid if too damanged
                        // (might be instant death!)
                        if i == 0 && self.grids[0].percent_tiles_alive() < GRID.no_spawn_threshold {
                            continue;
                        }
                        // Don't spawn above the screen.
                        if self.grids[i].world_offset.y > draw::WORLD_HEIGHT {
                            continue;
                        }
                        if player.respawn(&self.grids[i]) {
                            break;
                        }
                    }

                    if !player.alive {
                        println!("Player {:?} cant find a spot", player.team);
                    }
                }
            }

            for bullet in &mut self.bullets {
                bullet.fixed_update(&mut self.grids, &mut self.players, self.in_menu);
            }
            self.bullets.retain(|bullet| bullet.is_alive);

            if thread_rng().gen_bool(0.2) {
                let grid_id = *thread_rng()
                    .choose(&[
                        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 2,
                    ])
                    .unwrap();
                self.grids[grid_id].damage_tile(
                    thread_rng().gen_range(0, grid::GRID_WIDTH),
                    thread_rng().gen_range(0, grid::GRID_HEIGHT),
                );
            }
        }

        if self.grids.len() > 0 && self.grids[0].world_offset.y <= -(grid::GRID_HEIGHT as f32) {
            self.grids.remove(0);
            self.grids.push(Grid::new_from_module(
                grid::GRID_HEIGHT as f32 * 3.0,
                rand::thread_rng().choose(&self.modules).unwrap().clone(),
            ));
        }

        for i in 0..self.grids.len() {
            self.grids[i].update();
        }
        self.sounds.update();
        timer::yield_now();
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        if !self.focused {
            return Ok(());
        }

        let time = timer::duration_to_f64(timer::get_time_since_start(ctx));
        graphics::set_background_color(ctx, Color::new(0.0, 0.0, 0.0, 1.0));
        graphics::clear(ctx);

        for grid in &mut self.grids {
            grid.draw(ctx, &self.images)?;
        }

        for player in somes_mut(&mut self.players) {
            player.draw(ctx, &self.images)?;
        }

        for bullet in &self.bullets {
            bullet.draw(ctx, &self.images)?;
        }

        let mut hearts = draw::Batch::atlas(self.images.heart.clone(), 2, 1);
        let mut lives = draw::Batch::atlas(self.images.lives.clone(), 2, 1);
        let mut ready = draw::Batch::atlas(self.images.ready.clone(), 1, 1);
        let a = if time % 1.5 < 0.8 { 1.0 } else { 0.25 };
        for ((player, info), &color) in self.players.iter().zip(&MENU.pos).zip(&TEAM.colors) {
            let join_pos = Point2::new(info.join_pos.0, info.join_pos.1);
            let heart_pos = Point2::new(info.heart_pos.0, info.heart_pos.1);
            let ready_pos = Point2::new(info.ready_pos.0, info.ready_pos.1);
            let life_pos = Point2::new(info.life_pos.0, info.life_pos.1);
            let life_offset = Vector2::new(MENU.life_offset.0, MENU.life_offset.1);
            let heart_offset = Vector2::new(MENU.heart_offset.0, MENU.heart_offset.1);
            if let Some(player) = player {
                for heart in 0..PLAYER.max_health {
                    let sprite = if player.health() > heart { 0 } else { 1 };
                    let offset = heart as f32 * heart_offset;
                    hearts.add(
                        sprite,
                        DrawParam {
                            dest: heart_pos + offset,
                            color: Some(color.into()),
                            ..Default::default()
                        },
                    );
                }

                if self.in_menu {
                    if player.ready {
                        ready.add(
                            0,
                            DrawParam {
                                dest: ready_pos,
                                color: Some(color.into()),
                                ..Default::default()
                            },
                        );
                    }
                } else {
                    for life in 0..PLAYER.max_lives {
                        let sprite = if player.lives > life { 0 } else { 1 };
                        let offset = life as f32 * life_offset;
                        lives.add(
                            sprite,
                            DrawParam {
                                dest: life_pos + offset,
                                color: Some(color.into()),
                                ..Default::default()
                            },
                        )
                    }
                }
            } else {
                if self.in_menu {
                    draw::draw_sprite(
                        ctx,
                        &self.images.join,
                        DrawParam {
                            dest: join_pos,
                            color: Some(Color { a, ..color.into() }),
                            ..Default::default()
                        },
                    )?;
                }
            }
        }
        ready.draw(ctx, Default::default())?;
        hearts.draw(ctx, Default::default())?;
        lives.draw(ctx, Default::default())?;

        graphics::present(ctx);
        Ok(())
    }

    fn key_down_event(
        &mut self,
        _ctx: &mut Context,
        keycode: event::Keycode,
        _keymod: event::Mod,
        _repeat: bool,
    ) {
        self.button(player::Button::Keyboard(keycode), true);
    }

    fn key_up_event(
        &mut self,
        _ctx: &mut Context,
        keycode: event::Keycode,
        _keymod: event::Mod,
        _repeat: bool,
    ) {
        self.button(player::Button::Keyboard(keycode), false);
    }

    fn controller_button_down_event(
        &mut self,
        _ctx: &mut Context,
        btn: event::Button,
        instance_id: i32,
    ) {
        self.button(player::Button::Controller(instance_id, btn), true);
    }

    fn controller_button_up_event(
        &mut self,
        _ctx: &mut Context,
        btn: event::Button,
        instance_id: i32,
    ) {
        self.button(player::Button::Controller(instance_id, btn), false);
    }

    fn controller_axis_event(
        &mut self,
        _ctx: &mut Context,
        axis: event::Axis,
        value: i16,
        instance_id: i32,
    ) {
        self.axis(axis, instance_id, value as f32 / std::i16::MAX as f32)
    }

    fn focus_event(&mut self, ctx: &mut Context, gained: bool) {
        self.focused = gained;
        if gained {
            if let Err(err) = config::load(ctx) {
                println!("Config error: {}", err);
            }
            match Images::new(ctx) {
                Ok(images) => self.images = images,
                Err(err) => println!("Error reloading images: {}", err),
            }
        }
    }
}

fn main() {
    let ctx = &mut ContextBuilder::new("fall", "acgames")
        .add_resource_path(path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("resources"))
        .window_setup(WindowSetup {
            title: "Fall".into(),
            ..Default::default()
        })
        .window_mode(WindowMode {
            width: draw::SCREEN_WIDTH as u32,
            height: draw::SCREEN_HEIGHT as u32,
            ..Default::default()
        })
        .build()
        .unwrap();

    ctx.sdl_context
        .game_controller()
        .unwrap()
        .load_mappings("./resources/gamecontrollerdb.txt")
        .unwrap();
    ctx.gamepad_context = ggez::input::GamepadContext::new(&ctx.sdl_context).unwrap();

    let state = &mut MainState::new(ctx).unwrap();
    if let Err(e) = ggez::event::run(ctx, state) {
        println!("Error encountered: {}", e);
    }
}
