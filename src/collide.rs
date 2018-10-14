use crate::grid::Grid;
use ggez::graphics::{Rect, Vector2};

use crate::math;

pub type WorldRect = Rect;
pub const COLLISION_TOLERANCE: f32 = 0.01;

pub fn get_overlapping_tiles(grids: &[Grid], rect: Rect, out: &mut Vec<WorldRect>) {
    let mut tiles = Vec::with_capacity(6);
    for grid in grids {
        tiles.clear();
        grid.overlapping_tiles(rect, &mut tiles);
        for &tile in &tiles {
            out.push(grid.to_world_collider(tile))
        }
    }
}

/// Give the horizontal displacement and velocity of a moving rectangle intersecting another rectangle
/// Assumes that the origin of the rectanges are at the lower left corner.
pub fn resolve_collider_horiz(rect: Rect, velocity: Vector2, fixed: Rect) -> (f32, Vector2) {
    if rect.overlaps(&fixed) {
        // Intersects while moving left, so push out right
        if velocity.x < 0.0 {
            return (
                fixed.right() - rect.left() + COLLISION_TOLERANCE,
                Vector2::new(0.0, velocity.y),
            );
        } else {
            return (
                fixed.left() - rect.right() - COLLISION_TOLERANCE,
                Vector2::new(0.0, velocity.y),
            );
        }
    }
    (0.0, velocity)
}

/// Give the vertical displacement and resulting velocity of a moving rectangle intersecting another rectangle
/// Assumes that the origin of the rectanges are at the lower left corner.
pub fn resolve_collider_vert(rect: Rect, velocity: Vector2, fixed: Rect) -> (f32, Vector2) {
    if rect.overlaps(&fixed) {
        // Intersects while moving up, so push out down
        if velocity.y > 0.0 {
            return (
                fixed.y - (rect.y + rect.h) - COLLISION_TOLERANCE,
                Vector2::new(velocity.x, 0.0),
            );
        } else {
            return (
                (fixed.y + fixed.h) - rect.y + COLLISION_TOLERANCE,
                Vector2::new(velocity.x, 0.0),
            );
        }
    }
    (0.0, velocity)
}

macro_rules! resolve_colliders {
    ($fname:ident, $worker:ident, $dim:ident) => {
        pub fn $fname(rect: Rect, velocity: Vector2, colliders: &[Rect]) -> (f32, Vector2) {
            let mut net_disp = Vector2::new(0.0, 0.0);
            let mut net_vel = velocity;
            for collider in colliders {
                let disp_rect = math::rect_from_point(rect.point() + net_disp, rect.w, rect.h);
                let (res_disp, res_vel) = $worker(disp_rect, net_vel, *collider);
                net_disp.$dim += res_disp;
                net_vel = res_vel;
            }
            (net_disp.$dim, net_vel)
        }
    };
}

resolve_colliders!(resolve_colliders_horiz, resolve_collider_horiz, x);
resolve_colliders!(resolve_colliders_vert, resolve_collider_vert, y);
