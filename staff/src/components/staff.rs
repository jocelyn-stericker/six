use specs::{Component, Entity, VecStorage};

#[derive(Debug, Default)]
pub struct Staff {
    /// This is a line of a staff, not the 5 staff lines.
    pub lines: Vec<Entity>,
}

impl Component for Staff {
    type Storage = VecStorage<Self>;
}
