use pitch::Pitch;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum PitchKind {
    Rest,
    Unpitched,
    Pitch(Pitch),
}

impl PitchKind {
    pub fn is_rest(self) -> bool {
        self == PitchKind::Rest
    }
}

