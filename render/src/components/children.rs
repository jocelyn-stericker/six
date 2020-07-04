#[derive(Debug)]
struct Children(Vec<Entity>);

impl Component for Children {
    type Storage = VecStorage<Self>;
}

