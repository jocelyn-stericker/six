use std::cmp::Ordering;
use std::collections::HashMap;
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
    pub fn offset(self) -> i32 {
        match self {
            Clef::G | Clef::Percussion => 34,
            Clef::F => 22,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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
    pub fn new(i: u8) -> Option<NoteName> {
        match i {
            0 => Some(NoteName::C),
            2 => Some(NoteName::D),
            4 => Some(NoteName::E),
            5 => Some(NoteName::F),
            7 => Some(NoteName::G),
            9 => Some(NoteName::A),
            11 => Some(NoteName::B),
            _ => None,
        }
    }

    pub fn from_index(idx: u8) -> Option<NoteName> {
        match idx {
            0 => Some(NoteName::C),
            1 => Some(NoteName::D),
            2 => Some(NoteName::E),
            3 => Some(NoteName::F),
            4 => Some(NoteName::G),
            5 => Some(NoteName::A),
            6 => Some(NoteName::B),
            _ => None,
        }
    }

    pub fn index(self) -> i32 {
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

pub fn key_signature_note_names(key: i8) -> HashMap<NoteName, NoteModifier> {
    let sharps = [
        NoteName::F,
        NoteName::C,
        NoteName::G,
        NoteName::D,
        NoteName::A,
        NoteName::E,
        NoteName::B,
    ];

    let flats = [
        NoteName::B,
        NoteName::E,
        NoteName::A,
        NoteName::D,
        NoteName::G,
        NoteName::C,
        NoteName::F,
    ];

    let mut map = HashMap::new();

    match key.cmp(&0) {
        Ordering::Greater => {
            for sharp in &sharps[0..((key as usize).min(sharps.len()))] {
                map.insert(*sharp, NoteModifier::SemiUp);
            }
        }
        Ordering::Less => {
            for flat in &flats[0..((-key as usize).min(flats.len()))] {
                map.insert(*flat, NoteModifier::SemiDown);
            }
        }
        Ordering::Equal => {}
    };

    map
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i8)]
pub enum NoteModifier {
    SemiUp = 1,
    SemiDown = -1,
}

impl NoteModifier {
    pub fn from_raw(modifier: i8) -> Option<NoteModifier> {
        match modifier {
            1 => Some(NoteModifier::SemiUp),
            -1 => Some(NoteModifier::SemiDown),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Pitch {
    name: NoteName,
    modifier: Option<NoteModifier>,
    octave: i8,
}

impl Default for Pitch {
    fn default() -> Pitch {
        Pitch::middle_c()
    }
}

impl Pitch {
    pub fn new(name: NoteName, modifier: Option<NoteModifier>, octave: i8) -> Pitch {
        Pitch {
            name,
            modifier,
            octave,
        }
    }

    pub fn from_y(y: f64, clef: Clef, key: i8) -> Pitch {
        let pitch = clef.offset() + (y / 125f64) as i32;
        if pitch < 0 {
            // TODO
            Pitch::middle_c();
        }
        let octave = (pitch / 7) as i8;
        let name = NoteName::from_index((pitch % 7) as u8).unwrap();
        Pitch::new(
            name,
            key_signature_note_names(key).get(&name).copied(),
            octave,
        )
    }

    /// midi is a midi note that corresponds to a white key, and modifier is a modifier.
    pub fn from_base_midi(midi: u8, modifier: Option<NoteModifier>) -> Option<Pitch> {
        let octave = (midi / 12) as i8 - 1;
        let name = NoteName::new(midi % 12)?;
        Some(Pitch::new(name, modifier, octave))
    }

    pub fn a440() -> Pitch {
        Self::new(NoteName::A, None, 4)
    }

    pub fn middle_c() -> Pitch {
        Self::new(NoteName::C, None, 4)
    }

    pub fn base_midi(self) -> u8 {
        ((self.octave + 1) * 12 + (self.name as i8)) as u8
    }

    pub fn midi(self) -> u8 {
        self.base_midi() + (self.modifier.map(|m| m as i8).unwrap_or(0) as u8)
    }

    pub fn name(self) -> NoteName {
        self.name
    }

    pub fn modifier(self) -> Option<NoteModifier> {
        self.modifier
    }

    /// Scientific pitch notation octave.
    ///
    /// Octaves start at 60.
    ///
    /// Middle C (60) is C4.
    /// A440 is A4.
    pub fn octave(self) -> i8 {
        self.octave
    }

    pub fn y(self, clef: Clef) -> f64 {
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
