use kurbo::{Line, Point};
use stencil::Stencil;

use entity::{Entity, Join};
use std::collections::HashMap;

pub fn sys_print_cursors(cursors: &HashMap<Entity, ()>, stencils: &mut HashMap<Entity, Stencil>) {
    for (_id, (_cursor, stencil)) in (cursors, stencils).join() {
        *stencil = Stencil::line(
            Line::new(Point::new(-100.0, -1100.0), Point::new(-100.0, 1100.0)),
            40.0,
        );
    }
}
