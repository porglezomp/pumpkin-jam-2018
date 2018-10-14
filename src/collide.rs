use ggez::graphics::{Rect, Vector2};

use crate::math;

pub type WorldRect = Rect;
pub const COLLISION_TOLERANCE: f32 = 0.01;

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

pub fn resolve_colliders_horiz(
    rect: Rect,
    velocity: Vector2,
    colliders: &[Rect],
) -> (f32, Vector2) {
    let mut net_disp = Vector2::new(0.0, 0.0);
    let mut net_vel = velocity;
    for collider in colliders {
        let disp_rect = math::rect_from_point(rect.point() + net_disp, rect.w, rect.h);
        let (res_x_disp, res_vel) = resolve_collider_horiz(disp_rect, net_vel, *collider);
        net_disp.x += res_x_disp;
        net_vel = res_vel;
    }
    (net_disp.x, net_vel)
}

pub fn resolve_colliders_vert(rect: Rect, velocity: Vector2, colliders: &[Rect]) -> (f32, Vector2) {
    let mut net_disp = Vector2::new(0.0, 0.0);
    let mut net_vel = velocity;
    for collider in colliders {
        let disp_rect = math::rect_from_point(rect.point() + net_disp, rect.w, rect.h);
        let (res_y_disp, res_vel) = resolve_collider_vert(disp_rect, net_vel, *collider);
        net_disp.y += res_y_disp;
        net_vel = res_vel;
    }
    (net_disp.y, net_vel)
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
