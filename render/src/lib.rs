#![allow(clippy::blacklisted_name)]

mod sys_delete_orphans;
mod sys_print_song;
use sys_delete_orphans::sys_delete_orphans;
use sys_print_song::sys_print_song;

use entity::{EntitiesRes, Entity, Join};
use kurbo::Rect;
use num_rational::Rational;
use rest_note_chord::{
    sys_apply_warp, sys_print_rnc, sys_record_space_time_warp, sys_relative_spacing,
    sys_update_rnc_timing, RestNoteChord, SpaceTimeWarp,
};
use rhythm::{Bar, Duration, Metre, NoteValue, RelativeRhythmicSpacing, Start};
use staff::{
    sys_print_between_bars, sys_print_staff, sys_print_staff_lines, sys_update_bar_numbers,
    Barline, BetweenBars, Staff,
};
use std::collections::{HashMap, HashSet};
use stencil::{sys_update_world_bboxes, Stencil, StencilMap};
use wasm_bindgen::prelude::*;

#[derive(Debug, Default)]
pub struct Song {
    freeze_spacing: Option<isize>,
    prev_freeze_spacing: Option<isize>,
}

#[wasm_bindgen]
#[derive(Debug, Default)]
pub struct Render {
    entities: EntitiesRes,

    root: Option<Entity>,
    parents: HashMap<Entity, Entity>,

    songs: HashMap<Entity, Song>,
    staffs: HashMap<Entity, Staff>,
    bars: HashMap<Entity, Bar>,
    between_bars: HashMap<Entity, BetweenBars>,
    rncs: HashMap<Entity, RestNoteChord>,
    stencils: HashMap<Entity, Stencil>,
    stencil_maps: HashMap<Entity, StencilMap>,
    world_bbox: HashMap<Entity, Rect>,
    starts: HashMap<Entity, Start>,
    spacing: HashMap<Entity, RelativeRhythmicSpacing>,
    ordered_children: HashMap<Entity, Vec<Entity>>,
    warps: HashMap<Entity, SpaceTimeWarp>,
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
    pub fn song_create(&mut self, freeze_spacing: Option<isize>) -> usize {
        let entity = self.entities.create();

        self.songs.insert(
            entity,
            Song {
                freeze_spacing,
                prev_freeze_spacing: None,
            },
        );
        self.ordered_children.insert(entity, vec![]);
        self.stencil_maps.insert(entity, StencilMap::default());

        entity.id()
    }

    pub fn song_set_freeze_spacing(&mut self, entity: usize, freeze_spacing: Option<isize>) {
        let entity = Entity::new(entity);
        if let Some(song) = self.songs.get_mut(&entity) {
            song.freeze_spacing = freeze_spacing;
        }
    }

