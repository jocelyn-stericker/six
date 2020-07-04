use std::collections::HashMap;

use crate::components::SpaceTimeWarp;
use rhythm::{components::Bar, components::Spacing, BarChild};
use specs::Entity;

pub fn sys_record_space_time_warp(
    bars: &HashMap<Entity, Bar>,
    rel_spacings: &HashMap<Entity, Spacing>,
    warps: &mut HashMap<Entity, SpaceTimeWarp>,
) {
    for (bar_id, bar) in bars {
        let mut warp = Vec::new();
        let mut max_x = 0.0;
        for BarChild { start, stencil, .. } in bar.children() {
            if let Some(spacing) = rel_spacings.get(&stencil) {
                warp.push((start, spacing.start_x));
                max_x = spacing.end_x.max(max_x);
            }
        }

        warp.push((bar.metre().duration(), max_x));
        warps.insert(*bar_id, SpaceTimeWarp(warp));
    }
}
