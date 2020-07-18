use kurbo::Point;
use specs::{Component, VecStorage};

#[derive(Debug, Default)]
pub struct FlagAttachment(pub Option<Point>);

impl Component for FlagAttachment {
    type Storage = VecStorage<Self>;
}
