#![allow(clippy::blacklisted_name)]

mod sys_delete_orphans;
mod sys_print_song;

use entity::{EntitiesRes, Entity, Join};
use num_rational::Rational;
use rest_note_chord::{sys_implicit_rests, sys_print_rnc, sys_relative_spacing, RestNoteChord};
use rhythm::{Bar, Duration, Metre, NoteValue, RelativeRhythmicSpacing};
use staff::{
    sys_print_between_bars, sys_print_staff, sys_print_staff_lines, Barline, BetweenBars, Staff,
};
use std::collections::HashMap;
use stencil::{Stencil, StencilMap};
use wasm_bindgen::prelude::*;

#[derive(Debug, Default)]
pub struct Song {}

#[wasm_bindgen]
#[derive(Debug, Default)]
pub struct Render {
    entities: EntitiesRes,

    root: Option<Entity>,
    songs: HashMap<Entity, Song>,

    staffs: HashMap<Entity, Staff>,
    bars: HashMap<Entity, Bar>,
    between_bars: HashMap<Entity, BetweenBars>,
    rncs: HashMap<Entity, RestNoteChord>,

    stencils: HashMap<Entity, Stencil>,
    stencil_maps: HashMap<Entity, StencilMap>,
    spacing: HashMap<Entity, RelativeRhythmicSpacing>,

    ordered_children: HashMap<Entity, Vec<Entity>>,
    parents: HashMap<Entity, Entity>,
}

/// A DOM-based interface to Six Eight's ECS.
#[wasm_bindgen]
impl Render {
    pub fn new() -> Render {
        Self::default()
    }

    /// Set the Song entity to be rendered.
    pub fn root_set(&mut self, song: usize) {
        if self.root.is_none() {
            let song = Entity::new(song);
            self.root = Some(song);
        }
    }

    /// Clear the Song entity to be rendered.
    pub fn root_clear(&mut self, song: usize) {
        if let Some(root) = self.root {
            let song = Entity::new(song);
            if root == song {
                self.root = None;
            }
        }
    }

    /// Append `child` to the `parent` ordered container.
    ///
    /// If `child` already has a parent, nothing will happen.
    pub fn child_append(&mut self, parent: usize, child: usize) {
        let parent_id = Entity::new(parent);
        let child = Entity::new(child);

        if self.parents.contains_key(&child) {
            return;
        }

        if let Some(ordered_children) = self.ordered_children.get_mut(&parent_id) {
            ordered_children.push(child);
            self.parents.insert(child, parent_id);
        }
    }

    /// Insert `child` to the `parent` ordered container, before `before`.
    ///
    /// If `child` already has a parent, or `before` is not a child of `parent`, nothing will
    /// happen.
    pub fn child_insert_before(&mut self, parent: usize, before: usize, child: usize) {
        let before = Entity::new(before);
        let child = Entity::new(child);
        let parent_id = Entity::new(parent);

        if self.parents.contains_key(&child) {
            return;
        }

        if let Some(ordered_children) = self.ordered_children.get_mut(&parent_id) {
            if let Some(idx) = ordered_children.iter().position(|&x| x == before) {
                ordered_children.insert(idx, child);
                self.parents.insert(child, parent_id);
            }
        }
    }

    /// Remove `child` from the `parent` ordered container.
    ///
    /// If `child` is not a child of `parent`, nothing will happen.
    pub fn child_remove(&mut self, parent: usize, entity: usize) {
        let entity = Entity::new(entity);
        let parent = Entity::new(parent);

        if let Some(ordered_children) = self.ordered_children.get_mut(&parent) {
            if let Some(entity_idx) = ordered_children.iter().position(|&x| x == entity) {
                ordered_children.remove(entity_idx);
                self.parents.remove(&entity);
            }
        }
    }

    /// Create a song, without attaching it as the document root.
    pub fn song_create(&mut self) -> usize {
        let entity = self.entities.create();

        self.songs.insert(entity, Song::default());
        self.ordered_children.insert(entity, vec![]);
        self.stencil_maps.insert(entity, StencilMap::default());

        entity.id()
    }

    /// Create a staff, without attaching it to a song.
    pub fn staff_create(&mut self) -> usize {
        let entity = self.entities.create();

        self.staffs.insert(entity, Staff::default());
        self.stencil_maps.insert(entity, StencilMap::default());
        self.ordered_children.insert(entity, vec![]);

        entity.id()
    }

