use std::path;

use ggez::{
    audio::{SoundData, Source},
    error::GameError,
    Context, GameResult,
};
use rand::Rng;

const JUMPS_PATH: &[&str] = &[
    "/jump1.ogg",
    "/jump2.ogg",
    "/jump3.ogg",
    "/jump4.ogg",
    "/jump5.ogg",
];
const BREAK_BLOCKS_PATH: &[&str] = &["/break1.ogg", "/break2.ogg", "/break3.ogg"];
const SHOOT_PATH: &[&str] = &["/shoot1.ogg", "/shoot2.ogg", "/shoot3.ogg", "/shoot4.ogg"];
const LAND_PATH: &[&str] = &["/land1.ogg"];

pub struct Sound {
    jump: Vec<SoundData>,
    land: Vec<SoundData>,
    shoot: Vec<SoundData>,
    break_block: Vec<SoundData>,
    sources: Vec<Source>,
}

impl Sound {
    pub fn new(ctx: &mut Context) -> GameResult<Sound> {
        Ok(Sound {
            jump: to_sounds(ctx, JUMPS_PATH)?,
            break_block: to_sounds(ctx, BREAK_BLOCKS_PATH)?,
            shoot: to_sounds(ctx, SHOOT_PATH)?,
            land: to_sounds(ctx, LAND_PATH)?,
            sources: vec![],
        })
    }

    pub fn update(&mut self) {
        // Drop non playing sounds
        self.sources.retain(|source| source.playing());
    }

    pub fn play_sound(&mut self, ctx: &mut Context, sound: SoundEffect) -> GameResult<()> {
        use self::SoundEffect::*;
        match sound {
            Jump => {
                let source = get_random(ctx, &self.jump)?;
                source.play()?;
                assert!(source.playing());
                self.sources.push(source);
                println!("{:?}", self.sources);
                Ok(())
            }
            _ => unimplemented!(),
        }
    }
}

#[derive(Debug)]
pub enum SoundEffect {
    Jump,
    Land,
    Shoot,
    BreakBlock,
}

fn get_random(ctx: &mut Context, sounds: &[SoundData]) -> GameResult<Source> {
    if let Some(sound) = rand::thread_rng().choose(&sounds) {
        return Source::from_data(ctx, sound.clone());
    }
    Err(GameError::AudioError("Could not play sound".to_owned()))
}

fn to_sounds<P: AsRef<path::Path>>(ctx: &mut Context, paths: &[P]) -> GameResult<Vec<SoundData>> {
    paths.iter().map(|path| SoundData::new(ctx, path)).collect()
}
