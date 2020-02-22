use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Clef {
    G,
    F,
    Percussion,
}

impl Clef {
    /// Y-position of C0, in steps.
    pub fn offset(&self) -> i32 {
        match self {
            Clef::G | Clef::Percussion => 34,
            Clef::F => 22,
        }
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum NoteName {
    C = 0,
    D = 2,
    E = 4,
    F = 5,
    G = 7,
    A = 9,
    B = 11,
}

impl NoteName {
    pub fn index(&self) -> i32 {
        match self {
            NoteName::C => 0,
            NoteName::D => 1,
            NoteName::E => 2,
            NoteName::F => 3,
            NoteName::G => 4,
            NoteName::A => 5,
            NoteName::B => 6,
        }
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(i8)]
pub enum NoteModifier {
    SemiUp = 1,
    SemiDown = -1,
}

#[derive(Debug, Clone, Copy)]
pub struct Pitch {
    name: NoteName,
    modifier: Option<NoteModifier>,
    octave: i8,
}

impl Pitch {
    pub fn new(name: NoteName, modifier: Option<NoteModifier>, octave: i8) -> Pitch {
        Pitch {
            name,
            modifier,
            octave,
        }
    }

    pub fn a440() -> Pitch {
        Self::new(NoteName::A, None, 4)
    }

    pub fn middle_c() -> Pitch {
        Self::new(NoteName::C, None, 4)
    }

    pub fn midi(&self) -> u8 {
        ((self.octave + 1) * 12 + (self.name as i8) + self.modifier.map(|m| m as i8).unwrap_or(0))
            as u8
    }

    pub fn name(&self) -> NoteName {
        self.name
    }

    pub fn modifier(&self) -> Option<NoteModifier> {
        self.modifier
    }

    /// Scientific pitch notation octave.
    ///
    /// Octaves start at 60.
    ///
    /// Middle C (60) is C4.
    /// A440 is A4.
    pub fn octave(&self) -> i8 {
        self.octave
    }

    pub fn y(&self, clef: Clef) -> f64 {
        (clef.offset() - self.name().index() - 7 * (self.octave() as i32)) as f64 * 125f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        assert_eq!(Pitch::a440().midi(), 69);
        assert_eq!(Pitch::middle_c().midi(), 60);
        assert_eq!(
            Pitch::new(NoteName::B, Some(NoteModifier::SemiUp), 3).midi(),
            60
        );
    }
}
