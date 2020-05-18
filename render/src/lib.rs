#![allow(clippy::blacklisted_name)]

mod sys_delete_orphans;
mod sys_print_meta;
mod sys_print_song;
use sys_delete_orphans::sys_delete_orphans;
use sys_print_meta::sys_print_meta;
use sys_print_song::sys_print_song;

use entity::{EntitiesRes, Entity, Join};
use kurbo::{Affine, Point, Rect, Size, Vec2};
use num_rational::Rational;
use pitch::{Clef, NoteModifier, Pitch};
use rest_note_chord::{
    sys_apply_warp, sys_draft_beaming, sys_print_beams, sys_print_rnc, sys_record_space_time_warp,
    sys_space_beams, sys_update_rnc_timing, Beam, Context, PitchKind, RestNoteChord, SpaceTimeWarp,
};
use rhythm::{Bar, Duration, Lifetime, Metre, NoteValue, RelativeRhythmicSpacing};
use staff::{
    sys_break_into_lines, sys_print_between_bars, sys_print_staff, sys_print_staff_lines,
    sys_update_context, Barline, BetweenBars, BreakIntoLineComponents, LineOfStaff, Staff,
};
use std::collections::{HashMap, HashSet};
use stencil::{sys_update_world_bboxes, Pdf, Stencil, StencilMap};
use wasm_bindgen::prelude::*;

#[derive(Debug)]
pub struct Song {
    freeze_spacing: Option<isize>,
    prev_freeze_spacing: Option<isize>,

    /// In mm
    width: f64,

    /// In mm
    height: f64,

    /// Convert from staff-size (1 unit is 1 staff) to paper-size (1 unit is 1 mm)
    ///
    /// Behind Bars, p483.
    ///
    /// Rastal sizes vary from 0 to 8, where 0 is large and 8 is small.
    ///  - 0 and 1 are used for educational music.
    ///  - 2 is not generally used, but is sometimes used for piano music/songs.
    ///  - 3-4 are commonly used for single-staff-parts, piano music, and songs.
    ///  - 5 is less commonly used for single-staff-parts, piano music, and songs.
    ///  - 6-7 are used for choral music, cue saves, or ossia.
    ///  - 8 is used for full scores.
    rastal_size: u8,

    title: String,
    title_width: f64,
    title_stencil: Option<Entity>,

    author: String,
    author_width: f64,
    author_stencil: Option<Entity>,
}

impl Default for Song {
    fn default() -> Song {
        Song {
            freeze_spacing: None,
            prev_freeze_spacing: None,

            width: 0f64,
            height: 0f64,
            rastal_size: 3,
            title: String::default(),
            title_width: 0f64,
            title_stencil: None,
            author: String::default(),
            author_width: 0f64,
            author_stencil: None,
        }
    }
}

impl Song {
    pub fn scale(&self) -> f64 {
        match self.rastal_size {
            0 => 9.2 / 1000.0,
            1 => 7.9 / 1000.0,
            2 => 7.4 / 1000.0,
            3 => 7.0 / 1000.0,
            4 => 6.5 / 1000.0,
            5 => 6.0 / 1000.0,
            6 => 5.5 / 1000.0,
            7 => 4.8 / 1000.0,
            8 => 3.7 / 1000.0,
            _ => panic!("Expected rastal size <= 8"),
        }
    }
}

#[wasm_bindgen]
#[derive(Debug, Default)]
pub struct Render {
    entities: EntitiesRes,

    root: Option<Entity>,
    parents: HashMap<Entity, Entity>,

