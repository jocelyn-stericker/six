use specs::{Component, Entity, VecStorage};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Parent(pub Entity);

impl Component for Parent {
    type Storage = VecStorage<Self>;
}
