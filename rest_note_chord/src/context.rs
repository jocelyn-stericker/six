use num_rational::Rational;
use pitch::Clef;

#[derive(Debug, Clone, Copy)]
/// Where in the line an entity is.
pub enum Place {
    /// First element in a line.
    Start,
    /// Not the start or end of a line.
    Mid,
    /// Last element in a line.
    End,
}

#[derive(Debug, Clone, Copy)]
pub struct Context {
    pub bar: usize,
    pub beat: Rational,
    pub natural_beat: Rational,
    pub clef: Clef,
    pub key: i8,
    pub time: (u8, u8),
    pub place: Place,
}

impl Default for Context {
    fn default() -> Context {
        Context {
            bar: 0,
            beat: Rational::new(0, 1),
            natural_beat: Rational::new(0, 1),
            clef: Clef::G,
            key: 0,
            time: (4, 4),
            place: Place::Mid,
        }
    }
}
