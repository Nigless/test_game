pub enum Side {
    Front,
    Back,
    Top,
    Bottom,
    Left,
    Right,
}

impl Side {
    pub fn to_positions(&self, local_x: usize, local_y: usize, local_z: usize) -> [[f32; 3]; 6] {
        let local_x = local_x as f32;
        let local_y = local_y as f32;
        let local_z = local_z as f32;

        match self {
            Self::Front => [
                [local_x + 0.5, local_y + 0.5, local_z - 0.5],
                [local_x + 0.5, local_y - 0.5, local_z + 0.5],
                [local_x + 0.5, local_y - 0.5, local_z - 0.5],
                [local_x + 0.5, local_y - 0.5, local_z + 0.5],
                [local_x + 0.5, local_y + 0.5, local_z - 0.5],
                [local_x + 0.5, local_y + 0.5, local_z + 0.5],
            ],
            Self::Back => [
                [local_x - 0.5, local_y + 0.5, local_z + 0.5],
                [local_x - 0.5, local_y + 0.5, local_z - 0.5],
                [local_x - 0.5, local_y - 0.5, local_z + 0.5],
                [local_x - 0.5, local_y - 0.5, local_z - 0.5],
                [local_x - 0.5, local_y - 0.5, local_z + 0.5],
                [local_x - 0.5, local_y + 0.5, local_z - 0.5],
            ],
            Side::Top => [
                [local_x - 0.5, local_y + 0.5, local_z + 0.5],
                [local_x + 0.5, local_y + 0.5, local_z - 0.5],
                [local_x - 0.5, local_y + 0.5, local_z - 0.5],
                [local_x + 0.5, local_y + 0.5, local_z - 0.5],
                [local_x - 0.5, local_y + 0.5, local_z + 0.5],
                [local_x + 0.5, local_y + 0.5, local_z + 0.5],
            ],
            Side::Bottom => [
                [local_x + 0.5, local_y - 0.5, local_z + 0.5],
                [local_x - 0.5, local_y - 0.5, local_z + 0.5],
                [local_x + 0.5, local_y - 0.5, local_z - 0.5],
                [local_x - 0.5, local_y - 0.5, local_z - 0.5],
                [local_x + 0.5, local_y - 0.5, local_z - 0.5],
                [local_x - 0.5, local_y - 0.5, local_z + 0.5],
            ],
            Side::Left => [
                [local_x + 0.5, local_y - 0.5, local_z + 0.5],
                [local_x - 0.5, local_y + 0.5, local_z + 0.5],
                [local_x - 0.5, local_y - 0.5, local_z + 0.5],
                [local_x - 0.5, local_y + 0.5, local_z + 0.5],
                [local_x + 0.5, local_y - 0.5, local_z + 0.5],
                [local_x + 0.5, local_y + 0.5, local_z + 0.5],
            ],
            Side::Right => [
                [local_x + 0.5, local_y + 0.5, local_z - 0.5],
                [local_x + 0.5, local_y - 0.5, local_z - 0.5],
                [local_x - 0.5, local_y + 0.5, local_z - 0.5],
                [local_x - 0.5, local_y - 0.5, local_z - 0.5],
                [local_x - 0.5, local_y + 0.5, local_z - 0.5],
                [local_x + 0.5, local_y - 0.5, local_z - 0.5],
            ],
        }
    }
    pub fn to_normals(&self) -> [[f32; 3]; 6] {
        match self {
            Side::Front => [[1.0, 0.0, 0.0]; 6],
            Side::Back => [[-1.0, 0.0, 0.0]; 6],
            Side::Top => [[0.0, 1.0, 0.0]; 6],
            Side::Bottom => [[0.0, -1.0, 0.0]; 6],
            Side::Left => [[0.0, 0.0, 1.0]; 6],
            Side::Right => [[0.0, 0.0, -1.0]; 6],
        }
    }
    pub fn to_uvs(uv_x: f32, uv_y: f32) -> [[f32; 2]; 6] {
        [
            [uv_x, uv_y],
            [uv_y, uv_x],
            [uv_y, uv_y],
            [uv_y, uv_x],
            [uv_x, uv_y],
            [uv_x, uv_x],
        ]
    }
}
