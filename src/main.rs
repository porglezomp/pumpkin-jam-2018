use std::path;

use ggez::{
    conf::{WindowMode, WindowSetup},
    event,
    graphics::{self, Color, DrawParam, Point2, Vector2},
    timer, Context, ContextBuilder, GameResult,
};
use rand::{thread_rng, Rng};

use crate::bullet::Bullet;
use crate::grid::{Grid, GridState, Module};
use crate::images::Images;
use crate::player::{Axis, Button, Controls, Player, Team};

mod bullet;
mod collide;
mod draw;
mod grid;
mod images;
mod math;
mod player;

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
            Grid::new_from_module((grid::GRID_HEIGHT * 0) as f32, modules[2].clone()),
            Grid::new_from_module((grid::GRID_HEIGHT * 1) as f32, modules[2].clone()),
            Grid::new_from_module(
                (grid::GRID_HEIGHT * 2) as f32,
                rand::thread_rng().choose(&modules).unwrap().clone(),
            ),
        ];

        let images = images::Images::new(ctx)?;

        Ok(MainState {
            focused: true,
            in_menu: true,
            grids,
            modules,
            players: [None, None, None, None],
            bullets: Vec::with_capacity(20),
            images,
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

        for player in somes_mut(&mut self.players) {
            player.update(&mut self.bullets);
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
                        if i == 0 && self.grids[0].percent_tiles_alive() < grid::NO_SPAWN_THRESHOLD
                        {
                            continue;
                        }
                        let result = player.respawn(&self.grids[i]);
                        if result {
                            break;
                        }
                    }

                    if !player.alive {
                        println!("cant find a spot");
                    }
                }
            }

            for bullet in &mut self.bullets {
                bullet.fixed_update(somes_mut(&mut self.players));
            }

            self.bullets.retain(|bullet| bullet.is_alive);

            for i in 0..self.grids.len() {
                if i == 0 {
                    self.grids[0].update(None);

                    // If the bottom grid is dead, make it fall, and a new grid
                    if self.grids[0].state == GridState::Dead {
                        self.grids.push(Grid::new_from_module(
                            grid::GRID_HEIGHT as f32 * 3.0,
                            rand::thread_rng().choose(&self.modules).unwrap().clone(),
                        ));
                        self.grids[0].state =
                            GridState::DeadFalling(-1.0 * (grid::GRID_HEIGHT as f32));
                    }

                    // When the bottom gird is offscreen, remove it
                    if let GridState::DeadFalling(goal_height) = self.grids[0].state {
                        if (goal_height - self.grids[0].height()).abs() < 0.1 {
                            self.grids.remove(0);
                        }
                    }
                } else {
                    let (left, right) = self.grids.split_at_mut(i);
                    right.first_mut().unwrap().update(left.last());
                }
            }

            self.grids[0].damage_tile(
                thread_rng().gen_range(0, grid::GRID_WIDTH),
                thread_rng().gen_range(0, grid::GRID_HEIGHT),
            );
            self.grids[0].damage_tile(
                thread_rng().gen_range(0, grid::GRID_WIDTH),
                thread_rng().gen_range(0, grid::GRID_HEIGHT),
            );
            self.grids[0].damage_tile(
                thread_rng().gen_range(0, grid::GRID_WIDTH),
                thread_rng().gen_range(0, grid::GRID_HEIGHT),
            );
        }

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

        draw::draw_sprite(
            ctx,
            &self.images.start_flag,
            DrawParam {
                dest: Point2::new(8.0, 12.0),
                ..Default::default()
            },
        )?;

        draw::draw_sprite(
            ctx,
            &self.images.leave_flag,
            DrawParam {
                dest: Point2::new(24.0, 12.0),
                ..Default::default()
            },
        )?;

        for player in somes_mut(&mut self.players) {
            player.draw(ctx, &self.images)?;
        }

        for bullet in &self.bullets {
            bullet.draw(ctx, &self.images)?;
        }

        struct MenuInfo {
            join_pos: Point2,
            heart_pos: Point2,
        }

        let menu_info = [
            MenuInfo {
                join_pos: Point2::new(draw::WORLD_WIDTH - 9.0, 1.0),
                heart_pos: Point2::new(draw::WORLD_WIDTH - 3.0, 1.0),
            },
            MenuInfo {
                join_pos: Point2::new(1.0, 1.0),
                heart_pos: Point2::new(1.0, 1.0),
            },
            MenuInfo {
                join_pos: Point2::new(1.0, draw::WORLD_HEIGHT - 3.0),
                heart_pos: Point2::new(1.0, draw::WORLD_HEIGHT - 2.0),
            },
            MenuInfo {
                join_pos: Point2::new(draw::WORLD_WIDTH - 9.0, draw::WORLD_HEIGHT - 3.0),
                heart_pos: Point2::new(draw::WORLD_WIDTH - 3.0, draw::WORLD_HEIGHT - 2.0),
            },
        ];

        let mut hearts = draw::Batch::atlas(self.images.heart.clone(), 2, 1);
        let a = if time % 1.5 < 0.8 { 1.0 } else { 0.25 };
        for ((player, info), &color) in self
            .players
            .iter()
            .zip(&menu_info)
            .zip(&player::TEAM_COLORS)
        {
            if let Some(player) = player {
                for heart in 0..player::PLAYER_MAX_HEALTH {
                    let sprite = if player.health > heart { 0 } else { 1 };
                    let offset = heart as f32 * Vector2::new(0.7, 0.0);
                    hearts.add(
                        sprite,
                        DrawParam {
                            dest: info.heart_pos + offset,
                            color: Some(color),
                            ..Default::default()
                        },
                    );
                }
            } else {
                if self.in_menu {
                    draw::draw_sprite(
                        ctx,
                        &self.images.join,
                        DrawParam {
                            dest: info.join_pos,
                            color: Some(Color { a, ..color }),
                            ..Default::default()
                        },
                    )?;
                }
            }
        }
        hearts.draw(ctx, Default::default())?;

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

    fn focus_event(&mut self, _ctx: &mut Context, gained: bool) {
        self.focused = gained;
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
