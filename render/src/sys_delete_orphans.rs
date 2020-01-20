use std::collections::HashMap;
use std::collections::HashSet;

use crate::Song;
use entity::Entity;
use rhythm::{Bar, RelativeRhythmicSpacing};
use staff::{BetweenBars, Staff};
use stencil::{Stencil, StencilMap};

pub fn sys_delete_orphans(
    parents: &HashMap<Entity, Entity>,
    root: Option<Entity>,
    songs: &mut HashMap<Entity, Song>,
    staffs: &mut HashMap<Entity, Staff>,
    bars: &mut HashMap<Entity, Bar>,
    between_bars: &mut HashMap<Entity, BetweenBars>,
    // These are secondary components. They do not necessarily need parents.
    stencils: &mut HashMap<Entity, Stencil>,
    stencil_maps: &mut HashMap<Entity, StencilMap>,
    spacing: &mut HashMap<Entity, RelativeRhythmicSpacing>,
    ordered_children: &mut HashMap<Entity, Vec<Entity>>,
) {
    let mut entities_to_check: HashSet<Entity> = HashSet::new();
    for key in songs.keys() {
        entities_to_check.insert(*key);
    }
    for key in staffs.keys() {
        entities_to_check.insert(*key);
    }
    for key in bars.keys() {
        entities_to_check.insert(*key);
    }
    for key in between_bars.keys() {
        entities_to_check.insert(*key);
    }

    for key in &entities_to_check {
        if !parents.contains_key(key) && Some(*key) != root {
            songs.remove(key);
            staffs.remove(key);
            bars.remove(key);
            between_bars.remove(key);
            stencils.remove(key);
            stencil_maps.remove(key);
            spacing.remove(key);
            ordered_children.remove(key);
        }
    }
}
