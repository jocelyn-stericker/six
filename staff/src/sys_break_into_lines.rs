use crate::{LineOfStaff, Staff};
use entity::{EntitiesRes, Entity};
use std::collections::HashMap;

pub fn sys_break_into_lines(
    entities: &EntitiesRes,
    staffs: &mut HashMap<Entity, Staff>,
    parents: &mut HashMap<Entity, Entity>,
    ordered_children: &mut HashMap<Entity, Vec<Entity>>,
    line_of_staffs: &mut HashMap<Entity, LineOfStaff>,
) {
    for (staff_id, staff) in staffs {
        // TODO: implement
        let oc: Vec<Vec<_>> = ordered_children
            .get(staff_id)
            .unwrap()
            .chunks(5)
            .map(|c| c.iter().copied().collect())
            .collect();

        for (line_number, line) in oc.into_iter().enumerate() {
            if staff.lines.len() == line_number {
                // This is a line of Staff.
                let line_of_staff_id = entities.create();
                // This is the 5 staff lines for the line of Staff.
                let staff_lines_id = entities.create();

                parents.insert(staff_lines_id, line_of_staff_id);

                line_of_staffs.insert(line_of_staff_id, LineOfStaff::new(staff_lines_id));

                staff.lines.push(line_of_staff_id);
                parents.insert(line_of_staff_id, *staff_id);
            }

            ordered_children.insert(staff.lines[line_number], line);
        }
    }
}
