use kurbo::{Rect, TranslateScale};
use specs::{
    shred::{Fetch, FetchMut},
    storage::MaskedStorage,
    Entities, Entity, Join, ReadStorage, Storage, System, WriteStorage,
};
use staff::components::Song;
use stencil::components::{Stencil, StencilMap, WorldBbox};

#[derive(Debug, Default)]
pub struct UpdateWorldBbox;

impl<'a> System<'a> for UpdateWorldBbox {
    type SystemData = (
        Entities<'a>,
        ReadStorage<'a, Song>,
        ReadStorage<'a, Stencil>,
        ReadStorage<'a, StencilMap>,
        WriteStorage<'a, WorldBbox>,
    );

    fn run(&mut self, (entities, songs, stencils, stencil_maps, mut world_bbox): Self::SystemData) {
        world_bbox.clear();

        for (song_entity, _) in (&entities, &songs).join() {
            update_world_bbox(
                song_entity,
                &stencils,
                &stencil_maps,
                &mut world_bbox,
                TranslateScale::default(),
            );
        }
    }
}

fn update_world_bbox(
    entity: Entity,
    stencils: &Storage<Stencil, Fetch<MaskedStorage<Stencil>>>,
    stencil_maps: &Storage<StencilMap, Fetch<MaskedStorage<StencilMap>>>,
    world_bbox: &mut Storage<WorldBbox, FetchMut<MaskedStorage<WorldBbox>>>,
    transform: TranslateScale,
) -> Rect {
    let rect = if let Some(stencil) = stencils.get(entity) {
        transform * stencil.rect()
    } else if let Some(stencil_map) = stencil_maps.get(entity) {
        let mut rect: Option<Rect> = None;
        for (subentity, subtransform) in stencil_map.get_sorted_children() {
            let child_bbox = update_world_bbox(
                subentity,
                stencils,
                stencil_maps,
                world_bbox,
                transform
                    * TranslateScale::translate(stencil_map.translate().unwrap_or_default())
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

    world_bbox.entry(entity).unwrap().replace(WorldBbox(rect));

    rect
}
