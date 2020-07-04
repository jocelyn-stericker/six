use crate::BeamAttachement;
use specs::{storage::BTreeStorage, Component};

#[derive(Debug)]
pub struct Beam(pub Vec<BeamAttachement>);

impl Component for Beam {
    type Storage = BTreeStorage<Beam>;
}
