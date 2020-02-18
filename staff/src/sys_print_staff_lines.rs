use kurbo::Vec2;
use std::collections::HashMap;

use crate::sys_break_into_lines::STAFF_MARGIN;
use crate::LineOfStaff;
use entity::Entity;
use stencil::Stencil;

pub fn sys_print_staff_lines(
    staffs: &HashMap<Entity, LineOfStaff>,
    stencils: &mut HashMap<Entity, Stencil>,
) {
    for staff in staffs.values() {
        // TODO: coordinate advance with sys_print_staff.
        let mut stencil = Stencil::default();

        for i in -2..=2 {
            stencil = stencil.and(
                Stencil::staff_line(staff.width - STAFF_MARGIN)
                    .with_translation(Vec2::new(STAFF_MARGIN, (i * 250) as f64)),
            );
        }

        *stencils.entry(staff.staff_lines).or_default() = stencil;
    }
}
