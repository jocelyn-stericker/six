use std::collections::HashMap;

use crate::Staff;
use entity::Entity;
use stencil::Stencil;

pub fn sys_print_staff_lines(
    staffs: &HashMap<Entity, Staff>,
    render: &mut HashMap<Entity, Stencil>,
) {
    for (_id, staff) in staffs {
        let staff_lines = staff.staff_lines.unwrap();
        *render.entry(staff_lines).or_default() = Stencil::staff_line(staff.width);
    }
}
