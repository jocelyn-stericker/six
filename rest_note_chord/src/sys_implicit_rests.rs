use std::collections::HashMap;

use crate::RestNoteChord;
use entity::{EntitiesRes, Entity, Join};
use num_rational::Rational;
use rhythm::{Bar, RelativeRhythmicSpacing};
use stencil::Stencil;

pub fn sys_implicit_rests(
    entities: &EntitiesRes,
    rnc: &mut HashMap<Entity, RestNoteChord>,
    bars: &mut HashMap<Entity, Bar>,
    spacing: &mut HashMap<Entity, RelativeRhythmicSpacing>,
    parents: &mut HashMap<Entity, Entity>,
    render: &mut HashMap<Entity, Stencil>,
) {
    for (_key, rnc) in rnc.join() {
        rnc.start = Rational::new(-1, 1);
    }

    for (bar_id, bar) in bars {
        while let Some((duration, entity)) = bar.push_managed_entity(entities) {
            // TODO: get correct start
            rnc.insert(
                entity,
                RestNoteChord::new(duration, false, Rational::new(0, 1)),
            );
            spacing.insert(entity, RelativeRhythmicSpacing::default());
            parents.insert(entity, *bar_id);
            render.insert(entity, Stencil::default());
        }

        while let Some(entity) = bar.pop_managed_entity() {
            rnc.remove(&entity);
            spacing.remove(&entity);
            parents.remove(&entity);
            render.remove(&entity);
        }

        for (duration, start, entity, is_managed) in bar.children() {
            if let Some(rnc) = rnc.get_mut(&entity) {
                rnc.duration = duration;
                rnc.start = start;
                if is_managed {
                    rnc.natural_duration = duration;
                    rnc.natural_start = rnc.start;
                }
            }
        }
    }
}
