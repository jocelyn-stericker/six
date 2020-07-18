use crate::components::Cursor;
use kurbo::{Line, Point};
use specs::{Join, ReadStorage, System, WriteStorage};
use stencil::components::Stencil;

#[derive(Debug, Default)]
pub struct PrintCursor;

impl<'a> System<'a> for PrintCursor {
    type SystemData = (ReadStorage<'a, Cursor>, WriteStorage<'a, Stencil>);

    fn run(&mut self, (cursors, mut stencils): Self::SystemData) {
        for (_cursor, stencil) in (&cursors, &mut stencils).join() {
            *stencil = Stencil::line(
                Line::new(Point::new(-100.0, -1100.0), Point::new(-100.0, 1100.0)),
                40.0,
            );
        }
    }
}
