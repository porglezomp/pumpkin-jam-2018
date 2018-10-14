use std::{
    io::{BufRead, BufReader},
    path,
};

use ggez::{
    graphics::{DrawParam, Point2, Vector2},
    Context, GameResult,
};
use rand;
use rand::Rng;

use crate::collide::WorldRect;
use crate::draw::{self, Batch, WorldCoord};
use crate::math;
use crate::Images;

pub type Module = [[Tile; GRID_WIDTH]; GRID_HEIGHT];
pub type GridCoord = usize;

pub const GRID_WIDTH: GridCoord = 32;
pub const GRID_HEIGHT: GridCoord = 8;
pub const TILE_SIZE: WorldCoord = 1.0f32;
pub const TILE_MAX_HEALTH: u8 = 4;
const GRID_TO_WORLD: f32 = TILE_SIZE as f32 * draw::WORLD_WIDTH / GRID_WIDTH as f32;

pub const GRID_FALLING_ACCEL: f32 = -25.0;
pub const DEATH_THRESHOLD: f32 = 0.95;
pub const NO_SPAWN_THRESHOLD: f32 = 0.5;

/// A grid contains the collidable tiles that our dynamic objects interact with
pub struct Grid {
    pub module: Module,       // Stored such that row zero is the bottom row
    pub world_offset: Point2, // lower left corner
    pub state: GridState,
    total_tiles: usize, // Number of tiles alive at the start
    tiles_alive: usize, // Number of tiles still currently alive
    pub vel: Vector2,
    acc: Vector2,
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
            vel: Vector2::new(0.0, 0.0),
            acc: Vector2::new(0.0, 0.0),
        }
    }

    pub fn height(&self) -> f32 {
        self.world_offset.y
    }

    pub fn fixed_update(&mut self) {
        use self::GridState::*;
        self.vel += self.acc * crate::DT;
        self.world_offset += self.vel * crate::DT;

        match self.state {
            AliveFalling(goal_height) | DeadFalling(goal_height) => {
                if self.world_offset.y < goal_height {
                    self.world_offset.y = goal_height;
                    self.state = GridState::Alive;
                    self.vel = Vector2::new(0.0, 0.0);
                    self.acc = Vector2::new(0.0, 0.0);
                }
            }
            Alive | Dead => (),
        }
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
                    if self.height() - grid_below.height() > 0.5 * TILE_SIZE + GRID_HEIGHT_F32 {
                        self.fall(goal_height + GRID_HEIGHT_F32);
                    }
                }
                _ => (),
            }
        }
    }

    pub fn draw(&mut self, ctx: &mut Context, images: &Images) -> GameResult<()> {
        use self::Tile::*;
        let mut batch = Batch::atlas(images.tiles.clone(), 16, 16);
        for (j, row) in self.module.iter().enumerate() {
            for (i, tile) in row.iter().enumerate() {
                let dest = Point2::new(TILE_SIZE * i as f32, TILE_SIZE * j as f32);
                match *tile {
                    Air => continue,
                    Tile::Start(idx) => batch.add(
                        17 + idx as usize,
                        DrawParam {
                            dest,
                            ..Default::default()
                        },
                    ),
                    Tile::Leave => batch.add(
                        16,
                        DrawParam {
                            dest,
                            ..Default::default()
                        },
                    ),
                    Solid(health) => {
                        // Tile is dead, don't need to render
                        if health == 0 {
                            continue;
                        }

                        let idx = (0 + (TILE_MAX_HEALTH - health)) as usize;
                        batch.add(
                            idx,
                            DrawParam {
                                dest,
                                ..Default::default()
                            },
                        );
                    }
                }
            }
        }
        let param = DrawParam {
            dest: self.world_offset,
            ..Default::default()
        };
        batch.draw(ctx, param)?;
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
            Leave | Start(_) | Air => (),
        }
    }

    pub fn fall(&mut self, goal_height: WorldCoord) {
        use self::GridState::*;
        self.state = match self.state {
            Alive => AliveFalling(goal_height),
            Dead => DeadFalling(goal_height),
            _ => unreachable!(),
        };
        self.acc = Vector2::new(0.0, GRID_FALLING_ACCEL);
    }

    pub fn percent_tiles_alive(&self) -> f32 {
        self.tiles_alive as f32 / self.total_tiles as f32
    }

    pub fn to_world_coords(&self, grid_coords: (GridCoord, GridCoord)) -> Point2 {
        self.world_offset + GRID_TO_WORLD * Vector2::new(grid_coords.0 as f32, grid_coords.1 as f32)
    }

    pub fn to_grid_x(&self, x: f32) -> f32 {
        (x - self.world_offset.x) / GRID_TO_WORLD
    }

    pub fn to_grid_y(&self, y: f32) -> f32 {
        (y - self.world_offset.y) / GRID_TO_WORLD
    }

    pub fn overlapping_tiles(&self, rect: WorldRect, out: &mut Vec<(Tile, usize, usize)>) {
        let left = self.to_grid_x(rect.left());
        let right = self.to_grid_x(rect.right());
        let top = self.to_grid_y(rect.y + rect.h);
        let bottom = self.to_grid_y(rect.y);

        if top < 0.0 || bottom > GRID_HEIGHT as f32 || left < 0.0 || right > GRID_WIDTH as f32 {
            return;
        }

        let left = math::clamp(0.0, (GRID_WIDTH - 1) as f32, left) as usize;
        let right = math::clamp(0.0, (GRID_WIDTH - 1) as f32, right) as usize;
        let bottom = math::clamp(0.0, (GRID_HEIGHT - 1) as f32, bottom) as usize;
        let top = math::clamp(0.0, (GRID_HEIGHT - 1) as f32, top) as usize;

        for x in left..=right {
            for y in bottom..=top {
                if self.module[y][x] == Tile::Air {
                    continue;
                }
                out.push((self.module[y][x], x, y));
            }
        }
    }

    pub fn to_world_collider(&self, tile: (Tile, usize, usize)) -> WorldRect {
        use self::Tile::*;
        const NO_RECT: WorldRect = WorldRect {
            x: -100.0,
            y: -100.0,
            w: 0.0,
            h: 0.0,
        };
        match tile.0 {
            Start(_) | Solid(_) => {
                let tile_point = self.to_world_coords((tile.1, tile.2));
                math::rect_from_point(tile_point, TILE_SIZE, TILE_SIZE)
            }
            Leave => NO_RECT,
            Air => unreachable!(),
        }
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
    let mut columns: Vec<usize> = (1..GRID_WIDTH - 1).collect();
    rand::thread_rng().shuffle(&mut columns);
    for x in columns {
        for y in 0..(GRID_HEIGHT - 2) {
            let mut good_location = true;
            for i in x - 1..=x + 1 {
                let ground_tile = module[y][i] != Tile::Air;
                let tile_above = module[y + 1][i] == Tile::Air;
                let tile_two_above = module[y + 2][i] == Tile::Air;
                if !(ground_tile && tile_above && tile_two_above) {
                    good_location = false;
                }
            }

            if good_location {
                return Some((x, y));
            } else {
                continue;
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

pub fn parse_modules_file<P: AsRef<path::Path>>(
    ctx: &mut Context,
    path: P,
) -> GameResult<Vec<Module>> {
    let file = ctx.filesystem.open(path)?;
    // let contents = &fs::read_to_string(path).unwrap();
    let mut modules_list = vec![];
    let lines: Vec<String> = BufReader::new(file).lines().collect::<Result<_, _>>()?;

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
            '[' => tiles[i] = Tile::Start(0),
            '!' => tiles[i] = Tile::Start(1),
            ']' => tiles[i] = Tile::Start(2),
            '?' => tiles[i] = Tile::Leave,
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

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Tile {
    Air,
    Solid(u8),
    Start(u8),
    Leave,
}
