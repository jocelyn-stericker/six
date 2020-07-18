#![allow(clippy::type_complexity)]

use crate::components::{Beam, BeamForChord, FlagAttachment};
use kurbo::Point;
use rhythm::{components::Bar, components::Spacing, BarChild};
use specs::{Join, ReadStorage, System, WriteStorage};

#[derive(Debug, Default)]
pub struct SpaceBeam;

impl<'a> System<'a> for SpaceBeam {
    type SystemData = (
        ReadStorage<'a, Bar>,
        ReadStorage<'a, Spacing>,
        ReadStorage<'a, BeamForChord>,
        ReadStorage<'a, FlagAttachment>,
        WriteStorage<'a, Beam>,
    );

    fn run(&mut self, (bars, spacings, beam_for_chords, attachments, mut beams): Self::SystemData) {
        for bar in bars.join() {
            let mut prev_beam = None;
            let mut idx_in_beam = 0;

            for BarChild { stencil, .. } in bar.children() {
                if let (
                    Some(BeamForChord(beam_id)),
                    Some(ref mut beam),
                    Some(spacing),
                    Some(FlagAttachment(Some(attachment))),
                ) = (
                    beam_for_chords.get(stencil),
                    beam_for_chords
                        .get(stencil)
                        .and_then(|beam_id| beams.get_mut(beam_id.0)),
                    spacings.get(stencil),
                    attachments.get(stencil),
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
}
