use std::collections::HashMap;

use crate::Song;
use entity::{EntitiesRes, Entity};
use kurbo::Vec2;
use stencil::Stencil;

pub fn sys_print_meta(
    entities: &EntitiesRes,
    parents: &mut HashMap<Entity, Entity>,
    songs: &mut HashMap<Entity, Song>,
    stencils: &mut HashMap<Entity, Stencil>,
) {
    for (song_id, song) in songs.iter_mut() {
        if song.title_stencil.is_none() {
            let id = entities.create();
            song.title_stencil = Some(id);
            parents.insert(id, *song_id);
        }

        // TODO: rastral size.
        let title_x = (song.width / 2f64 - song.title_width / 2f64) * 1000f64 / 7f64;
        stencils.insert(
            song.title_stencil.unwrap(),
            // TODO: sync with reconciler.ts.
            Stencil::text(
                &song.title,
                7f64 * 1000f64 / 7f64,
                song.title_width * 1000f64 / 7f64,
            )
            .with_translation_and_flip(Vec2::new(title_x, -2500f64)),
        );

        if song.author_stencil.is_none() {
            let id = entities.create();
            song.author_stencil = Some(id);
            parents.insert(id, *song_id);
        }

        // TODO: rastral size.
        // TODO: margin size
        let author_x = (song.width - song.author_width) * 1000f64 / 7f64 - 2500f64;
        stencils.insert(
            song.author_stencil.unwrap(),
            // TODO: sync with reconciler.ts.
            Stencil::text(
                &song.author,
                5f64 * 1000f64 / 7f64,
                song.author_width * 1000f64 / 7f64,
            )
            .with_translation_and_flip(Vec2::new(author_x, -3500f64)),
        );
    }
}
