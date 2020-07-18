#![allow(clippy::type_complexity)]

use crate::{components::Chord, components::Context, PitchKind};
use rhythm::{components::Bar, components::Spacing, BarChild};
use specs::{Entities, Join, System, WriteStorage};
use stencil::components::{Parent, Stencil};

#[derive(Debug, Default)]
pub struct UpdateTiming;

impl<'a> System<'a> for UpdateTiming {
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, Chord>,
        WriteStorage<'a, Context>,
        WriteStorage<'a, Bar>,
        WriteStorage<'a, Spacing>,
        WriteStorage<'a, Parent>,
        WriteStorage<'a, Stencil>,
    );

    fn run(
        &mut self,
        (entities, mut chords, mut contexts, mut bars, mut spacings, mut parents, mut stencils): Self::SystemData,
    ) {
        for (bar_id, bar) in (&entities, &mut bars).join() {
            let bar_context = contexts.get(bar_id).cloned().unwrap_or_default();
            while let Some((duration, entity)) = bar.push_managed_entity(&entities) {
                spacings.entry(entity).unwrap().replace(Spacing::default());
                chords
                    .entry(entity)
                    .unwrap()
                    .replace(Chord::new(duration, PitchKind::Rest));
                contexts.entry(entity).unwrap().replace(Context::default());
                parents.entry(entity).unwrap().replace(Parent(bar_id));
                stencils.entry(entity).unwrap().replace(Stencil::default());
            }

            while let Some(entity) = bar.pop_managed_entity() {
                chords.remove(entity);
                spacings.remove(entity);
                parents.remove(entity);
                stencils.remove(entity);
            }

            for BarChild {
                duration,
                start,
                stencil,
                lifetime,
            } in bar.children()
            {
                if let Some(chord) = chords.get_mut(stencil) {
                    chord.duration = duration;
                    if lifetime.is_automatic() {
                        chord.natural_duration = duration;
                    }
                }
                if let Some(context_data) = contexts.get_mut(stencil) {
                    context_data.bar = bar_context.bar;
                    context_data.beat = start;
                    if lifetime.is_automatic() {
                        context_data.natural_beat = start;
                    }
                }
            }
        }
    }
}
