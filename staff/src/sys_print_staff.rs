use std::collections::HashMap;

use crate::Staff;
use entity::{Entity, Join};
use kurbo::{Rect, TranslateScale, Vec2};
use rhythm::{Bar, RelativeRhythmicSpacing};
use stencil::{Stencil, StencilMap};

pub fn sys_print_staff(
    staffs: &mut HashMap<Entity, Staff>,
    bars: &HashMap<Entity, Bar>,
    spacing: &HashMap<Entity, RelativeRhythmicSpacing>,
    stencils: &HashMap<Entity, Stencil>,
    stencil_maps: &mut HashMap<Entity, StencilMap>,
    children: &HashMap<Entity, Vec<Entity>>,
) {
    for (staff_entity, (staff, children)) in (staffs, children).join() {
        let mut staff_advance = 0.0f64;
        let mut staff_stencil = StencilMap::default();

        for child in children {
            if let Some(bar) = bars.get(&child) {
                let mut bar_stencil = StencilMap::default();
                let start = 200f64;
                let mut advance = start;
                for (_, _, entity, _) in bar.children() {
                    let relative_spacing = spacing[&entity];

                    bar_stencil = bar_stencil.and(
                        entity,
                        Some(TranslateScale::translate(Vec2::new(
                            relative_spacing.start_x,
                            0.0,
                        ))),
                    );
                    advance = advance.max(relative_spacing.end_x);
                }

                bar_stencil.set_explicit_rect(Rect::new(start, -1000f64, advance, 1000f64));

                stencil_maps.insert(*child, bar_stencil);

                staff_stencil = staff_stencil.and(
                    *child,
                    Some(TranslateScale::translate(Vec2::new(staff_advance, 0.0))),
                );
                staff_advance += advance;
            } else if let Some(stencil) = stencils.get(&child) {
                staff_stencil = staff_stencil.and(
                    *child,
                    Some(TranslateScale::translate(Vec2::new(staff_advance, 0.0))),
                );
                staff_advance += stencil.advance();
            }
        }

        staff.width = staff_advance;

        staff_stencil = staff_stencil.and(staff.staff_lines, None);

        stencil_maps.insert(staff_entity, staff_stencil);
    }
}
