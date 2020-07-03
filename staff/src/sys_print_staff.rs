use std::collections::{BTreeSet, HashMap};

use crate::sys_break_into_lines::STAFF_MARGIN;
use crate::LineOfStaff;
use entity::{Entity, Join};
use kurbo::{Rect, Vec2};
use rhythm::{Bar, BarChild, RelativeRhythmicSpacing};
use stencil::{Stencil, StencilMap};

pub fn sys_print_staff(
    line_of_staffs: &mut HashMap<Entity, LineOfStaff>,
    bars: &HashMap<Entity, Bar>,
    beam_for_rnc: &HashMap<Entity, Entity>,
    spacing: &HashMap<Entity, RelativeRhythmicSpacing>,
    stencils: &HashMap<Entity, Stencil>,
    children: &HashMap<Entity, Vec<Entity>>,
    stencil_maps: &mut HashMap<Entity, StencilMap>,
) {
    for (staff_entity, (staff, staff_children)) in (line_of_staffs, children).join() {
        let mut staff_advance = STAFF_MARGIN;
        let mut staff_stencil = StencilMap::default();

        // Lines are behind contents.
        staff_stencil = staff_stencil.and(staff.staff_lines, None);

        for child in staff_children {
            if let Some(bar) = bars.get(&child) {
                let mut bar_stencil = StencilMap::default();
                let start = 0f64;
                let mut advance = start;
                let mut beams = BTreeSet::new();
                for BarChild { stencil, .. } in bar.children() {
                    let relative_spacing = spacing[&stencil];

                    bar_stencil =
                        bar_stencil.and(stencil, Some(Vec2::new(relative_spacing.start_x, 0.0)));
                    advance = advance.max(relative_spacing.end_x);
                    if let Some(beam) = beam_for_rnc.get(&stencil) {
                        beams.insert(*beam);
                    }

                    for (rnc_child_id, _) in (children.get(&stencil), stencils).join() {
                        bar_stencil = bar_stencil
                            .and(rnc_child_id, Some(Vec2::new(relative_spacing.start_x, 0.0)));
                    }
                }
                for beam in &beams {
                    bar_stencil = bar_stencil.and(*beam, None);
                }

                bar_stencil.set_explicit_rect(Rect::new(start, -1500f64, advance, 1500f64));

                stencil_maps.insert(*child, bar_stencil);

                staff_stencil = staff_stencil.and(*child, Some(Vec2::new(staff_advance, 0.0)));
                staff_advance += advance;
            } else if let Some(stencil) = stencils.get(&child) {
                staff_stencil = staff_stencil.and(*child, Some(Vec2::new(staff_advance, 0.0)));
                staff_advance += stencil.advance();
            }
        }

        staff.width = staff_advance;

        stencil_maps.insert(staff_entity, staff_stencil);
    }
}