    /// Create a bar, without attaching it to a staff.
    ///
    /// `numer` and `denom` are the numerator and denominator of the time signature in this bar.
    pub fn bar_create(&mut self, numer: u8, denom: u8) -> usize {
        let entity = self.entities.create();

        self.bars.insert(entity, Bar::new(Metre::new(numer, denom)));
        self.stencil_maps.insert(entity, StencilMap::default());

        entity.id()
    }

    /// Inserts a RestNoteChord into a bar.
    ///
    /// Note that children of bars are not ordered, instead children have a `start` property.
    pub fn bar_insert(&mut self, bar: usize, child: usize) {
        let child = Entity::new(child);
        let bar_id = Entity::new(bar);

        if self.parents.contains_key(&child) {
            return;
        }

        if let Some(bar) = self.bars.get_mut(&bar_id) {
            if let Some(rnc) = self.rncs.get(&child) {
                bar.splice(rnc.start(), vec![(rnc.duration(), Some(child))]);
                self.parents.insert(child, bar_id);
            }
        }
    }

    /// Remove a RestNoteChord from a bar.
    ///
    /// Note that children of bars are not ordered, instead children have a `start` property.
    pub fn bar_remove(&mut self, bar: usize, child: usize) {
        let bar_id = Entity::new(bar);
        let child = Entity::new(child);

        if let Some(bar) = self.bars.get_mut(&bar_id) {
            bar.remove(child);
            self.parents.remove(&child);
        }
    }

    /// Create a RestNoteChord, without attaching it to a bar.
    pub fn rnc_create(
        &mut self,
        note_value: isize,
        dots: u8,
        start_numer: isize,
        start_denom: isize,
        is_note: bool,
    ) -> usize {
        let note_value = NoteValue::new(note_value).unwrap();
        let start = Rational::new(start_numer, start_denom);

        let entity = self.entities.create();

        self.spacing
            .insert(entity, RelativeRhythmicSpacing::default());
        self.rncs.insert(
            entity,
            RestNoteChord::new(Duration::new(note_value, dots, None), is_note, start),
        );
        self.stencils.insert(entity, Stencil::default());

        entity.id()
    }

    pub fn rnc_update_time(
        &mut self,
        rnc: usize,
        note_value: isize,
        dots: u8,
        start_numer: isize,
        start_denom: isize,
    ) {
        let note_value = NoteValue::new(note_value).unwrap();
        let rnc_id = Entity::new(rnc);
        let bars = &mut self.bars;
        if let Some(parent_id) = self.parents.get(&rnc_id) {
            if let (Some(rnc), Some(bar)) = (self.rncs.get_mut(&rnc_id), bars.get_mut(parent_id)) {
                bar.remove(rnc_id);
                rnc.start = Rational::new(start_numer, start_denom);
                rnc.duration = Duration::new(note_value, dots, None);
                rnc.natural_duration = rnc.duration;
                rnc.natural_start = rnc.start;
                bar.splice(rnc.start(), vec![(rnc.duration(), Some(rnc_id))]);
            }

            // Fix previously overlapping notes.
            for (other_rnc_id, (rnc, parent)) in (&mut self.rncs, &self.parents).join() {
                if (rnc.natural_duration != rnc.duration || rnc.natural_start != rnc.start)
                    && parent == parent_id
                {
                    if let Some(bar) = bars.get_mut(parent_id) {
                        bar.remove(other_rnc_id);
                        rnc.duration = rnc.natural_duration;
                        rnc.start = rnc.natural_start;
                        bar.splice(rnc.start(), vec![(rnc.duration(), Some(other_rnc_id))]);
                    }
                }
            }
        }
    }

    /// Insert content that lives before or after a bar, without attaching it to a staff.
    ///
    /// This includes signatures, barlines, clefs, etc.
    pub fn between_bars_create(
        &mut self,
        barline: Option<Barline>,
        clef: bool,
        time_numer: Option<u8>,
        time_denom: Option<u8>,
    ) -> usize {
        let entity = self.entities.create();

        let time = if let (Some(time_numer), Some(time_denom)) = (time_numer, time_denom) {
            Some((time_numer, time_denom))
        } else {
            None
        };

        self.between_bars.insert(
            entity,
            BetweenBars {
                barline,
                clef,
                time,
            },
        );
        self.stencils.insert(entity, Stencil::default());

        entity.id()
    }

