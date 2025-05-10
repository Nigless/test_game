use bevy::math::Vec3;

pub fn move_toward(from: Vec3, to: Vec3, delta: f32) -> Vec3 {
    let direction = to - from;

    if direction.length() <= delta {
        to
    } else {
        from + direction.normalize_or_zero() * delta
    }
}
