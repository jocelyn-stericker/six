#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct RhythmicBeaming {
    /// Number of beams coming into this note. 0 if this is the start.
    pub entering: u8,

    /// Number of beams coming out of this note. 0 if this is the start.
    pub leaving: u8,
}
