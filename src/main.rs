use std::path;

use ggez::{
    conf::{WindowMode, WindowSetup},
    event,
    graphics::{self, Color, DrawParam, Image, Point2, Rect},
    timer, Context, ContextBuilder, GameResult,
};
use rand::{thread_rng, Rng};

use crate::bullet::Bullet;
use crate::grid::{Grid, GridState, Module};
use crate::player::{Player, Team};

mod bullet;
mod draw;
mod grid;
mod player;

const MODULES_PATH: &str = "./resources/modules.txt";
const LEAVES_PATH: &str = "/leaves.png";
const DT: f32 = 1.0 / 60.0;

struct MainState {
    // Grids are stored from lowest visually to highest
    grids: Vec<Grid>,
    modules: Vec<Module>,
    players: Vec<Player>,
    bullets: Vec<Bullet>,
    leaves_image: Image,
}

impl MainState {
    fn new(ctx: &mut Context) -> GameResult<MainState> {
        use crate::player::{Axis, Button, Controls};
        let players = vec![
            Player::new(
                ctx,
                Team(0),
                Controls {
                    lr: Axis::Buttons(
                        Button::Keyboard(event::Keycode::Left),
                        Button::Keyboard(event::Keycode::Right),
                    ),
                    jump: Button::Keyboard(event::Keycode::Up),
                    shoot: Button::Keyboard(event::Keycode::Space),
                },
            )?,
            Player::new(
                ctx,
                Team(1),
                Controls {
                    lr: Axis::Buttons(
                        Button::Keyboard(event::Keycode::A),
                        Button::Keyboard(event::Keycode::D),
                    ),
                    jump: Button::Keyboard(event::Keycode::W),
                    shoot: Button::Keyboard(event::Keycode::Tab),
                },
            )?,
        ];

        let modules = grid::parse_modules_file(&path::Path::new(MODULES_PATH)).unwrap();
        let grids = vec![
            Grid::new_from_module((grid::GRID_HEIGHT * 0) as f32, modules[2].clone()),
            Grid::new_from_module((grid::GRID_HEIGHT * 1) as f32, modules[2].clone()),
            Grid::new_from_module(
                (grid::GRID_HEIGHT * 2) as f32,
                rand::thread_rng().choose(&modules).unwrap().clone(),
            ),
        ];

        let leaves_image = Image::new(ctx, path::Path::new(LEAVES_PATH))?;

        Ok(MainState {
            grids,
            modules,
            players,
            bullets: Vec::with_capacity(20),
            leaves_image,
        })
    }

    fn button(&mut self, btn: player::Button, pressed: bool) {
        for player in &mut self.players {
            if btn == player.controls.jump {
                player.control_state.jump = pressed;
            }
            if btn == player.controls.shoot {
                player.control_state.shoot = pressed;
            }
            if let player::Axis::Buttons(ref l, ref r) = player.controls.lr {
                if btn == *l {
                    player.control_state.l_pressed = pressed;
                }
                if btn == *r {
                    player.control_state.r_pressed = pressed;
                }
            }
        }
    }

    fn axis(&mut self, axis: event::Axis, id: i32, value: f32) {
        let axis = player::Axis::Analog(id, axis);
        for player in &mut self.players {
            if axis == player.controls.lr {
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

        for player in &mut self.players {
            player.update(&mut self.bullets);
        }

        while timer::check_update_time(ctx, DESIRED_FPS) {
            // fixed update
            for player in &mut self.players {
                player.fixed_update();
            }

            for bullet in &mut self.bullets {
                bullet.fixed_update(&mut self.players);
            }

            self.bullets.retain(|bullet| bullet.is_alive);

            for grid in &mut self.grids {
                grid.update();
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
            //  self.grids[1].damage_tile(
            //      thread_rng().gen_range(0, grid::GRID_WIDTH),
            //      thread_rng().gen_range(0, grid::GRID_HEIGHT),
            //  );
            // self.grids[1].damage_tile(
            //     thread_rng().gen_range(0, grid::GRID_WIDTH),
            //     thread_rng().gen_range(0, grid::GRID_HEIGHT),
            // );
            // self.grids[2].damage_tile(
            //     thread_rng().gen_range(0, grid::GRID_WIDTH),
            //     thread_rng().gen_range(0, grid::GRID_HEIGHT),
            // );
        }

        // If the bottom grid is dead, remove it, add a new grid, and make the other
        // grids start to fall
        if self.grids[0].state == GridState::Dead {
            self.grids.remove(0);
            self.grids.push(Grid::new_from_module(
                grid::GRID_HEIGHT as f32 * 3.0,
                rand::thread_rng().choose(&self.modules).unwrap().clone(),
            ));
            for (i, grid) in self.grids.iter_mut().enumerate() {
                let new_height = (i * grid::GRID_HEIGHT) as f32;
                grid.state = GridState::Falling(new_height);
            }
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx);

        for grid in &mut self.grids {
            grid.draw(ctx, self.leaves_image.clone())?;
        }

        for player in &self.players {
            player.draw(ctx)?;
        }

        for bullet in &self.bullets {
            bullet.draw(ctx)?;
        }
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

    let state = &mut MainState::new(ctx).unwrap();
    if let Err(e) = ggez::event::run(ctx, state) {
        println!("Error encountered: {}", e);
    }
}
