use std::collections::HashMap;

use crate::RestNoteChord;
use entity::{EntitiesRes, Entity};
use rhythm::{Bar, RelativeRhythmicSpacing};
use stencil::Stencil;

pub fn sys_implicit_rests(
    entities: &EntitiesRes,
    rnc: &mut HashMap<Entity, RestNoteChord>,
    bars: &mut HashMap<Entity, Bar>,
    spacing: &mut HashMap<Entity, RelativeRhythmicSpacing>,
    render: &mut HashMap<Entity, Stencil>,
) {
    for (_bar_id, bar) in bars {
        while let Some((duration, entity)) = bar.push_managed_entity(entities) {
            rnc.insert(entity, RestNoteChord::new(duration, false));
            spacing.insert(entity, RelativeRhythmicSpacing::default());
            render.insert(entity, Stencil::default());
        }

        while let Some(entity) = bar.pop_managed_entity() {
            rnc.remove(&entity);
            spacing.remove(&entity);
            render.remove(&entity);
        }
    }
}
