#[derive(Default, Clone, Copy)]
pub struct Block {
    pub is_solid: bool,
}

impl Block {
    pub fn is_solid(&self) -> bool {
        self.is_solid
    }
}
