use crate::{components::SpaceTimeWarp, resources::KeepSpacing};
use rhythm::{components::Bar, components::Spacing, BarChild};
use specs::{Join, ReadStorage, System, WriteStorage, Read};

#[derive(Debug, Default)]
pub struct ApplySpaceTimeWarp;

impl<'a> System<'a> for ApplySpaceTimeWarp {
    type SystemData = (
        Read<'a, KeepSpacing>,
        ReadStorage<'a, Bar>,
        ReadStorage<'a, SpaceTimeWarp>,
        WriteStorage<'a, Spacing>,
    );

    fn run(&mut self, (keep_spacing, bars, warps, mut spacings): Self::SystemData) {
        if !keep_spacing.0 {
            return;
        }
        for (bar, warp) in (&bars, &warps).join() {
            for BarChild {
                duration,
                start,
                stencil,
                ..
            } in bar.children()
            {
                if let Some(rel_spacing) = spacings.get_mut(stencil) {
                    rel_spacing.start_x = warp.t_to_x(start);
                    rel_spacing.end_x = warp.t_to_x(start + duration.duration());
                }
            }
        }
    }
}
