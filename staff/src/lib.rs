mod sys_print_staff;
mod sys_print_staff_lines;

use entity::Entity;
pub use sys_print_staff::sys_print_staff;
pub use sys_print_staff_lines::sys_print_staff_lines;

#[derive(Debug)]
pub struct Staff {
    pub width: f64,
    pub staff_lines: Option<Entity>,
    pub bars: Vec<Entity>,
}

impl Staff {
    pub fn new() -> Staff {
        Staff {
            width: 0.0,
            staff_lines: None,
            bars: Vec::default(),
        }
    }
}
