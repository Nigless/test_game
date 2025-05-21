use bevy::math::Vec2;

pub fn n_gon_area(vertices: Vec<Vec2>) -> f32 {
    let n = vertices.len();
    let mut area = 0.0;

    for i in 0..n {
        let j = (i + 1) % n;
        area += vertices[i].x * vertices[j].y - vertices[j].x * vertices[i].y;
    }

    (area / 2.0).abs()
}
