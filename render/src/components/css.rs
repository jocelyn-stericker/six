use specs::{Component, VecStorage};

#[derive(Clone, Debug, Default)]
pub struct Css {
    pub class: Option<String>,
}

impl Component for Css {
    type Storage = VecStorage<Css>;
}
