#![allow(clippy::implicit_hasher, clippy::blacklisted_name)]

mod between_bars;
mod sys_print_between_bars;
mod sys_print_staff;
mod sys_print_staff_lines;
mod sys_update_bar_numbers;

pub use between_bars::{Barline, BetweenBars};
use entity::Entity;
pub use sys_print_between_bars::sys_print_between_bars;
pub use sys_print_staff::sys_print_staff;
pub use sys_print_staff_lines::sys_print_staff_lines;
pub use sys_update_bar_numbers::sys_update_bar_numbers;

#[derive(Debug)]
pub struct Staff {
    pub width: f64,
    pub staff_lines: Entity,
}

impl Staff {
    pub fn new(staff_lines: Entity) -> Staff {
        Staff {
            width: 0.0,
            staff_lines,
        }
    }
}
