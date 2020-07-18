#![allow(clippy::type_complexity)]

use std::collections::BTreeSet;

use crate::components::{Children, LineOfStaff};
use crate::systems::break_into_lines::STAFF_MARGIN;
use chord::components::BeamForChord;
use kurbo::{Rect, Vec2};
use rhythm::{components::Bar, components::Spacing, BarChild};
use specs::{Entities, Join, ReadStorage, System, WriteStorage};
use stencil::components::{Stencil, StencilMap};

#[derive(Debug, Default)]
pub struct PrintStaff;

impl<'a> System<'a> for PrintStaff {
    type SystemData = (
        Entities<'a>,
        ReadStorage<'a, Bar>,
        ReadStorage<'a, BeamForChord>,
        ReadStorage<'a, Spacing>,
        ReadStorage<'a, Stencil>,
        ReadStorage<'a, Children>,
        WriteStorage<'a, LineOfStaff>,
        WriteStorage<'a, StencilMap>,
    );

    fn run(
        &mut self,
        (
            entities,
            bars,
            beam_for_chords,
            spacings,
            stencils,
            children,
            mut line_of_staffs,
            mut stencil_maps,
        ): Self::SystemData,
    ) {
        for (entity, line_of_staff, staff_children) in
            (&entities, &mut line_of_staffs, &children).join()
        {
            let mut staff_advance = STAFF_MARGIN;
            let mut staff_stencil = StencilMap::default();

            // Lines are behind contents.
            staff_stencil = staff_stencil.and(line_of_staff.staff_lines, None);

            for &child in &staff_children.0 {
                if let Some(bar) = bars.get(child) {
                    let mut bar_stencil = StencilMap::default();
                    let start = 0f64;
                    let mut advance = start;
                    let mut beams = BTreeSet::new();
                    for BarChild { stencil, .. } in bar.children() {
                        let relative_spacing = spacings.get(stencil).unwrap();

                        bar_stencil = bar_stencil
                            .and(stencil, Some(Vec2::new(relative_spacing.start_x, 0.0)));
                        advance = advance.max(relative_spacing.end_x);
                        if let Some(beam) = beam_for_chords.get(stencil) {
                            beams.insert(beam.0);
                        }

                        if let Some(Children(children)) = children.get(stencil) {
                            for &child in children {
                                bar_stencil = bar_stencil
                                    .and(child, Some(Vec2::new(relative_spacing.start_x, 0.0)));
                            }
                        }
                    }
                    for beam in &beams {
                        bar_stencil = bar_stencil.and(*beam, None);
                    }

                    bar_stencil.set_explicit_rect(Rect::new(start, -1500f64, advance, 1500f64));

                    stencil_maps.entry(child).unwrap().replace(bar_stencil);

                    staff_stencil = staff_stencil.and(child, Some(Vec2::new(staff_advance, 0.0)));
                    staff_advance += advance;
                } else if let Some(stencil) = stencils.get(child) {
                    staff_stencil = staff_stencil.and(child, Some(Vec2::new(staff_advance, 0.0)));
                    staff_advance += stencil.advance();
                }
            }

            line_of_staff.width = staff_advance;

            stencil_maps.entry(entity).unwrap().replace(staff_stencil);
        }
    }
}
