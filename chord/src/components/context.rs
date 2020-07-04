use num_rational::Rational;
use pitch::{Clef, NoteModifier, NoteName};
use specs::{Component, VecStorage};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Context {
    pub bar: usize,
    pub beat: Rational,
    pub natural_beat: Rational,
    pub clef: Clef,
    pub key: i8,
    pub time: (u8, u8),
    pub accidentals: HashMap<(NoteName, i8), NoteModifier>,
}

impl Component for Context {
    type Storage = VecStorage<Self>;
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
            accidentals: HashMap::new(),
        }
    }
}
