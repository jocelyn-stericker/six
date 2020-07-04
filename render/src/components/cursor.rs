#[derive(Debug)]
struct Cursor();

impl Component for Cursor {
    type Storage = NullStorage<Self>;
}
