use std::collections::HashMap;

use crate::RestNoteChord;
use entity::{EntitiesRes, Entity};
use rhythm::{Bar, RelativeRhythmicSpacing, Start};
use stencil::Stencil;

pub fn sys_update_rnc_timing(
    entities: &EntitiesRes,
    rnc: &mut HashMap<Entity, RestNoteChord>,
    starts: &mut HashMap<Entity, Start>,
    bars: &mut HashMap<Entity, Bar>,
    spacing: &mut HashMap<Entity, RelativeRhythmicSpacing>,
    parents: &mut HashMap<Entity, Entity>,
    render: &mut HashMap<Entity, Stencil>,
) {
    for (bar_id, bar) in bars {
        let bar_start = starts.get(bar_id).cloned().unwrap_or_default();
        while let Some((duration, entity)) = bar.push_managed_entity(entities) {
            // TODO: get correct start
            rnc.insert(entity, RestNoteChord::new(duration, false));
            starts.insert(entity, Start::default());
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
                if is_managed {
                    rnc.natural_duration = duration;
                }
            }
            if let Some(start_data) = starts.get_mut(&entity) {
                start_data.bar = bar_start.bar;
                start_data.beat = start;
                if is_managed {
                    start_data.natural_beat = start;
                }
            }
        }
    }
}
