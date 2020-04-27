use entity::{EntitiesRes, Entity};
use kurbo::{Line, Point};
use num_rational::Rational;
use rhythm::{Bar, Duration, RelativeRhythmicSpacing, RhythmicBeaming};
use std::collections::BTreeSet;
use std::collections::HashMap;
use stencil::Stencil;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BeamAttachement {
    stem_start: Point,
    extreme_y: f64,
    entering: u8,
    leaving: u8,
}

#[derive(Debug)]
pub struct Beam(Vec<BeamAttachement>);

impl Beam {
    pub fn new(
        beaming: &[Option<RhythmicBeaming>],
        spacing: &[RelativeRhythmicSpacing],
        attachments: &[Point],
    ) -> Beam {
        Beam(
            beaming
                .iter()
                .zip(spacing)
                .zip(attachments)
                .map(|((beam, spacing), attachment)| BeamAttachement {
                    stem_start: Point::new(spacing.start_x + attachment.x, attachment.y),
                    extreme_y: 1000.0,
                    entering: beam.map(|beam| beam.entering).unwrap_or(0),
                    leaving: beam.map(|beam| beam.leaving).unwrap_or(0),
                })
                .collect(),
        )
    }

    pub fn draft(beaming: &[Option<RhythmicBeaming>]) -> Beam {
        Beam(
            beaming
                .iter()
                .map(|beam| BeamAttachement {
                    stem_start: Point::default(),
                    extreme_y: 0.0,
                    entering: beam.map(|beam| beam.entering).unwrap_or(0),
                    leaving: beam.map(|beam| beam.leaving).unwrap_or(0),
                })
                .collect(),
        )
    }

    pub fn print(&self) -> Stencil {
        let mut stencil = Stencil::default();

        let mut level = 0;
        for (i, attachment) in self.0.iter().enumerate() {
            // Backwards fractional.
            for l in level..attachment.entering {
                let next_level = self.0.get(i + 1).map(|l| l.entering).unwrap_or(0);
                if next_level <= l {
                    let start_x = attachment.stem_start.x - 295.0;
                    let start_y = attachment.extreme_y;
                    stencil = stencil.and(Stencil::beam(
                        Line::new(
                            Point::new(start_x, start_y + 187.5 * (l as f64)),
                            Point::new(
                                attachment.stem_start.x,
                                attachment.extreme_y + 187.5 * (l as f64),
                            ),
                        ),
                        level as isize,
                    ));
                }
            }

            // Whole or forwards fractional.
            for l in level..attachment.leaving {
                let mut end_x = attachment.stem_start.x;
                let mut end_y = attachment.extreme_y;
                let mut fractional = true;
                for maybe_end in self.0.iter().skip(i + 1) {
                    if maybe_end.entering <= l {
                        if fractional {
                            end_x += 295.0;
                        }

                        break;
                    }
                    fractional = false;
                    end_x = maybe_end.stem_start.x;
                    end_y = maybe_end.extreme_y;
                }
                stencil = stencil.and(Stencil::beam(
                    Line::new(
                        Point::new(
                            attachment.stem_start.x,
                            attachment.extreme_y + 187.5 * (l as f64),
                        ),
                        Point::new(end_x, end_y + 187.5 * (l as f64)),
                    ),
                    level as isize,
                ));
            }
            level = attachment.leaving;
            stencil = stencil.and(Stencil::stem_line(
                attachment.stem_start.x,
                attachment.stem_start.y,
                attachment.extreme_y,
            ));
        }

        stencil
    }
}

pub fn sys_draft_beaming(
    entities: &EntitiesRes,
    bars: &HashMap<Entity, Bar>,
    parents: &mut HashMap<Entity, Entity>,
    beam_for_rnc: &mut HashMap<Entity, Entity>,
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
        for (duration, start, entity, is_automatic) in bar.children() {
            if !is_automatic
                && duration
                    .duration_display_base()
                    .map(|b| b.beam_count())
                    .unwrap_or(0)
                    > 0
            {
                if let Some(current_candidate) = &mut current_candidate {
                    current_candidate.1.push((duration, entity));
                } else {
                    current_candidate = Some((start, vec![(duration, entity)]));
                }
            } else if let Some(current_candidate) = current_candidate.take() {
                candidates.push(current_candidate);
            }
            if let Some(beam) = beam_for_rnc.get(&entity) {
                available_beam_entities.insert(*beam);
            }
        }
        if let Some(current_candidate) = current_candidate.take() {
            candidates.push(current_candidate);
        }
        for (t0, durations) in candidates {
            let mut beam_entity = None;
            let mut beam_attachements = Vec::new();

            for (beaming, (_duration, rnc_entity)) in bar
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
                    beam_for_rnc.insert(rnc_entity, this_beam_entity);
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
                    beam_for_rnc.remove(&rnc_entity);
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

pub fn sys_space_beams(
    bars: &HashMap<Entity, Bar>,
    spacing: &HashMap<Entity, RelativeRhythmicSpacing>,
    beam_for_rnc: &HashMap<Entity, Entity>,
    attachments: &HashMap<Entity, Option<Point>>,
    beams: &mut HashMap<Entity, Beam>,
) {
    for bar in bars.values() {
        let mut prev_beam = None;
        let mut idx_in_beam = 0;

        for (_, _, rnc, _) in bar.children() {
            if let (Some(beam_id), Some(ref mut beam), Some(spacing), Some(Some(attachment))) = (
                beam_for_rnc.get(&rnc).copied(),
                beam_for_rnc
                    .get(&rnc)
                    .and_then(|beam_id| beams.get_mut(beam_id)),
                spacing.get(&rnc),
                attachments.get(&rnc),
            ) {
                if Some(beam_id) != prev_beam {
                    idx_in_beam = 0;
                }
                if let Some(beam_attachment) = beam.0.get_mut(idx_in_beam) {
                    beam_attachment.stem_start =
                        Point::new(spacing.start_x + attachment.x, attachment.y);
                    beam_attachment.extreme_y = -1000.0;
                }
                idx_in_beam += 1;
                prev_beam = Some(beam_id);
            }
        }
    }
}

pub fn sys_print_beams(beams: &HashMap<Entity, Beam>, stencils: &mut HashMap<Entity, Stencil>) {
    for (beam_id, beam) in beams {
        stencils.insert(*beam_id, beam.print());
    }
}
