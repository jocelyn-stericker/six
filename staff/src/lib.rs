#![allow(clippy::implicit_hasher, clippy::blacklisted_name)]

mod between_bars;
mod sys_break_into_lines;
mod sys_print_between_bars;
mod sys_print_staff;
mod sys_print_staff_lines;
mod sys_update_context;

pub use between_bars::{Barline, BetweenBars};
use specs::{BTreeStorage, Component, Entity};
pub use sys_break_into_lines::{sys_break_into_lines, BreakIntoLineComponents};
pub use sys_print_between_bars::sys_print_between_bars;
pub use sys_print_staff::sys_print_staff;
pub use sys_print_staff_lines::sys_print_staff_lines;
pub use sys_update_context::sys_update_context;
