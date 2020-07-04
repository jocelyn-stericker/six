use crate::components::Beam;
use kurbo::Point;
use rhythm::{components::Bar, components::Spacing, BarChild};
use specs::Entity;
use std::collections::HashMap;

pub fn sys_space_beams(
    bars: &HashMap<Entity, Bar>,
    spacing: &HashMap<Entity, Spacing>,
    beam_for_rnc: &HashMap<Entity, Entity>,
    attachments: &HashMap<Entity, Option<Point>>,
    beams: &mut HashMap<Entity, Beam>,
) {
    for bar in bars.values() {
        let mut prev_beam = None;
        let mut idx_in_beam = 0;

        for BarChild { stencil, .. } in bar.children() {
            if let (Some(beam_id), Some(ref mut beam), Some(spacing), Some(Some(attachment))) = (
                beam_for_rnc.get(&stencil).copied(),
                beam_for_rnc
                    .get(&stencil)
                    .and_then(|beam_id| beams.get_mut(beam_id)),
                spacing.get(&stencil),
                attachments.get(&stencil),
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
