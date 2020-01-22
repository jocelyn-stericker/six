use num_rational::Rational;
use std::collections::HashMap;

use crate::RestNoteChord;
use entity::{Entity, Join};
use rhythm::RelativeRhythmicSpacing;

pub fn sys_relative_spacing(
    rnc: &HashMap<Entity, RestNoteChord>,
    parents: &HashMap<Entity, Entity>,
    spacing: &mut HashMap<Entity, RelativeRhythmicSpacing>,
) {
    let mut shortest_per_bar: HashMap<Entity, Rational> = HashMap::new();

    for (_id, (rnc, parent)) in (rnc, parents).join() {
        let dur = rnc.duration.duration();
        let entry = shortest_per_bar
            .entry(*parent)
            .or_insert(Rational::new(1, 8));
        *entry = (*entry).min(dur);
    }

    for (_id, (rnc, parent, spacing)) in (rnc, parents, spacing).join() {
        *spacing = RelativeRhythmicSpacing::new(shortest_per_bar[&parent], &rnc.duration);
    }
}
