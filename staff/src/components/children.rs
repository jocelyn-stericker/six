use specs::{Component, Entity, VecStorage};

#[derive(Debug, Default, Clone)]
pub struct Children(pub Vec<Entity>);

impl Component for Children {
    type Storage = VecStorage<Self>;
}
