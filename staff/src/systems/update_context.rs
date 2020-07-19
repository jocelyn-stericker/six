#![allow(clippy::type_complexity)]

use std::collections::HashMap;

use crate::{
    components::{Children, Chord, Context, Signature, Staff},
    PitchKind,
};
use num_rational::Rational;
use pitch::{key_signature_note_names, Clef, NoteModifier, NoteName};
use rhythm::{components::Bar, BarChild};
use specs::{Join, ReadStorage, System, WriteStorage};

#[derive(Debug, Default)]
pub struct UpdateContext;

impl<'a> System<'a> for UpdateContext {
    type SystemData = (
        ReadStorage<'a, Staff>,
        ReadStorage<'a, Children>,
        ReadStorage<'a, Bar>,
        ReadStorage<'a, Signature>,
        ReadStorage<'a, Chord>,
        WriteStorage<'a, Context>,
    );

    fn run(
        &mut self,
        (staffs, ordered_children, bars, signatures, chords, mut contexts): Self::SystemData,
    ) {
        for (_staff, Children(children)) in (&staffs, &ordered_children).join() {
            let mut idx = 0;
            let mut clef = Clef::G;
            let mut key = 0;
            let mut time = (4, 4);
            let mut def_accidentals: HashMap<(NoteName, i8), NoteModifier> = HashMap::new();
            let mut start_beat = Rational::new(0, 1);

            for (i, &child) in children.iter().enumerate() {
                // HACK: get correct correct pickup start
                if let Some(bar) = children.get(i + 1).and_then(|&c| bars.get(c)) {
                    start_beat = bar.skip();
                }

                if let Some(context) = contexts.get_mut(child) {
                    context.bar = idx;
                    context.clef = clef;
                    context.key = key;
                    context.time = time;
                    context.beat = start_beat;
                    context.accidentals = def_accidentals.clone();
                }
                if let Some(bar) = bars.get(child) {
                    let mut accidentals = def_accidentals.clone();
                    for BarChild { stencil, .. } in bar.children() {
                        if let (Some(context), Some(chord)) =
                            (contexts.get_mut(stencil), chords.get(stencil))
                        {
                            context.bar = idx;
                            context.clef = clef;
                            context.key = key;
                            context.time = time;
                            context.accidentals = accidentals.clone();

                            if let PitchKind::Pitch(pitch) = chord.pitch {
                                let pitch_base = (pitch.name(), pitch.octave());
                                if accidentals.get(&pitch_base).cloned() != pitch.modifier() {
                                    if let Some(modifier) = pitch.modifier() {
                                        accidentals.insert(pitch_base, modifier);
                                    } else {
                                        accidentals.remove(&pitch_base);
                                    }
                                }
                            }
                        }
                    }
                    idx += 1;
                }
                if let Some(signature) = signatures.get(child) {
                    if let Some(new_clef) = signature.clef {
                        clef = new_clef;
                    }
                    if let Some(new_key) = signature.key {
                        def_accidentals = HashMap::new();
                        key = new_key;
                        for (note_name, note_modifier) in key_signature_note_names(key) {
                            for octave in -2..=8 {
                                def_accidentals.insert((note_name, octave), note_modifier);
                            }
                        }
                    }
                    if let Some(new_time) = signature.time {
                        time = new_time;
                    }
                }
            }
        }
    }
}
