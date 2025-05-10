use std::f32::consts;

use bevy::math::Vec3;

pub fn fibonacci_sphere(n: usize) -> Vec<Vec3> {
    let mut points = Vec::with_capacity(n);
    let golden_ratio = (1.0 + 5.0_f32.sqrt()) / 2.0;
    let angle_increment = 2.0 * consts::PI / golden_ratio;

    for i in 0..n {
        let t = i as f32 / (n as f32 - 1.0);
        let z = 1.0 - 2.0 * t;
        let radius = (1.0 - z * z).sqrt();
        let theta = angle_increment * i as f32;
        let x = theta.cos() * radius;
        let y = theta.sin() * radius;
        points.push(Vec3::new(x, y, z).normalize());
    }

    points
}
