use crate::components::Beam;
use kurbo::{Line, Point};
use specs::Entity;
use std::collections::HashMap;
use stencil::components::Stencil;

pub fn sys_print_beams(beams: &HashMap<Entity, Beam>, stencils: &mut HashMap<Entity, Stencil>) {
    for (beam_id, beam) in beams {
        let mut stencil = Stencil::default();

        let mut level = 0;
        for (i, attachment) in beam.0.iter().enumerate() {
            // Backwards fractional.
            for l in level..attachment.entering {
                let next_level = beam.0.get(i + 1).map(|l| l.entering).unwrap_or(0);
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
                for maybe_end in beam.0.iter().skip(i + 1) {
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

        stencils.insert(*beam_id, stencil);
    }
}
