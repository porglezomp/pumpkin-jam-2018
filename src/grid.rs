use std::fs;
use std::path;

use ggez::{
    graphics::{spritebatch::SpriteBatch, Color, DrawParam, Image, Point2, Vector2},
    Context, GameResult,
};
use rand;
use rand::Rng;

use crate::draw;
use crate::draw::WorldCoord;

pub type GridCoord = usize;
pub type Module = [[Tile; GRID_WIDTH]; GRID_HEIGHT];

pub const GRID_WIDTH: GridCoord = 32;
pub const GRID_HEIGHT: GridCoord = 8;
pub const TILE_SIZE: WorldCoord = 1.0f32;
pub const TILE_MAX_HEALTH: usize = 5;

pub const DEATH_THRESHOLD: f32 = 0.25;
pub const NO_SPAWN_THRESHOLD: f32 = 0.5;
/// A grid contains the collidable tiles that our dynamic objects interact with
pub struct Grid {
    pub module: Module,       // Stored such that row zero is the bottom row
    pub world_offset: Point2, // lower left corner
    pub state: GridState,
    total_tiles: usize, // Number of tiles alive at the start
    tiles_alive: usize, // Number of tiles still currently alive
}

impl Grid {
    pub fn new_from_module(height: WorldCoord, module: Module) -> Grid {
        let total_tiles = total_tiles(module);
        Grid {
            module: module,
            world_offset: Point2::new(0.0, height),
            state: GridState::Alive,
            total_tiles: total_tiles,
            tiles_alive: total_tiles,
        }
    }

    pub fn height(&self) -> f32 {
        self.world_offset.y
    }

    pub fn update(&mut self, grid_below: Option<&Grid>) {
        use self::GridState::*;

        if self.percent_tiles_alive() < DEATH_THRESHOLD && self.state == Alive {
            self.state = Dead;
        }
        const GRID_HEIGHT_F32: f32 = TILE_SIZE * GRID_HEIGHT as f32;
        if let Some(grid_below) = grid_below {
            match (&self.state, &grid_below.state) {
                (Alive, AliveFalling(goal_height)) | (Alive, DeadFalling(goal_height)) => {
                    if self.height() - grid_below.height() > 2.0 * TILE_SIZE + GRID_HEIGHT_F32 {
                        self.state = AliveFalling(goal_height + GRID_HEIGHT_F32);
                    }
                }
                _ => (),
            }
        }

        match self.state {
            AliveFalling(goal_height) | DeadFalling(goal_height) => {
                self.world_offset.y -= 0.5;
                if (goal_height - self.world_offset.y).abs() < 0.1 {
                    self.world_offset.y = goal_height;
                    self.state = GridState::Alive;
                }
            }
            _ => (),
        }
    }

    pub fn draw(&mut self, ctx: &mut Context, image: Image) -> GameResult<()> {
        use self::Tile::*;
        let mut sprite_batch = SpriteBatch::new(image);
        for (j, row) in self.module.iter().enumerate() {
            for (i, tile) in row.iter().enumerate() {
                match *tile {
                    Air => continue,
                    Solid(health) => {
                        // Tile is dead, don't need to render
                        if health == 0 {
                            continue;
                        }
                        let draw_param = DrawParam {
                            dest: Point2::new(TILE_SIZE * i as f32, TILE_SIZE * j as f32),
                            color: Some(color_lerp(
                                RED,
                                WHITE,
                                (health - 1) as f32 / (TILE_MAX_HEALTH - 1) as f32,
                            )),
                            scale: Point2::new(1.0 / 32.0, 1.0 / 32.0),
                            ..Default::default()
                        };
                        sprite_batch.add(draw_param);
                    }
                }
            }
        }
        let param = DrawParam {
            dest: self.world_offset,
            ..Default::default()
        };
        draw::draw_ex(ctx, &sprite_batch, param)?;
        Ok(())
    }

    pub fn damage_tile(&mut self, x: GridCoord, y: GridCoord) {
        use self::Tile::*;
        match self.module[y][x] {
            Solid(ref mut health) => {
                if *health == 0 {
                    return;
                }

                *health -= 1;
                if *health == 0 {
                    self.tiles_alive -= 1;
                    self.module[y][x] = Tile::Air;
                }
            }
            Air => return,
        }
    }

