use std::collections::HashMap;

use crate::Song;
use entity::{Entity, Join};
use kurbo::Vec2;
use stencil::StencilMap;

pub fn sys_print_song(
    songs: &HashMap<Entity, Song>,
    children: &HashMap<Entity, Vec<Entity>>,
    stencil_maps: &mut HashMap<Entity, StencilMap>,
) {
    for (_id, (_song, children, render)) in (songs, children, stencil_maps).join() {
        let mut map = StencilMap::new();
        for child in children {
            map = map.and(*child, None);
        }
        *render = map
            .with_translation(Vec2::new(200.0, -1500.0))
            .with_paper_size(3);
    }
}
