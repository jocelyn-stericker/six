use std::collections::HashMap;

use crate::{BetweenBars, Staff};
use entity::{Entity, Join};
use pitch::Clef;
use rest_note_chord::Context;
use rhythm::Bar;

/// Adds bar numbers to children of Staffs (BetweenBars and Bars).
pub fn sys_update_context(
    staffs: &HashMap<Entity, Staff>,
    ordered_children: &HashMap<Entity, Vec<Entity>>,
    bars: &HashMap<Entity, Bar>,
    between_bars: &HashMap<Entity, BetweenBars>,
    contexts: &mut HashMap<Entity, Context>,
) {
    for (_staff_entity, (_staff, children)) in (staffs, ordered_children).join() {
        let mut idx = 0;
        let mut clef = Clef::G;
        let mut key = 0;
        let mut time = (4, 4);

        for child in children {
            if let Some(context) = contexts.get_mut(child) {
                context.bar = idx;
                context.clef = clef;
                context.key = key;
                context.time = time;
            }
            if let Some(bar) = bars.get(child) {
                for (_, _, grandchild, _) in bar.children() {
                    if let Some(context) = contexts.get_mut(&grandchild) {
                        context.bar = idx;
                        context.clef = clef;
                        context.key = key;
                        context.time = time;
                    }
                }
                idx += 1;
            }
            if let Some(between_bar) = between_bars.get(child) {
                if let Some(new_clef) = between_bar.clef {
                    clef = new_clef;
                }
                if let Some(new_key) = between_bar.key {
                    key = new_key;
                }
                if let Some(new_time) = between_bar.time {
                    time = new_time;
                }
            }
        }
    }
}
