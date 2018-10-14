use ggez::graphics::{Rect, Vector2};

pub type WorldRect = Rect;

/// Give the horizontal displacement and velocity of a moving rectangle intersecting another rectangle
/// Assumes that the origin of the rectanges are at the lower left corner.
pub fn collision_resolve_horiz(rect: Rect, velocity: Vector2, fixed: Rect) -> (f32, Vector2) {
    if rect.overlaps(&fixed) {
        // Intersects while moving left, so push out right
        if velocity.x < 0.0 {
            return (
                fixed.right() - rect.left() + 0.01,
                Vector2::new(0.0, velocity.y),
            );
        } else {
            return (
                fixed.left() - rect.right() - 0.01,
                Vector2::new(0.0, velocity.y),
            );
        }
    }
    (0.0, velocity)
}

/// Give the vertical displacement and resulting velocity of a moving rectangle intersecting another rectangle
/// Assumes that the origin of the rectanges are at the lower left corner.
pub fn collision_resolve_vert(rect: Rect, velocity: Vector2, fixed: Rect) -> (f32, Vector2) {
    if rect.overlaps(&fixed) {
        // Intersects while moving up, so push out down
        if velocity.y > 0.0 {
            return (
                fixed.y - (rect.y + rect.h) - 0.01,
                Vector2::new(velocity.x, 0.0),
            );
        } else {
            return (
                (fixed.y + fixed.h) - rect.y + 0.01,
                Vector2::new(velocity.x, 0.0),
            );
        }
    }
    (0.0, velocity)
}
