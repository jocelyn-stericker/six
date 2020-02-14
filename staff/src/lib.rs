#![allow(clippy::implicit_hasher, clippy::blacklisted_name)]

mod between_bars;
mod sys_break_into_lines;
mod sys_print_between_bars;
mod sys_print_staff;
mod sys_print_staff_lines;
mod sys_update_bar_numbers;

pub use between_bars::{Barline, BetweenBars};
use entity::Entity;
pub use sys_break_into_lines::sys_break_into_lines;
pub use sys_print_between_bars::sys_print_between_bars;
pub use sys_print_staff::sys_print_staff;
pub use sys_print_staff_lines::sys_print_staff_lines;
pub use sys_update_bar_numbers::sys_update_bar_numbers;

#[derive(Debug)]
pub struct LineOfStaff {
    pub width: f64,
    pub staff_lines: Entity,
}

#[derive(Debug, Default)]
pub struct Staff {
    /// This is a line of a staff, not the 5 staff lines.
    pub lines: Vec<Entity>,
}

impl LineOfStaff {
    pub fn new(staff_lines: Entity) -> LineOfStaff {
        LineOfStaff {
            width: 0.0,
            staff_lines,
        }
    }
}
