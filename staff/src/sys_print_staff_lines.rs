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
        // TODO: coordinate advancew with sys_print_staff.
        *stencils.entry(staff.staff_lines).or_default() =
            Stencil::staff_line(staff.width - STAFF_MARGIN)
                .with_translation(Vec2::new(STAFF_MARGIN, 0f64));
    }
}
