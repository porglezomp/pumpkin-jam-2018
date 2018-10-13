use std::path;

use ggez::{
    conf::{WindowMode, WindowSetup},
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
}

impl ggez::event::EventHandler for MainState {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        const DESIRED_FPS: u32 = 60;

        for player in &mut self.players {
            player.update();
        }

        while timer::check_update_time(ctx, DESIRED_FPS) {
            // fixed update
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
