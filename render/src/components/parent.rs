#[derive(Debug)]
struct Parent(Entity);

impl Component for Parent {
    type Storage = VecStorage<Self>;
}
