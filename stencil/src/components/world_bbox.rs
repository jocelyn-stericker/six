use kurbo::Rect;
use specs::{Component, VecStorage};

#[derive(Debug)]
pub struct WorldBbox(Rect);

impl Component for WorldBbox {
    type Storage = VecStorage<Self>;
}

