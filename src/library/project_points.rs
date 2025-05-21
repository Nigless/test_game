use bevy::math::{Quat, Vec2, Vec3};

pub fn project_points(points: Vec<Vec3>, normal: Vec3) -> Vec<Vec2> {
    points
        .iter()
        .map(|p| Quat::from_rotation_arc(normal, Vec3::Z) * p.reject_from(normal))
        .map(|p| Vec2::new(p.x, p.y))
        .collect()
}
