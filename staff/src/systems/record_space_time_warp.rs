use crate::{components::SpaceTimeWarp, resources::KeepSpacing};
use rhythm::{components::Bar, components::Spacing, BarChild};
use specs::{Join, ReadStorage, System, WriteStorage, Read};

#[derive(Debug, Default)]
pub struct RecordSpaceTimeWarp;

impl<'a> System<'a> for RecordSpaceTimeWarp {
    type SystemData = (
        Read<'a, KeepSpacing>,
        ReadStorage<'a, Bar>,
        ReadStorage<'a, Spacing>,
        WriteStorage<'a, SpaceTimeWarp>,
    );

    fn run(&mut self, (keep_spacing, bars, rel_spacings, mut warps): Self::SystemData) {
        if keep_spacing.0 {
            return;
        }

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
