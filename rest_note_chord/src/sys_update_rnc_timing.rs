use std::collections::HashMap;

use crate::context::Context;
use crate::{PitchKind, RestNoteChord};
use entity::{EntitiesRes, Entity};
use rhythm::{Bar, BarChild, RelativeRhythmicSpacing};
use stencil::Stencil;

pub fn sys_update_rnc_timing(
    entities: &EntitiesRes,
    rnc: &mut HashMap<Entity, RestNoteChord>,
    contexts: &mut HashMap<Entity, Context>,
    bars: &mut HashMap<Entity, Bar>,
    spacing: &mut HashMap<Entity, RelativeRhythmicSpacing>,
    parents: &mut HashMap<Entity, Entity>,
    render: &mut HashMap<Entity, Stencil>,
) {
    for (bar_id, bar) in bars {
        let bar_context = contexts.get(bar_id).cloned().unwrap_or_default();
        while let Some((duration, entity)) = bar.push_managed_entity(entities) {
            rnc.insert(entity, RestNoteChord::new(duration, PitchKind::Rest));
            contexts.insert(entity, Context::default());
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

        for BarChild {
            duration,
            start,
            stencil,
            lifetime,
        } in bar.children()
        {
            if let Some(rnc) = rnc.get_mut(&stencil) {
                rnc.duration = duration;
                if lifetime.is_automatic() {
                    rnc.natural_duration = duration;
                }
            }
            if let Some(context_data) = contexts.get_mut(&stencil) {
                context_data.bar = bar_context.bar;
                context_data.beat = start;
                if lifetime.is_automatic() {
                    context_data.natural_beat = start;
                }
            }
        }
    }
}
