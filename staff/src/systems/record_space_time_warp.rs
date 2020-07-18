use crate::components::SpaceTimeWarp;
use rhythm::{components::Bar, components::Spacing, BarChild};
use specs::{Join, ReadStorage, System, WriteStorage};

#[derive(Debug, Default)]
pub struct RecordSpaceTimeWarp;

impl<'a> System<'a> for RecordSpaceTimeWarp {
    type SystemData = (
        ReadStorage<'a, Bar>,
        ReadStorage<'a, Spacing>,
        WriteStorage<'a, SpaceTimeWarp>,
    );

    fn run(&mut self, (bars, rel_spacings, mut warps): Self::SystemData) {
        for (bar, warp_entry) in (&bars, warps.entries()).join() {
            let mut warp = Vec::new();
            let mut max_x = 0.0;
            for BarChild { start, stencil, .. } in bar.children() {
                if let Some(spacing) = rel_spacings.get(stencil) {
                    warp.push((start, spacing.start_x));
                    max_x = spacing.end_x.max(max_x);
                }
            }

            warp.push((bar.metre().duration(), max_x));
            warp_entry.replace(SpaceTimeWarp(warp));
        }
    }
}
