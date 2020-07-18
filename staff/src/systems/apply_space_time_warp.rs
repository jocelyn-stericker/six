use crate::components::SpaceTimeWarp;
use rhythm::{components::Bar, components::Spacing, BarChild};
use specs::{Join, ReadStorage, System, WriteStorage};

#[derive(Debug, Default)]
pub struct ApplySpaceTimeWarp;

impl<'a> System<'a> for ApplySpaceTimeWarp {
    type SystemData = (
        ReadStorage<'a, Bar>,
        ReadStorage<'a, SpaceTimeWarp>,
        WriteStorage<'a, Spacing>,
    );

    fn run(&mut self, (bars, warps, mut spacings): Self::SystemData) {
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
