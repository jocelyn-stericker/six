#[derive(Debug)]
struct FlagAttachment(Entity);

impl Component for FlagAttachment {
    type Storage = BTreeStorage<Self>;
}

