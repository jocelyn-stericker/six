use std::collections::HashMap;

use crate::Staff;
use entity::Entity;
use stencil::Stencil;

pub fn sys_print_staff_lines(
    staffs: &HashMap<Entity, Staff>,
    stencils: &mut HashMap<Entity, Stencil>,
) {
    for staff in staffs.values() {
        *stencils.entry(staff.staff_lines).or_default() = Stencil::staff_line(staff.width);
    }
}
