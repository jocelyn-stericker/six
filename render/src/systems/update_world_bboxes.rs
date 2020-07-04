use std::collections::HashMap;

use crate::components::stencil::Stencil;
use crate::components::stencil_map::StencilMap;
use crate::components::world_bbox::WorldBbox;
use kurbo::{Rect, TranslateScale};
use specs::{Entities, Entity, ReadStorage, System, WriteStorage};

pub struct UpdateWorldBbox;

impl<'a> System<'a> for UpdateWorldBbox {
    type SystemData = (
        ReadStorage<'a, Song>,
        ReadStorage<'a, Stencil>,
        ReadStorage<'a, StencilMap>,
        WriteStorage<'a, WorldBbox>,
    );
}

fn update_world_bbox(
    entity: Entity,
    stencils: &HashMap<Entity, Stencil>,
    stencil_maps: &HashMap<Entity, StencilMap>,
    world_bbox: &mut HashMap<Entity, Rect>,
    transform: TranslateScale,
) -> Rect {
    let rect = if let Some(stencil) = stencils.get(&entity) {
        transform * stencil.rect()
    } else if let Some(stencil_map) = stencil_maps.get(&entity) {
        let mut rect: Option<Rect> = None;
        for (subentity, subtransform) in stencil_map.get_sorted_children() {
            let child_bbox = update_world_bbox(
                subentity,
                stencils,
                stencil_maps,
                world_bbox,
                transform
                    * TranslateScale::translate(stencil_map.translate.unwrap_or_default())
                    * TranslateScale::translate(subtransform.unwrap_or_default()),
            );
            rect = Some(match rect {
                None => child_bbox,
                Some(mut rect) => {
                    if child_bbox.x0 < rect.x0 {
                        rect.x0 = child_bbox.x0;
                    }
                    if child_bbox.y0 < rect.y0 {
                        rect.y0 = child_bbox.y0;
                    }
                    if child_bbox.x1 > rect.x1 {
                        rect.x1 = child_bbox.x1;
                    }
                    if child_bbox.y1 > rect.y1 {
                        rect.y1 = child_bbox.y1;
                    }

                    rect
                }
            });
        }
        if let Some(explicit_rect) = stencil_map.explicit_rect() {
            rect = Some(transform * explicit_rect);
        }
        rect.unwrap_or_default()
    } else {
        Rect::default()
    };

    world_bbox.insert(entity, rect);

    rect
}

pub fn sys_update_world_bboxes<T>(
    songs: &HashMap<Entity, T>,
    stencils: &HashMap<Entity, Stencil>,
    stencil_maps: &HashMap<Entity, StencilMap>,
    world_bbox: &mut HashMap<Entity, Rect>,
) {
    world_bbox.clear();

    for song in songs.keys() {
        update_world_bbox(
            *song,
            stencils,
            stencil_maps,
            world_bbox,
            TranslateScale::default(),
        );
    }
}
