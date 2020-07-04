#[derive(Debug)]
struct BeamForChord(Entity);

impl Component for BeamForChord {
    type Storage = BTreeStorage<Self>;
}
