use crate::components::SpaceTimeWarp;
use rhythm::{components::Bar, components::Spacing, BarChild};
use specs::Entity;
use std::collections::HashMap;

pub fn sys_apply_warp(
    bars: &HashMap<Entity, Bar>,
    rel_spacings: &mut HashMap<Entity, Spacing>,
    warps: &HashMap<Entity, SpaceTimeWarp>,
) {
    for (_bar_id, (bar, warp)) in (bars, warps).join() {
        for BarChild {
            duration,
            start,
            stencil,
            ..
        } in bar.children()
        {
            if let Some(rel_spacing) = rel_spacings.get_mut(&stencil) {
                rel_spacing.start_x = warp.t_to_x(start);
                rel_spacing.end_x = warp.t_to_x(start + duration.duration());
            }
        }
    }
}
