use crate::BeamAttachment;
use specs::{Component, VecStorage};

#[derive(Debug)]
pub struct Beam(pub Vec<BeamAttachment>);

impl Component for Beam {
    type Storage = VecStorage<Beam>;
}
