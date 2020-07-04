use crate::{components::Beam, BeamAttachement};
use kurbo::Point;
use num_rational::Rational;
use rhythm::{components::Bar, BarChild, Duration};
use specs::{Entities, Entity};
use std::collections::{BTreeSet, HashMap};

pub fn sys_draft_beaming(
    entities: &Entities,
    bars: &HashMap<Entity, Bar>,
    parents: &mut HashMap<Entity, Entity>,
    beam_for_chord: &mut HashMap<Entity, Entity>,
    beams: &mut HashMap<Entity, Beam>,
) {
    let mut beams_to_delete: BTreeSet<Entity> = beams.keys().copied().collect();
    for (bar_id, bar) in bars {
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
            if let Some(beam) = beam_for_chord.get(&stencil) {
                available_beam_entities.insert(*beam);
            }
        }
        if let Some(current_candidate) = current_candidate.take() {
            candidates.push(current_candidate);
        }
        for (t0, durations) in candidates {
            let mut beam_entity = None;
            let mut beam_attachements = Vec::new();

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
                    beam_for_chord.insert(chord_entity, this_beam_entity);
                    beams_to_delete.remove(&this_beam_entity);
                    beam_entity = Some(this_beam_entity);
                    beam_attachements.push(BeamAttachement {
                        stem_start: Point::default(),
                        extreme_y: 0.0,
                        entering: beaming.entering,
                        leaving: beaming.leaving,
                    });
                } else {
                    if let Some(beam_entity) = beam_entity.take() {
                        parents.insert(beam_entity, *bar_id);
                        beams.insert(beam_entity, Beam(std::mem::take(&mut beam_attachements)));
                    }
                    beam_for_chord.remove(&chord_entity);
                }
            }

            if let Some(beam_entity) = beam_entity.take() {
                parents.insert(beam_entity, *bar_id);
                beams.insert(beam_entity, Beam(std::mem::take(&mut beam_attachements)));
            }
        }
    }

    for beam in beams_to_delete {
        parents.remove(&beam);
        beams.remove(&beam);
    }
}
