use ggez::{graphics, timer, Context, GameResult};
use std::path;

struct MainState;

impl MainState {
    fn new(_ctx: &mut Context) -> GameResult<MainState> {
        Ok(MainState)
    }
}

impl ggez::event::EventHandler for MainState {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        const DESIRED_FPS: u32 = 60;
        // variable update
        while timer::check_update_time(ctx, DESIRED_FPS) {
            // fixed update
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx);

        // draw ???

        graphics::present(ctx);
        Ok(())
    }
}

fn main() {
    let c = ggez::conf::Conf::new();
    let ctx = &mut Context::load_from_conf("fall", "acgames", c).unwrap();

    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let mut path = path::PathBuf::from(manifest_dir);
    path.push("resources");
    ctx.filesystem.mount(&path, true);

    let state = &mut MainState::new(ctx).unwrap();
    if let Err(e) = ggez::event::run(ctx, state) {
        println!("Error encountered: {}", e);
    }
}
