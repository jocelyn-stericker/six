use std::collections::HashMap;

use crate::Song;
use entity::{Entity, Join};
use kurbo::{TranslateScale, Vec2};
use staff::Staff;
use stencil::StencilMap;

pub fn sys_print_song(
    songs: &HashMap<Entity, Song>,
    staffs: &HashMap<Entity, Staff>,
    children: &HashMap<Entity, Vec<Entity>>,
    stencil_maps: &mut HashMap<Entity, StencilMap>,
) {
    for (_id, (_song, children, render)) in (songs, children, stencil_maps).join() {
        let mut map = StencilMap::new();
        let mut h = 0.0;
        for child in children {
            if let Some(staff) = staffs.get(child) {
                for line in &staff.lines {
                    map = map.and(
                        *line,
                        if h > 0.0 {
                            Some(TranslateScale::translate(Vec2::new(0.0, -h)))
                        } else {
                            None
                        },
                    );
                    h += 2000.0;
                }
            }
        }
        *render = map
            .with_translation(Vec2::new(200.0, -1500.0))
            .with_paper_size(3);
    }
}