    /// Insert content that lives before or after a bar, without attaching it to a staff.
    ///
    /// This includes signatures, barlines, clefs, etc.
    pub fn between_bars_update(
        &mut self,
        entity: usize,
        barline: Option<Barline>,
        clef: bool,
        time_numer: Option<u8>,
        time_denom: Option<u8>,
    ) {
        let entity = Entity::new(entity);

        let time = if let (Some(time_numer), Some(time_denom)) = (time_numer, time_denom) {
            Some((time_numer, time_denom))
        } else {
            None
        };

        self.between_bars.insert(
            entity,
            BetweenBars {
                barline,
                clef,
                time,
            },
        );
    }

    /* Frame */

    pub fn exec(&mut self) {
        sys_delete_orphans::sys_delete_orphans(
            &self.parents,
            self.root,
            &mut self.songs,
            &mut self.staffs,
            &mut self.bars,
            &mut self.between_bars,
            &mut self.rncs,
            &mut self.stencils,
            &mut self.stencil_maps,
            &mut self.spacing,
            &mut self.ordered_children,
        );

        sys_implicit_rests(
            &self.entities,
            &mut self.rncs,
            &mut self.bars,
            &mut self.spacing,
            &mut self.parents,
            &mut self.stencils,
        );
        sys_relative_spacing(&self.rncs, &self.parents, &mut self.spacing);
        sys_print_rnc(&self.rncs, &mut self.stencils);
        sys_print_between_bars(&self.between_bars, &mut self.stencils);

        sys_print_staff(
            &self.entities,
            &mut self.staffs,
            &self.bars,
            &self.spacing,
            &self.stencils,
            &mut self.stencil_maps,
            &self.ordered_children,
        );
        sys_print_staff_lines(&self.staffs, &mut self.stencils);

        sys_print_song::sys_print_song(&self.songs, &self.ordered_children, &mut self.stencil_maps);
    }

    pub fn stencils(&self) -> String {
        let mut lines: Vec<String> = Vec::new();
        for (entity, stencil) in &self.stencils {
            lines.push(entity.id().to_string());
            lines.push(stencil.to_svg());
        }
        lines.join("\n")
    }

    pub fn stencil_maps(&self) -> String {
        let mut lines: Vec<String> = Vec::new();
        for (entity, stencil) in &self.stencil_maps {
            lines.push(entity.id().to_string());
            lines.push(stencil.to_json());
        }
        lines.join("\n")
    }

    pub fn print_for_demo(&mut self) -> String {
        use kurbo::Vec2;

        self.exec();

        if let Some(root) = self.root.and_then(|root| self.stencil_maps.get(&root)) {
            root.clone()
                .with_translation(Vec2::new(0.0, -1500.0))
                .with_paper_size(3)
                .to_svg_doc_for_testing(&self.stencil_maps, &self.stencils)
        } else {
            String::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        use rhythm::NoteValue;
        use stencil::snapshot;

        let mut render = Render::default();
        let song = render.song_create();

        let staff = render.staff_create();
        let clef = render.between_bars_create(None, true, Some(4), Some(4));
        render.child_append(staff, clef);

        let bar1 = render.bar_create(4, 4);
        render.child_append(staff, bar1);

        let rnc1 = render.rnc_create(NoteValue::Eighth.log2() as isize, 0, 1, 8, true);

        render.bar_insert(bar1, rnc1);
        let barline = render.between_bars_create(Some(Barline::Normal), false, None, None);
        render.child_append(staff, barline);

        let bar2 = render.bar_create(4, 4);
        render.child_append(staff, bar2);

        let rnc2 = render.rnc_create(NoteValue::SixtyFourth.log2() as isize, 0, 1, 4, true);

        render.bar_insert(bar2, rnc2);

        let final_barline = render.between_bars_create(Some(Barline::Final), false, None, None);
        render.child_append(staff, final_barline);

        render.child_append(song, staff);
        render.root_set(song);

        render.exec();

        render.rnc_update_time(rnc1, NoteValue::Eighth.log2() as isize, 0, 4, 16);

        snapshot("./snapshots/hello_world.svg", &render.print_for_demo());
    }
}
