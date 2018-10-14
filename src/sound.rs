use std::path;

use ggez::{
    audio::{SoundData, Source},
    error::GameError,
    Context, GameResult,
};
use rand::Rng;

const BREAK_BLOCKS: &str = "/break1.ogg";
const SHOOT: &[&str; 4] = &["/shoot1.ogg", "/shoot2.ogg", "/shoot3.ogg", "/shoot4.ogg"];
const LAND: &str = "/land1.ogg";

const JUMP: &str = "/jump1.ogg";
const SECOND_JUMP: &str = "/jump5.ogg";

pub struct Sound {
    jump: SoundData,
    second_jump: SoundData,
    land: SoundData,
    shoot: Vec<SoundData>,
    break_block: SoundData,
    sources: Vec<Source>,
}

impl Sound {
    pub fn new(ctx: &mut Context) -> GameResult<Sound> {
        Ok(Sound {
            jump: SoundData::new(ctx, JUMP)?,
            second_jump: SoundData::new(ctx, SECOND_JUMP)?,
            break_block: SoundData::new(ctx, BREAK_BLOCKS)?,
            shoot: to_sounds(ctx, SHOOT)?,
            land: SoundData::new(ctx, LAND)?,
            sources: vec![],
        })
    }

    pub fn update(&mut self) {
        // Drop non playing sounds
        self.sources.retain(|source| source.playing());
    }

    pub fn play_sound(&mut self, ctx: &mut Context, sound: SoundEffect) {
        use self::SoundEffect::*;
        let (sound, volume) = match sound {
            Jump => (self.jump.clone(), 0.5),
            SecondJump => (self.second_jump.clone(), 0.4),
            Shoot => (get_random(&self.shoot), 0.3),
            Land => (self.land.clone(), 0.2),
            _ => unimplemented!(),
        };
        self.play(ctx, sound, volume).expect("Couldn't play sound");
    }

    fn play(&mut self, ctx: &mut Context, sound: SoundData, volume: f32) -> GameResult<()> {
        let mut source = Source::from_data(ctx, sound)?;
        source.set_volume(volume);
        source.play()?;
        self.sources.push(source);
        Ok(())
    }
}

#[derive(Debug)]
pub enum SoundEffect {
    Jump,
    SecondJump,
    Land,
    Shoot,
    BreakBlock,
}

fn get_random(sounds: &[SoundData]) -> SoundData {
    rand::thread_rng().choose(&sounds).unwrap().clone()
}

fn to_sounds<P: AsRef<path::Path>>(ctx: &mut Context, paths: &[P]) -> GameResult<Vec<SoundData>> {
    paths.iter().map(|path| SoundData::new(ctx, path)).collect()
}