    songs: HashMap<Entity, Song>,
    staffs: HashMap<Entity, Staff>,
    line_of_staffs: HashMap<Entity, LineOfStaff>,
    bars: HashMap<Entity, Bar>,
    between_bars: HashMap<Entity, BetweenBars>,
    rncs: HashMap<Entity, RestNoteChord>,
    beams: HashMap<Entity, Beam>,
    beam_for_rnc: HashMap<Entity, Entity>,
    stencils: HashMap<Entity, Stencil>,
    stencil_maps: HashMap<Entity, StencilMap>,
    world_bbox: HashMap<Entity, Rect>,
    contexts: HashMap<Entity, Context>,
    spacing: HashMap<Entity, RelativeRhythmicSpacing>,
    ordered_children: HashMap<Entity, Vec<Entity>>,
    warps: HashMap<Entity, SpaceTimeWarp>,
    attachments: HashMap<Entity, Option<Point>>,
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

    pub fn song_set_freeze_spacing(&mut self, entity: usize, freeze_spacing: Option<isize>) {
        let entity = Entity::new(entity);
        if let Some(song) = self.songs.get_mut(&entity) {
            song.freeze_spacing = freeze_spacing;
        }
    }

    pub fn song_set_size(&mut self, entity: usize, width: f64, height: f64) {
        let entity = Entity::new(entity);
        if let Some(song) = self.songs.get_mut(&entity) {
            song.width = width;
            song.height = height;
        }
    }

    pub fn song_set_title(&mut self, entity: usize, title: &str, width: f64) {
        let entity = Entity::new(entity);
        if let Some(song) = self.songs.get_mut(&entity) {
            song.title = title.to_owned();
            song.title_width = width;
        }
    }

    pub fn song_set_author(&mut self, entity: usize, author: &str, width: f64) {
        let entity = Entity::new(entity);
        if let Some(song) = self.songs.get_mut(&entity) {
            song.author = author.to_owned();
            song.author_width = width;
        }
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
        self.contexts.insert(entity, Context::default());

        entity.id()
    }

