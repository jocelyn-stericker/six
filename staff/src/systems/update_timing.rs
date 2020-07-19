#![allow(clippy::type_complexity)]

use crate::{
    components::{Children, Chord, Context, FlagAttachment},
    PitchKind,
};
use rhythm::{components::Bar, components::Spacing, BarChild};
use specs::{world::Builder, Entities, Join, LazyUpdate, Read, System, WriteStorage};
use stencil::components::{Parent, Stencil};

#[derive(Debug, Default)]
pub struct UpdateTiming;

impl<'a> System<'a> for UpdateTiming {
    type SystemData = (
        Entities<'a>,
        Read<'a, LazyUpdate>,
        WriteStorage<'a, Chord>,
        WriteStorage<'a, Context>,
        WriteStorage<'a, Bar>,
    );

    fn run(&mut self, (entities, lazy, mut chords, mut contexts, mut bars): Self::SystemData) {
        for (bar_id, bar) in (&entities, &mut bars).join() {
            let bar_context = contexts.get(bar_id).cloned().unwrap_or_default();
            while let Some((duration, start)) = bar.next_missing_child() {
                bar.push_managed_entity(
                    lazy.create_entity(&entities)
                        .with(Spacing::default())
                        .with(Chord::new(duration, PitchKind::Rest))
                        .with(Children::default())
                        .with(Context {
                            beat: start,
                            natural_beat: start,
                            ..Default::default()
                        })
                        .with(FlagAttachment::default())
                        .with(Stencil::default())
                        .with(Parent(bar_id))
                        .build(),
                );
            }

            while let Some(entity) = bar.pop_managed_entity() {
                entities.delete(entity).unwrap();
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
