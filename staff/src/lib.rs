#![allow(clippy::implicit_hasher, clippy::blacklisted_name)]

mod between_bars;
mod sys_print_between_bars;
mod sys_print_staff;
mod sys_print_staff_lines;

pub use between_bars::{Barline, BetweenBars};
use entity::Entity;
pub use sys_print_between_bars::sys_print_between_bars;
pub use sys_print_staff::sys_print_staff;
pub use sys_print_staff_lines::sys_print_staff_lines;

#[derive(Debug, Default)]
pub struct Staff {
    pub width: f64,
    pub staff_lines: Option<Entity>,
    pub children: Vec<Entity>,
}

impl Staff {
    pub fn new() -> Staff {
        Staff::default()
    }
}
