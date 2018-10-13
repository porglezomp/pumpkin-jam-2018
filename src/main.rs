use std::path;

use ggez::{
    conf::{WindowMode, WindowSetup},
    event,
    graphics::{self, Point2, Rect},
    timer, Context, ContextBuilder, GameResult,
};

use crate::player::Player;

mod player;

const DT: f32 = 1.0 / 60.0;

struct MainState {
    players: Vec<Player>,
}

pub fn draw_pos(p: Point2) -> Point2 {
    Point2::new(p.x, 24.0 - p.y)
}

impl MainState {
    fn new(ctx: &mut Context) -> GameResult<Self> {
        graphics::set_screen_coordinates(
            ctx,
            Rect {
                x: 0.0,
                y: 0.0,
                h: 24.0,
                w: 32.0,
            },
        )?;

        let players = vec![Player::new(ctx)?];
        Ok(MainState { players })
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

impl ggez::event::EventHandler for MainState {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        const DESIRED_FPS: u32 = 60;

        for player in &mut self.players {
            player.update();
        }

        while timer::check_update_time(ctx, DESIRED_FPS) {
            for player in &mut self.players {
                player.fixed_update();
            }
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx);

        for player in &self.players {
            player.draw(ctx)?;
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
            width: 960,
            height: 720,
            ..Default::default()
        })
        .build()
        .unwrap();

    let state = &mut MainState::new(ctx).unwrap();
    if let Err(e) = ggez::event::run(ctx, state) {
        println!("Error encountered: {}", e);
    }
}
