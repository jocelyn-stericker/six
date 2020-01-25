use std::collections::HashMap;

use crate::Staff;
use entity::{Entity, Join};
use rhythm::{Bar, Start};

pub fn sys_update_starts(
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
