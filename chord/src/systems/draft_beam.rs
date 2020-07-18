use crate::{components::Beam, components::BeamForChord, BeamAttachment};
use kurbo::Point;
use num_rational::Rational;
use rhythm::{components::Bar, BarChild, Duration};
use specs::{Entities, Entity, Join, ReadStorage, System, WriteStorage};
use std::collections::BTreeSet;
use stencil::components::Parent;

#[derive(Debug, Default)]
pub struct DraftBeam;

impl<'a> System<'a> for DraftBeam {
    type SystemData = (
        Entities<'a>,
        ReadStorage<'a, Bar>,
        WriteStorage<'a, Parent>,
        WriteStorage<'a, BeamForChord>,
        WriteStorage<'a, Beam>,
    );

    fn run(
        &mut self,
        (entities, bars, mut parents, mut beam_for_chords, mut beams): Self::SystemData,
    ) {
        let mut beams_to_delete: BTreeSet<Entity> =
            (&entities, &beams).join().map(|(ent, _)| ent).collect();

        for (bar_id, bar) in (&entities, &bars).join() {
            if bar.whole_rest() {
                continue;
            }
            let mut candidates = vec![];
            let mut current_candidate: Option<(Rational, Vec<(Duration, Entity)>)> = None;
            // We reuse these if we have more beams, and delete them otherwise.
            let mut available_beam_entities = BTreeSet::new();
            for BarChild {
                duration,
                start,
                lifetime,
                stencil,
            } in bar.children()
            {
                if !lifetime.is_temporary()
                    && !lifetime.is_automatic()
                    && duration
                        .duration_display_base()
                        .map(|b| b.beam_count())
                        .unwrap_or(0)
                        > 0
                {
                    if let Some(current_candidate) = &mut current_candidate {
                        current_candidate.1.push((duration, stencil));
                    } else {
                        current_candidate = Some((start, vec![(duration, stencil)]));
                    }
                } else if let Some(current_candidate) = current_candidate.take() {
                    candidates.push(current_candidate);
                }
                if let Some(beam) = beam_for_chords.get(stencil) {
                    available_beam_entities.insert(beam.0);
                }
            }
            if let Some(current_candidate) = current_candidate.take() {
                candidates.push(current_candidate);
            }
            for (t0, durations) in candidates {
                let mut beam_entity = None;
                let mut beam_attachments = Vec::new();

                for (beaming, (_duration, chord_entity)) in bar
                    .beaming(t0, durations.iter().map(|(d, _e)| *d).collect())
                    .iter()
                    .zip(durations)
                {
                    if let Some(beaming) = beaming {
                        let next_available_beam = available_beam_entities.iter().next().copied();
                        let this_beam_entity = beam_entity
                            .or_else(|| {
                                next_available_beam.and_then(|f| available_beam_entities.take(&f))
                            })
                            .unwrap_or_else(|| entities.create());
                        beam_for_chords
                            .entry(chord_entity)
                            .unwrap()
                            .replace(BeamForChord(this_beam_entity));
                        beams_to_delete.remove(&this_beam_entity);
                        beam_entity = Some(this_beam_entity);
                        beam_attachments.push(BeamAttachment {
                            stem_start: Point::default(),
                            extreme_y: 0.0,
                            entering: beaming.entering,
                            leaving: beaming.leaving,
                        });
                    } else {
                        if let Some(beam_entity) = beam_entity.take() {
                            parents.entry(beam_entity).unwrap().replace(Parent(bar_id));
                            beams
                                .entry(beam_entity)
                                .unwrap()
                                .replace(Beam(std::mem::take(&mut beam_attachments)));
                        }
                        beam_for_chords.remove(chord_entity);
                    }
                }

                if let Some(beam_entity) = beam_entity.take() {
                    parents.entry(beam_entity).unwrap().replace(Parent(bar_id));
                    beams
                        .entry(beam_entity)
                        .unwrap()
                        .replace(Beam(std::mem::take(&mut beam_attachments)));
                }
            }
        }

        for beam in beams_to_delete {
            parents.remove(beam);
            beams.remove(beam);
        }
    }
}