    pub fn percent_tiles_alive(&self) -> f32 {
        self.tiles_alive as f32 / self.total_tiles as f32
    }

    pub fn to_world_coords(&self, grid_coords: (GridCoord, GridCoord)) -> Point2 {
        const SCALING_FACTOR: f32 = TILE_SIZE as f32 * draw::WORLD_WIDTH / GRID_WIDTH as f32;
        self.world_offset + Vector2::new(
            SCALING_FACTOR * grid_coords.1 as f32,
            SCALING_FACTOR * grid_coords.0 as f32,
        )
    }
}

#[derive(PartialEq)]
pub enum GridState {
    Alive,
    AliveFalling(WorldCoord), // Stores the target height to get to
    DeadFalling(WorldCoord),  // Stores the target height to get to
    Dead,
}

pub fn find_spawn_location(module: Module) -> Option<(GridCoord, GridCoord)> {
    let mut columns: Vec<usize> = (0..GRID_WIDTH).collect();
    rand::thread_rng().shuffle(&mut columns);
    for col in columns {
        for row in 0..(GRID_HEIGHT - 2) {
            let ground_tile = if let Tile::Solid(_) = module[row][col] {
                true
            } else {
                false
            };
            let tile_above = module[row + 1][col] == Tile::Air;
            let tile_two_above = module[row + 2][col] == Tile::Air;
            if ground_tile && tile_above && tile_two_above {
                return Some((row, col));
            }
        }
    }
    None
}

fn total_tiles(module: Module) -> usize {
    let mut total_tiles = 0;
    for row in module.iter() {
        for tile in row.iter() {
            match tile {
                Tile::Air => continue,
                _ => total_tiles += 1,
            }
        }
    }
    total_tiles
}

pub fn parse_modules_file(path: &path::Path) -> Result<Vec<Module>, String> {
    let contents = &fs::read_to_string(path).unwrap();
    let mut modules_list = vec![];
    let lines: Vec<&str> = contents.lines().collect();

    for module in lines.chunks(9) {
        // last line in file, stop parsing
        if module.len() == 1 {
            assert_eq!(module[0].trim(), "");
            break;
        }
        assert_eq!(module.len(), 9);
        assert_eq!(module[8].trim(), "-");
        let mut grid = [[Tile::Air; GRID_WIDTH]; GRID_HEIGHT];
        for (i, row) in module[..8].iter().enumerate() {
            grid[7 - i] = text_to_row(row)
                .map_err(|err| format!("Could not parse {} (line: {}) Reason: {}", row, i, err))?;
        }
        modules_list.push(grid);
    }
    Ok(modules_list)
}

fn text_to_row(row: &str) -> Result<[Tile; GRID_WIDTH], String> {
    let mut tiles = [Tile::Air; GRID_WIDTH];
    for (i, character) in row.trim_right().chars().enumerate() {
        match character {
            '#' => tiles[i] = Tile::Solid(TILE_MAX_HEALTH),
            ' ' => tiles[i] = Tile::Air,
            _ => {
                return Err(format!(
                    "Unknown Character: {} at position {}",
                    character, i
                ));
            }
        }
    }
    Ok(tiles)
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum Tile {
    Air,
    Solid(usize),
}

pub const WHITE: Color = Color {
    r: 1.0,
    g: 1.0,
    b: 1.0,
    a: 1.0,
};

pub const RED: Color = Color {
    r: 1.0,
    g: 0.0,
    b: 0.0,
    a: 1.0,
};

// todo : make this not stupid
pub fn color_lerp(a: Color, b: Color, t: f32) -> Color {
    fn f32_lerp(a: f32, b: f32, t: f32) -> f32 {
        a + (b - a) * t
    }

    Color::new(
        f32_lerp(a.r, b.r, t),
        f32_lerp(a.g, b.g, t),
        f32_lerp(a.b, b.b, t),
        f32_lerp(a.a, b.a, t),
    )
}