    /// Create a staff, without attaching it to a song.
    pub fn staff_create(&mut self) -> usize {
        let entity = self.entities.create();
        let staff_lines_entity = self.entities.create();

        self.parents.insert(staff_lines_entity, entity);
        self.staffs.insert(entity, Staff::new(staff_lines_entity));

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
        self.starts.insert(entity, Start::default());

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
            if let (Some(rnc), Some(start)) = (self.rncs.get(&child), self.starts.get(&child)) {
                bar.splice(start.beat, vec![(rnc.duration(), Some(child))]);
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
            self.fixup_bar(bar_id);
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
            RestNoteChord::new(Duration::new(note_value, dots, None), is_note),
        );
        self.starts.insert(
            entity,
            Start {
                bar: 0,
                beat: start,
                natural_beat: start,
            },
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
        if let Some(parent_id) = self.parents.get(&rnc_id).copied() {
            if let (Some(rnc), Some(bar), Some(start)) = (
                self.rncs.get_mut(&rnc_id),
                self.bars.get_mut(&parent_id),
                self.starts.get_mut(&rnc_id),
            ) {
                bar.remove(rnc_id);
                start.beat = Rational::new(start_numer, start_denom);
                start.natural_beat = start.beat;
                rnc.duration = Duration::new(note_value, dots, None);
                rnc.natural_duration = rnc.duration;
                bar.splice(start.beat, vec![(rnc.duration(), Some(rnc_id))]);
            }
            self.fixup_bar(parent_id);
        }
    }

    fn fixup_bar(&mut self, parent_id: Entity) {
        // Fix previously overlapping notes.
        if let Some(bar) = self.bars.get_mut(&parent_id) {
            for (other_rnc_id, (rnc, parent, start)) in
                (&mut self.rncs, &self.parents, &mut self.starts).join()
            {
                if (rnc.natural_duration != rnc.duration || start.natural_beat != start.beat)
                    && *parent == parent_id
                {
                    bar.remove(other_rnc_id);
                    rnc.duration = rnc.natural_duration;
                    start.beat = start.natural_beat;
                    bar.splice(start.beat, vec![(rnc.duration(), Some(other_rnc_id))]);
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
        self.starts.insert(entity, Start::default());
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

    fn entities(&self) -> HashSet<Entity> {
        vec![
            self.songs.keys().copied().collect::<HashSet<Entity>>(),
            self.staffs.keys().copied().collect::<HashSet<Entity>>(),
            self.bars.keys().copied().collect::<HashSet<Entity>>(),
            self.between_bars
                .keys()
                .copied()
                .collect::<HashSet<Entity>>(),
            self.rncs.keys().copied().collect::<HashSet<Entity>>(),
            self.stencils.keys().copied().collect::<HashSet<Entity>>(),
            self.stencil_maps
                .keys()
                .copied()
                .collect::<HashSet<Entity>>(),
            self.world_bbox.keys().copied().collect::<HashSet<Entity>>(),
            self.starts.keys().copied().collect::<HashSet<Entity>>(),
            self.spacing.keys().copied().collect::<HashSet<Entity>>(),
            self.ordered_children
                .keys()
                .copied()
                .collect::<HashSet<Entity>>(),
        ]
        .into_iter()
        .flatten()
        .collect()
    }

    /* Frame */

    fn keep_spacing(&self) -> bool {
        self.root
            .and_then(|root| self.songs.get(&root))
            .map(|root| {
                root.freeze_spacing.is_some()
                    && (root.freeze_spacing == root.prev_freeze_spacing
                        || root.prev_freeze_spacing.is_none())
            })
            .unwrap_or(false)
    }

    pub fn exec(&mut self) {
        let entities = self.entities();
        sys_delete_orphans(
            self.root,
            &mut self.parents,
            &entities,
            &mut [
                &mut self.songs,
                &mut self.staffs,
                &mut self.bars,
                &mut self.between_bars,
                &mut self.rncs,
                &mut self.starts,
                &mut self.stencils,
                &mut self.stencil_maps,
                &mut self.spacing,
                &mut self.ordered_children,
                &mut self.warps,
            ],
        );

        sys_update_bar_numbers(
            &self.staffs,
            &self.ordered_children,
            &self.bars,
            &mut self.starts,
        );

        sys_update_rnc_timing(
            &self.entities,
            &mut self.rncs,
            &mut self.starts,
            &mut self.bars,
            &mut self.spacing,
            &mut self.parents,
            &mut self.stencils,
        );

        sys_print_rnc(&self.rncs, &mut self.stencils);
        if self.keep_spacing() {
            sys_apply_warp(&self.bars, &mut self.spacing, &self.warps);
        } else {
            sys_relative_spacing(
                &self.rncs,
                &self.parents,
                &self.bars,
                &self.stencils,
                &mut self.spacing,
            );
            sys_record_space_time_warp(&self.bars, &self.spacing, &mut self.warps);
        }
        sys_print_between_bars(&self.between_bars, &mut self.stencils);

        sys_print_staff(
            &mut self.staffs,
            &self.bars,
            &self.spacing,
            &self.stencils,
            &mut self.stencil_maps,
            &self.ordered_children,
        );
        sys_print_staff_lines(&self.staffs, &mut self.stencils);

        sys_print_song(&self.songs, &self.ordered_children, &mut self.stencil_maps);
        sys_update_world_bboxes(
            &self.songs,
            &self.stencils,
            &self.stencil_maps,
            &mut self.world_bbox,
        );
        if let Some(root) = self.root {
            if let Some(root) = self.songs.get_mut(&root) {
                root.prev_freeze_spacing = root.freeze_spacing;
            }
        }
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

    pub fn get_root_id(&self) -> Option<usize> {
        self.root.map(|root| root.id())
    }

    pub fn get_stencil_bboxes(&self) -> String {
        let mut lines: Vec<String> = Vec::new();
        for (entity, bbox) in &self.world_bbox {
            let start = self.starts.get(entity);
            lines.push(entity.id().to_string());
            let kind = if self.rncs.contains_key(entity) {
                0
            } else if self.between_bars.contains_key(entity) {
                1
            } else {
                -1
            };
            lines.push(format!(
                "[{},{},{},{},{},{},{},{}]",
                bbox.x0,
                bbox.y0,
                bbox.x1,
                bbox.y1,
                start.map(|s| s.bar as isize).unwrap_or(-1),
                start.map(|s| *s.beat.numer()).unwrap_or(0),
                start.map(|s| *s.beat.denom()).unwrap_or(1),
                kind,
            ));
        }
        lines.join("\n")
    }

    /// Returns [bar, num, den]
    pub fn get_time_for_cursor(&self, x: f64, y: f64) -> Option<Vec<usize>> {
        let quant = Rational::new(1, 8);
        for (_id, (bbox, bar, start)) in (&self.world_bbox, &self.bars, &self.starts).join() {
            if x >= bbox.x0 && x <= bbox.x1 && y >= bbox.y0 && y <= bbox.y1 {
                let child_starts: Vec<_> = bar
                    .children()
                    .into_iter()
                    .map(|c| {
                        (
                            self.world_bbox
                                .get(&c.2)
                                .map(|rect| rect.x0)
                                .unwrap_or_default(),
                            c.1,
                        )
                    })
                    .collect();
                for (i, (child_left, child_start_beat)) in child_starts.iter().enumerate().rev() {
                    if *child_left <= x {
                        let next = child_starts
                            .get(i + 1)
                            .copied()
                            .unwrap_or((bbox.x1, bar.metre().duration()));
                        let time_delta = next.1 - child_start_beat;
                        let quant = quant.min(time_delta);
                        let steps = time_delta / quant;
                        let steps = ((*steps.numer() as f64) / (*steps.denom() as f64)).ceil();

                        let pct = (x - *child_left) / (next.0 - *child_left);
                        let step = (pct * steps).floor() as usize;
                        let beat = child_start_beat + quant * (step as isize);

                        return Some(vec![
                            start.bar,
                            *beat.numer() as usize,
                            *beat.denom() as usize,
                            i,
                        ]);
                    }
                }

                return Some(vec![
                    start.bar,
                    *start.beat.numer() as usize,
                    *start.beat.denom() as usize,
                ]);
            }
        }
        None
    }

    pub fn print_for_demo(&mut self) -> String {
        self.exec();

        if let Some(root) = self.root.and_then(|root| self.stencil_maps.get(&root)) {
            root.clone()
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
        let song = render.song_create(None);

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

        // Make sure we can clean up and no entities are left over.
        render.root_clear(song);
        render.exec();

        assert_eq!(render.parents.len(), 0);
        assert_eq!(render.songs.len(), 0);
        assert_eq!(render.staffs.len(), 0);
        assert_eq!(render.bars.len(), 0);
        assert_eq!(render.between_bars.len(), 0);
        assert_eq!(render.rncs.len(), 0);
        assert_eq!(render.stencils.len(), 0);
        assert_eq!(render.stencil_maps.len(), 0);
        assert_eq!(render.world_bbox.len(), 0);
        assert_eq!(render.starts.len(), 0);
        assert_eq!(render.spacing.len(), 0);
        assert_eq!(render.ordered_children.len(), 0);
    }
}
