use std::io::Read;

use ggez::{Context, GameResult};

use crate::draw::WorldCoord;
use crate::grid::GridCoord;

const CONFIG_FILE: &str = "/config.toml";

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
            use super::$Name;
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
        max_health: u8 = 3,
        height: f32 = 0.8,
        width: f32 = 0.8,
        shoot_offset_x: f32 = 0.8 / 1.5,
        shoot_offset_y: f32 = 0.8 / 1.5,
        jump_power: f32 = 16.0,
    };

    config GRID: grid = Grid {
        tile_max_health: u8 = 4,
        falling_accel: f32 = -25.0,
        death_threshold: f32 = 0.95,
        no_spawn_threshold: f32 = 0.5,
    };

}