    /// Inserts a RestNoteChord into a bar.
    ///
    /// Note that children of bars are not ordered, instead children have a `start` property.
    pub fn bar_insert(&mut self, bar: usize, child: usize, is_temporary: bool) {
        let child = Entity::new(child);
        let bar_id = Entity::new(bar);

        if self.parents.contains_key(&child) {
            return;
        }

        if let Some(bar) = self.bars.get_mut(&bar_id) {
            if let (Some(rnc), Some(start)) = (self.rncs.get(&child), self.contexts.get(&child)) {
                bar.splice(
                    start.beat,
                    vec![(
                        rnc.duration(),
                        if is_temporary {
                            Lifetime::Temporary(child)
                        } else {
                            Lifetime::Explicit(child)
                        },
                    )],
                );
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

    pub fn bar_set_skip(&mut self, entity: usize, num: isize, den: isize) {
        let entity = Entity::new(entity);
        if let Some(bar) = self.bars.get_mut(&entity) {
            bar.set_pickup_skip(Rational::new(num, den));
        }
    }

    pub fn bar_clear_skip(&mut self, entity: usize) {
        let entity = Entity::new(entity);
        if let Some(bar) = self.bars.get_mut(&entity) {
            bar.clear_pickup_skip();
        }
    }

    /// Create a RestNoteChord, without attaching it to a bar.
    pub fn rnc_create(
        &mut self,
        note_value: isize,
        dots: u8,
        start_numer: isize,
        start_denom: isize,
    ) -> usize {
        let note_value = NoteValue::new(note_value).unwrap();
        let start = Rational::new(start_numer, start_denom);

        let entity = self.entities.create();

        self.spacing
            .insert(entity, RelativeRhythmicSpacing::default());
        self.rncs.insert(
            entity,
            RestNoteChord::new(Duration::new(note_value, dots, None), PitchKind::Rest),
        );
        self.contexts.insert(
            entity,
            Context {
                beat: start,
                natural_beat: start,
                ..Default::default()
            },
        );
        self.stencils.insert(entity, Stencil::default());

        entity.id()
    }

    pub fn rnc_set_rest(&mut self, rnc_id: usize) {
        let rnc_id = Entity::new(rnc_id);
        if let Some(rnc) = self.rncs.get_mut(&rnc_id) {
            rnc.pitch = PitchKind::Rest;
        }
    }

    pub fn rnc_set_unpitched(&mut self, rnc_id: usize) {
        let rnc_id = Entity::new(rnc_id);
        if let Some(rnc) = self.rncs.get_mut(&rnc_id) {
            rnc.pitch = PitchKind::Unpitched;
        }
    }

    pub fn rnc_set_pitch(&mut self, rnc_id: usize, midi: u8, modifier: i8) {
        let rnc_id = Entity::new(rnc_id);
        if let Some(rnc) = self.rncs.get_mut(&rnc_id) {
            rnc.pitch = PitchKind::Pitch(
                Pitch::from_base_midi(midi, NoteModifier::from_raw(modifier)).unwrap_or_default(),
            );
        }
    }

    pub fn rnc_update_time(
        &mut self,
        rnc_id: usize,
        note_value: isize,
        dots: u8,
        start_numer: isize,
        start_denom: isize,
        is_temporary: bool,
    ) {
        let note_value = NoteValue::new(note_value).unwrap();
        let rnc_id = Entity::new(rnc_id);
        if let Some(parent_id) = self.parents.get(&rnc_id).copied() {
            if let (Some(rnc), Some(bar), Some(start)) = (
                self.rncs.get_mut(&rnc_id),
                self.bars.get_mut(&parent_id),
                self.contexts.get_mut(&rnc_id),
            ) {
                bar.remove(rnc_id);
                start.beat = Rational::new(start_numer, start_denom);
                start.natural_beat = start.beat;
                rnc.duration = Duration::new(note_value, dots, None);
                rnc.natural_duration = rnc.duration;
                bar.splice(
                    start.beat,
                    vec![(
                        rnc.duration(),
                        if is_temporary {
                            Lifetime::Temporary(rnc_id)
                        } else {
                            Lifetime::Explicit(rnc_id)
                        },
                    )],
                );
            }
            self.fixup_bar(parent_id);
        }
    }

    fn fixup_bar(&mut self, parent_id: Entity) {
        // Fix previously overlapping notes.
        if let Some(bar) = self.bars.get_mut(&parent_id) {
            for (other_rnc_id, (rnc, parent, start)) in
                (&mut self.rncs, &self.parents, &mut self.contexts).join()
            {
                if (rnc.natural_duration != rnc.duration || start.natural_beat != start.beat)
                    && *parent == parent_id
                {
                    rnc.duration = rnc.natural_duration;
                    start.beat = start.natural_beat;
                    if let Some(lifetime) = bar.remove(other_rnc_id) {
                        bar.splice(start.beat, vec![(rnc.duration(), lifetime)]);
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
        clef: Option<Clef>,
        time_numer: Option<u8>,
        time_denom: Option<u8>,
        key: Option<i8>,
    ) -> usize {
        let entity = self.entities.create();

        let time = if let (Some(time_numer), Some(time_denom)) = (time_numer, time_denom) {
            Some((time_numer, time_denom))
        } else {
            None
        };

        let stencil_start = self.entities.create();
        self.stencils.insert(stencil_start, Stencil::default());
        self.parents.insert(stencil_start, entity);

        let stencil_middle = self.entities.create();
        self.parents.insert(stencil_middle, entity);
        self.stencils.insert(stencil_middle, Stencil::default());

        let stencil_end = self.entities.create();
        self.parents.insert(stencil_end, entity);
        self.stencils.insert(stencil_end, Stencil::default());

        self.between_bars.insert(
            entity,
            BetweenBars {
                barline,
                clef,
                time,
                key,
                stencil_start,
                stencil_middle,
                stencil_end,
            },
        );
        self.contexts.insert(entity, Context::default());

        entity.id()
    }

    pub fn between_bars_update(
        &mut self,
        entity: usize,
        barline: Option<Barline>,
        clef: Option<Clef>,
        time_numer: Option<u8>,
        time_denom: Option<u8>,
        key: Option<i8>,
    ) {
        let entity = Entity::new(entity);

        let time = if let (Some(time_numer), Some(time_denom)) = (time_numer, time_denom) {
            Some((time_numer, time_denom))
        } else {
            None
        };

        let bb = self.between_bars.remove(&entity).unwrap();

        self.between_bars.insert(
            entity,
            BetweenBars {
                barline,
                clef,
                time,
                key,
                ..bb
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
            self.contexts.keys().copied().collect::<HashSet<Entity>>(),
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
                &mut self.line_of_staffs,
                &mut self.staffs,
                &mut self.bars,
                &mut self.between_bars,
                &mut self.rncs,
                &mut self.contexts,
                &mut self.stencils,
                &mut self.stencil_maps,
                &mut self.spacing,
                &mut self.ordered_children,
                &mut self.warps,
                &mut self.beams,
                &mut self.beam_for_rnc,
                &mut self.attachments,
            ],
        );

        sys_update_rnc_timing(
            &self.entities,
            &mut self.rncs,
            &mut self.contexts,
            &mut self.bars,
            &mut self.spacing,
            &mut self.parents,
            &mut self.stencils,
        );

        sys_draft_beaming(
            &self.entities,
            &self.bars,
            &mut self.parents,
            &mut self.beam_for_rnc,
            &mut self.beams,
        );

        sys_update_context(
            &self.staffs,
            &self.ordered_children,
            &self.bars,
            &self.between_bars,
            &self.rncs,
            &mut self.contexts,
        );

        sys_print_rnc(
            &self.rncs,
            &self.contexts,
            &self.beam_for_rnc,
            &mut self.attachments,
            &mut self.stencils,
        );
        sys_print_between_bars(&self.between_bars, &self.contexts, &mut self.stencils);

        if self.keep_spacing() {
            sys_apply_warp(&self.bars, &mut self.spacing, &self.warps);
        } else {
            // TODO(joshuan): scale is fixed as rastal size 3.
            sys_break_into_lines(BreakIntoLineComponents {
                entities: &self.entities,
                page_size: self
                    .root
                    .and_then(|root| self.songs.get(&root))
                    .map(|root| (root.width / 7.0 * 1000.0, root.height / 7.0 * 1000.0)),
                bars: &self.bars,
                between_bars: &self.between_bars,
                stencils: &self.stencils,
                spacing: &mut self.spacing,
                staffs: &mut self.staffs,
                parents: &mut self.parents,
                ordered_children: &mut self.ordered_children,
                line_of_staffs: &mut self.line_of_staffs,
            });
            sys_record_space_time_warp(&self.bars, &self.spacing, &mut self.warps);
        }

        sys_space_beams(
            &self.bars,
            &self.spacing,
            &self.beam_for_rnc,
            &self.attachments,
            &mut self.beams,
        );

        sys_print_beams(&self.beams, &mut self.stencils);

        sys_print_staff(
            &mut self.line_of_staffs,
            &self.bars,
            &self.beam_for_rnc,
            &self.spacing,
            &self.stencils,
            &mut self.stencil_maps,
            &self.ordered_children,
        );
        sys_print_staff_lines(&self.line_of_staffs, &mut self.stencils);

        sys_print_meta(
            &self.entities,
            &mut self.parents,
            &mut self.songs,
            &mut self.stencils,
        );

        sys_print_song(
            &self.songs,
            &self.staffs,
            &self.ordered_children,
            &mut self.stencil_maps,
        );
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

    pub fn parents(&self) -> String {
        let mut lines: Vec<String> = Vec::new();
        for (parent, child) in &self.parents {
            lines.push(parent.id().to_string());
            lines.push(child.id().to_string());
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

    pub fn get_song_width(&self, song: usize) -> Option<f64> {
        let song = Entity::new(song);
        self.songs
            .get(&song)
            .map(|song| (song.width / song.scale()).round())
    }

    pub fn get_song_height(&self, song: usize) -> Option<f64> {
        let song = Entity::new(song);
        self.songs
            .get(&song)
            .map(|song| (song.height / song.scale()).round())
    }

    pub fn get_stencil_bboxes(&self) -> String {
        let mut lines: Vec<String> = Vec::new();
        for (entity, bbox) in &self.world_bbox {
            let start = self.contexts.get(entity);
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

    /// Returns [bar, num, den, pitch_base, pitch_modifier]
    pub fn get_hover_info(&self, x: f64, y: f64) -> Option<Vec<isize>> {
        let quant = Rational::new(1, 8);
        for (_id, (bbox, bar, context)) in (&self.world_bbox, &self.bars, &self.contexts).join() {
            if x >= bbox.x0 && x <= bbox.x1 && y >= bbox.y0 && y <= bbox.y1 {
                let child_contexts: Vec<_> = bar
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
                let middle_y = (bbox.y0 + bbox.y1) / 2.0;

                let pitch = Pitch::from_y(middle_y - y, context.clef, context.key);
                for (i, (child_left, child_start_beat)) in child_contexts.iter().enumerate().rev() {
                    if *child_left <= x {
                        let next = child_contexts
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
                            context.bar as isize,
                            *beat.numer() as isize,
                            *beat.denom() as isize,
                            pitch.base_midi() as isize,
                            pitch.modifier().map(|m| m as isize).unwrap_or(0),
                        ]);
                    }
                }

                return Some(vec![
                    context.bar as isize,
                    *context.beat.numer() as isize,
                    *context.beat.denom() as isize,
                    pitch.base_midi() as isize,
                    pitch.modifier().map(|m| m as isize).unwrap_or(0),
                ]);
            }
        }
        None
    }

    pub fn split_note(
        &self,
        bar: usize,
        start_num: isize,
        start_den: isize,
        duration_num: isize,
        duration_den: isize,
    ) -> Vec<isize> {
        if let Some(bar) = self.bars.get(&Entity::new(bar)) {
            let mut start = Rational::new(start_num, start_den);
            let solution = bar.split_note(
                start,
                Duration::exact(Rational::new(duration_num, duration_den), None),
            );

            // Format it.
            solution
                .into_iter()
                .map(|part| {
                    let my_start = start;
                    start += part.duration();
                    vec![
                        part.duration_display_base()
                            .map(|d| d as isize)
                            .unwrap_or(0),
                        part.display_dots().map(|d| d as isize).unwrap_or(0),
                        *my_start.numer(),
                        *my_start.denom(),
                    ]
                    .into_iter()
                })
                .flatten()
                .collect()
        } else {
            vec![]
        }
    }

    pub fn print_for_demo(&mut self) -> String {
        self.exec();

        let song = &self.songs[&self.root.unwrap()];

        if let Some(root) = self.root.and_then(|root| self.stencil_maps.get(&root)) {
            root.clone()
                .to_svg_doc_for_testing(song.scale(), &self.stencil_maps, &self.stencils)
        } else {
            String::default()
        }
    }

    pub fn to_pdf(&self, embed_file: Option<String>) -> String {
        let song = &self.songs[&self.root.unwrap()];

        if let Some(root) = self.root.and_then(|root| self.stencil_maps.get(&root)) {
            let mut pdf = Pdf::new();
            let scale = song.scale();
            pdf.add_page(Size::new(215.9, 279.4));
            if let Some(embed_file) = embed_file {
                pdf.add_file(&embed_file);
            }

            pdf.write_stencil_map(
                root,
                Affine::translate(Vec2::new(0.0, 279.4)) * Affine::scale(scale) * Affine::FLIP_Y,
                &self.stencils,
                &self.stencil_maps,
            );
            base64::encode(pdf.into_binary())
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
        render.song_set_size(song, 215.9, 279.4);
        render.song_set_title(song, "Six Eight", 26.4f64);
        render.song_set_author(song, "Six Eight", 26.4f64 * 5f64 / 7f64);

        let staff = render.staff_create();
        let clef =
            render.between_bars_create(None, Some(Clef::Percussion), Some(4), Some(4), Some(0));
        render.child_append(staff, clef);

        let bar1 = render.bar_create(4, 4);
        render.child_append(staff, bar1);

        let rnc1 = render.rnc_create(NoteValue::Eighth.log2() as isize, 0, 1, 8);
        render.rnc_set_unpitched(rnc1);

        render.bar_insert(bar1, rnc1, false);
        let barline = render.between_bars_create(Some(Barline::Normal), None, None, None, Some(0));
        render.child_append(staff, barline);

        let bar2 = render.bar_create(4, 4);
        render.child_append(staff, bar2);

        let rnc2 = render.rnc_create(NoteValue::SixtyFourth.log2() as isize, 0, 1, 4);
        render.rnc_set_unpitched(rnc2);

        render.bar_insert(bar2, rnc2, false);

        let final_barline =
            render.between_bars_create(Some(Barline::Final), None, None, None, Some(0));
        render.child_append(staff, final_barline);

        render.child_append(song, staff);
        render.root_set(song);

        render.exec();

        render.rnc_update_time(rnc1, NoteValue::Eighth.log2() as isize, 0, 4, 16, false);

        snapshot("./snapshots/hello_world.svg", &render.print_for_demo());

        // Make sure we can clean up and no entities are left over.
        render.root_clear(song);
        render.exec();

        assert_eq!(render.parents.len(), 0);
        assert_eq!(render.songs.len(), 0);
        assert_eq!(render.staffs.len(), 0);
        assert_eq!(render.line_of_staffs.len(), 0);
        assert_eq!(render.bars.len(), 0);
        assert_eq!(render.between_bars.len(), 0);
        assert_eq!(render.rncs.len(), 0);
        assert_eq!(render.stencils.len(), 0);
        assert_eq!(render.stencil_maps.len(), 0);
        assert_eq!(render.world_bbox.len(), 0);
        assert_eq!(render.contexts.len(), 0);
        assert_eq!(render.spacing.len(), 0);
        assert_eq!(render.ordered_children.len(), 0);
    }

    #[test]
    fn beaming_1() {
        use rhythm::NoteValue;
        use stencil::snapshot;

        let mut render = Render::default();
        let song = render.song_create();
        render.song_set_size(song, 215.9, 279.4);
        render.song_set_title(song, "Six Eight", 26.4f64);
        render.song_set_author(song, "Six Eight", 26.4f64 * 5f64 / 7f64);

        let staff = render.staff_create();
        let clef = render.between_bars_create(None, Some(Clef::G), Some(4), Some(4), Some(0));
        render.child_append(staff, clef);

        let bar1 = render.bar_create(4, 4);
        render.child_append(staff, bar1);

        let rnc1 = render.rnc_create(NoteValue::Eighth.log2() as isize, 0, 0, 1);
        let rnc2 = render.rnc_create(NoteValue::Eighth.log2() as isize, 0, 1, 8);

        render.rnc_set_pitch(rnc1, 60, 0);
        render.bar_insert(bar1, rnc1, false);
        render.rnc_set_pitch(rnc2, 60, 0);
        render.bar_insert(bar1, rnc2, false);

        let final_barline =
            render.between_bars_create(Some(Barline::Final), None, None, None, Some(0));
        render.child_append(staff, final_barline);

        render.child_append(song, staff);
        render.root_set(song);

        render.exec();

        snapshot("./snapshots/beaming_1.svg", &render.print_for_demo());
    }
}
