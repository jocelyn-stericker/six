use std::collections::HashMap;

use crate::{components::Chord, components::Context, PitchKind};
use rhythm::{components::Bar, components::Spacing, BarChild};
use specs::{Entities, Entity};
use stencil::components::Stencil;

pub fn sys_update_chord_timing(
    entities: &Entities,
    chord: &mut HashMap<Entity, Chord>,
    contexts: &mut HashMap<Entity, Context>,
    bars: &mut HashMap<Entity, Bar>,
    spacing: &mut HashMap<Entity, Spacing>,
    parents: &mut HashMap<Entity, Entity>,
    render: &mut HashMap<Entity, Stencil>,
) {
    for (bar_id, bar) in bars {
        let bar_context = contexts.get(bar_id).cloned().unwrap_or_default();
        while let Some((duration, entity)) = bar.push_managed_entity(entities) {
            chord.insert(entity, Chord::new(duration, PitchKind::Rest));
            contexts.insert(entity, Context::default());
            spacing.insert(entity, Spacing::default());
            parents.insert(entity, *bar_id);
            render.insert(entity, Stencil::default());
        }

        while let Some(entity) = bar.pop_managed_entity() {
            chord.remove(&entity);
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
            if let Some(chord) = chord.get_mut(&stencil) {
                chord.duration = duration;
                if lifetime.is_automatic() {
                    chord.natural_duration = duration;
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
