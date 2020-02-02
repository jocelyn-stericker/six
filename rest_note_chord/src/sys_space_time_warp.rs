use num_rational::Rational;
use std::collections::HashMap;

use entity::{Entity, Join};
use rhythm::{Bar, RelativeRhythmicSpacing};

/// A map from time to position.
///
/// Used to position items when hovering (see root.freeze_spacing)
#[derive(Debug)]
pub struct SpaceTimeWarp(Vec<(Rational, f64)>);

impl SpaceTimeWarp {
    pub fn t_to_x(&self, t: Rational) -> f64 {
        for (i, (t1, x1)) in self.0.iter().enumerate() {
            if let Some((t2, x2)) = self.0.get(i + 1) {
                if t >= *t1 && t <= *t2 {
                    let pct = (t - t1) / (t2 - t1);
                    let pct = (*pct.numer() as f64) / (*pct.denom() as f64);

                    return x1 + pct * (x2 - x1);
                }
            }
        }

        0.0
    }
}

pub fn sys_record_space_time_warp(
    bars: &HashMap<Entity, Bar>,
    rel_spacings: &HashMap<Entity, RelativeRhythmicSpacing>,
    warps: &mut HashMap<Entity, SpaceTimeWarp>,
) {
    for (bar_id, bar) in bars {
        let mut warp = Vec::new();
        let mut max_x = 0.0;
        for (_, start, entity, _) in bar.children() {
            if let Some(spacing) = rel_spacings.get(&entity) {
                warp.push((start, spacing.start_x));
                max_x = spacing.end_x.max(max_x);
            }
        }

        warp.push((bar.metre().duration(), max_x));
        warps.insert(*bar_id, SpaceTimeWarp(warp));
    }
}

pub fn sys_apply_warp(
    bars: &HashMap<Entity, Bar>,
    rel_spacings: &mut HashMap<Entity, RelativeRhythmicSpacing>,
    warps: &HashMap<Entity, SpaceTimeWarp>,
) {
    for (_bar_id, (bar, warp)) in (bars, warps).join() {
        for (duration, start, entity, _) in bar.children() {
            if let Some(rel_spacing) = rel_spacings.get_mut(&entity) {
                rel_spacing.start_x = warp.t_to_x(start);
                rel_spacing.end_x = warp.t_to_x(start + duration.duration());
            }
        }
    }
}
