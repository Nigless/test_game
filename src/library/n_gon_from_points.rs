use bevy::math::Vec2;

fn cross(o: Vec2, a: Vec2, b: Vec2) -> f32 {
    (a.x - o.x) * (b.y - o.y) - (a.y - o.y) * (b.x - o.x)
}

pub fn n_gon_from_points(mut points: Vec<Vec2>) -> Vec<Vec2> {
    points.sort_by(|a, b| {
        a.y.partial_cmp(&b.y)
            .unwrap()
            .then(a.x.partial_cmp(&b.x).unwrap())
    });

    let pivot = points[0];
    points.sort_by(|a, b| {
        let angle_a = (a - pivot).to_angle();
        let angle_b = (b - pivot).to_angle();
        angle_a.partial_cmp(&angle_b).unwrap()
    });

    let mut hull = Vec::new();
    for point in points {
        while hull.len() >= 2 && cross(hull[hull.len() - 2], hull[hull.len() - 1], point) <= 0.0 {
            hull.pop();
        }
        hull.push(point);
    }

    hull
}
