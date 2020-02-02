use std::collections::HashMap;

use crate::Staff;
use entity::{Entity, Join};
use rhythm::{Bar, Start};

/// Adds bar numbers to children of Staffs (BetweenBars and Bars).
pub fn sys_update_bar_numbers(
    staffs: &HashMap<Entity, Staff>,
    ordered_children: &HashMap<Entity, Vec<Entity>>,
    bars: &HashMap<Entity, Bar>,
    starts: &mut HashMap<Entity, Start>,
) {
    for (_staff_entity, (_staff, children)) in (staffs, ordered_children).join() {
        let mut idx = 0;
        for child in children {
            if let Some(start) = starts.get_mut(child) {
                start.bar = idx;
            }
            if bars.contains_key(child) {
                idx += 1;
            }
        }
    }
}
