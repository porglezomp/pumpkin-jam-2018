use std::io::Read;

use ggez::{Context, GameResult};

use crate::draw;

const CONFIG_FILE: &str = "/config.toml";

#[derive(Deserialize)]
pub struct MenuInfo {
    pub join_pos: (f32, f32),
    pub heart_pos: (f32, f32),
    pub ready_pos: (f32, f32),
    pub life_pos: (f32, f32),
}

macro_rules! config {
    ($($(const $CNAME:ident: $cty:ty = $cval:expr;)* config $NAME:ident: $fieldname:ident = $Name:ident {
        $($(#[$meta:meta])* $var:ident : $ty:ty = $val:expr,)*
    };)*) => {$(
        #[derive(Deserialize)]
        pub struct $Name {
            $($(#[$meta])* pub $var: $ty,)*
        }

        $(const $CNAME: $cty = $cval;)*

        #[allow(non_snake_case)]
        mod $NAME {
            #[allow(unused)]
            use super::*;
            pub static mut $NAME: $Name = $Name {
                $($var: $val,)*
            };

            pub struct Config;
            impl std::ops::Deref for Config {
                type Target = $Name;
                fn deref(&self) -> &$Name {
                    unsafe { &$NAME }
                }
            }
        }

        pub static $NAME: $NAME::Config = $NAME::Config;
        )*

        #[derive(Deserialize)]
        struct Config {
            $($fieldname: $Name,)*
        }

        pub fn load(ctx: &mut Context) -> GameResult<()> {
            let mut file = ctx.filesystem.open(CONFIG_FILE)?;
            let mut text = String::new();
            file.read_to_string(&mut text)?;

            let config: Config = toml::from_str(&text).map_err(
                |err| format!("Error loading '{}' {}", CONFIG_FILE, err)
            )?;
            $(unsafe { $NAME::$NAME = config.$fieldname; };)*
            Ok(())
        }
    }
}

config! {
    config TEAM: team = Team {
        colors: [[f32; 4]; 4] = [
            [0.25, 0.7, 1.0, 1.0],
            [0.8, 0.2, 0.2, 1.0],
            [0.3, 1.0, 0.5, 1.0],
            [1.0, 0.9, 0.2, 1.0],
        ],
    };

    config PLAYER: player = Player {
        max_health: u8 = 4,
        max_lives: u8 = 4,
        height: f32 = 0.8,
        width: f32 = 0.8,
        shoot_offset_x: f32 = 0.8 / 1.5,
        shoot_offset_y: f32 = 0.8 / 1.5,
        jump_power: f32 = 16.0,
        second_jump_power: f32 = 16.0,

    };

    config GRID: grid = Grid {
        tile_max_health: u8 = 4,
        falling_accel: f32 = -25.0,
        death_threshold: f32 = 0.95,
        no_spawn_threshold: f32 = 0.5,
        gap: f32 = 0.5,
    };

    config MENU: menu = Menu {
        life_offset: (f32, f32) = (1.2, 0.0),
        heart_offset: (f32, f32) = (0.7, 0.0),
        pos: [MenuInfo; 4] = [
            MenuInfo {
                join_pos: (draw::WORLD_WIDTH - 9.0, 1.0),
                heart_pos: (draw::WORLD_WIDTH - 3.0, 1.0),
                ready_pos: (draw::WORLD_WIDTH - 4.5, 2.0),
                life_pos: (draw::WORLD_WIDTH - 4.5, 2.0),
            },
            MenuInfo {
                join_pos:(1.0, 1.0),
                heart_pos: (1.0, 1.0),
                ready_pos: (0.5, 2.0),
                life_pos: (0.5, 2.0),
            },
            MenuInfo {
                join_pos: (1.0, draw::WORLD_HEIGHT - 3.0),
                heart_pos:(1.0, draw::WORLD_HEIGHT - 2.0),
                ready_pos: (0.5, draw::WORLD_HEIGHT - 3.0),
                life_pos: (0.5, draw::WORLD_HEIGHT - 3.0),
            },
            MenuInfo {
                join_pos: (draw::WORLD_WIDTH - 9.0, draw::WORLD_HEIGHT - 3.0),
                heart_pos: (draw::WORLD_WIDTH - 3.0, draw::WORLD_HEIGHT - 2.0),
                ready_pos: (draw::WORLD_WIDTH - 4.5, draw::WORLD_HEIGHT - 3.0),
                life_pos: (draw::WORLD_WIDTH - 4.5, draw::WORLD_HEIGHT - 3.0),
            },
        ],
    };
}
