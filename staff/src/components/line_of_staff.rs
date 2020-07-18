use specs::{Component, Entity, VecStorage};

#[derive(Debug)]
pub struct LineOfStaff {
    pub width: f64,
    pub staff_lines: Entity,
}

impl Component for LineOfStaff {
    type Storage = VecStorage<Self>;
}

impl LineOfStaff {
    pub fn new(staff_lines: Entity) -> LineOfStaff {
        LineOfStaff {
            width: 0.0,
            staff_lines,
        }
    }
}
