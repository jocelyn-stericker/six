use specs::{Component, Entity, VecStorage};

#[derive(Debug)]
pub struct BeamForChord(pub Entity);

impl Component for BeamForChord {
    type Storage = VecStorage<Self>;
}
