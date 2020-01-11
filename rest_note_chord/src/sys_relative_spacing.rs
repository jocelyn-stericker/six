use num_rational::Rational;
use std::collections::HashMap;

use crate::RestNoteChord;
use entity::{Entity, Join};
use rhythm::RelativeRhythmicSpacing;

pub fn sys_relative_spacing(
    rnc: &HashMap<Entity, RestNoteChord>,
    spacing: &mut HashMap<Entity, RelativeRhythmicSpacing>,
) {
    let shortest = Rational::new(1, 32);

    for (_id, (rnc, spacing)) in (rnc, spacing).join() {
        *spacing = RelativeRhythmicSpacing::new(shortest, &rnc.duration);
    }
}
