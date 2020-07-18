use kurbo::Vec2;

use crate::components::LineOfStaff;
use crate::systems::break_into_lines::STAFF_MARGIN;
use specs::{Join, ReadStorage, System, WriteStorage};
use stencil::components::Stencil;

#[derive(Debug, Default)]
pub struct PrintStaffLines;

impl<'a> System<'a> for PrintStaffLines {
    type SystemData = (ReadStorage<'a, LineOfStaff>, WriteStorage<'a, Stencil>);

    fn run(&mut self, (staffs, mut stencils): Self::SystemData) {
        for staff in staffs.join() {
            // TODO: coordinate advance with sys_print_staff.
            let mut stencil = Stencil::default();

            for i in -2..=2 {
                stencil = stencil.and(
                    Stencil::staff_line(staff.width - STAFF_MARGIN)
                        .with_translation(Vec2::new(STAFF_MARGIN, (i * 250) as f64)),
                );
            }

            stencils.entry(staff.staff_lines).unwrap().replace(stencil);
        }
    }
}
