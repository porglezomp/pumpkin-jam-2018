use ggez::graphics::{Point2, Rect};

pub fn clamp(lower: f32, upper: f32, n: f32) -> f32 {
    if upper < n {
        return upper;
    } else if lower > n {
        return lower;
    }
    n
}

/// Makes a rect from a given point
pub fn rect_from_point(point: Point2, w: f32, h: f32) -> Rect {
    Rect {
        x: point.x,
        y: point.y,
        w,
        h,
    }
}
