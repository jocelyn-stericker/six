#![allow(clippy::blacklisted_name)]

mod systems;

use crate::systems::{DeleteOrphans, PrintMeta, PrintSong, UpdateKeepSpacing, UpdateWorldBbox};
use kurbo::{Affine, Size, Vec2};
use num_rational::Rational;
use pitch::{Clef, NoteModifier, Pitch};
use rhythm::{
    components::{Bar, Spacing},
    BarChild, Duration, Lifetime, Metre, NoteValue,
};
use specs::{Builder, Entity, Join, RunNow, World, WorldExt};
use staff::{
    components::{
        Beam, BeamForChord, BetweenBars, Children, Chord, Context, Cursor, FlagAttachment,
        LineOfStaff, Song, SpaceTimeWarp, Staff,
    },
    resources::{KeepSpacing, Root},
    systems::{
        ApplySpaceTimeWarp, BreakIntoLines, DraftBeam, PrintBeam, PrintBetweenBar, PrintChord,
        PrintCursor, PrintStaff, PrintStaffLines, RecordSpaceTimeWarp, SpaceBeam, UpdateContext,
        UpdateTiming,
    },
    Barline, PitchKind,
};
use std::convert::TryInto;
use stencil::{
    components::{Parent, Stencil, StencilMap, WorldBbox},
    Pdf,
};
use wasm_bindgen::prelude::*;

#[derive(Default)]
struct Systems {
    apply_space_time_warp: ApplySpaceTimeWarp,
    draft_beam: DraftBeam,
    print_beam: PrintBeam,
    print_chord: PrintChord,
    record_space_time_warp: RecordSpaceTimeWarp,
    space_beam: SpaceBeam,
    update_timing: UpdateTiming,
    break_into_lines: BreakIntoLines,
    print_between_bar: PrintBetweenBar,
    print_staff: PrintStaff,
    print_staff_lines: PrintStaffLines,
    print_cursor: PrintCursor,
    delete_orphans: DeleteOrphans,
    print_meta: PrintMeta,
    print_song: PrintSong,
    update_keep_spacing: UpdateKeepSpacing,
    update_world_bbox: UpdateWorldBbox,
    update_context: UpdateContext,
}

#[wasm_bindgen]
pub struct Render {
    world: World,

    systems: Systems,
}

impl Default for Render {
    fn default() -> Render {
        let mut world = World::new();
        world.insert(Root(None));
        world.insert(KeepSpacing(false));
        world.register::<Parent>();
        world.register::<Song>();
        world.register::<Staff>();
        world.register::<LineOfStaff>();
        world.register::<Bar>();
        world.register::<BetweenBars>();
        world.register::<Chord>();
        world.register::<Beam>();
        world.register::<BeamForChord>();
        world.register::<Stencil>();
        world.register::<StencilMap>();
        world.register::<WorldBbox>();
        world.register::<Context>();
        world.register::<Spacing>();
        world.register::<Children>();
        world.register::<SpaceTimeWarp>();
        world.register::<FlagAttachment>();
        world.register::<Cursor>();

        Self {
            world,
            systems: Default::default(),
        }
    }
}

// /// A DOM-based interface to Six Eight's ECS.
#[wasm_bindgen]
impl Render {
    pub fn new() -> Render {
        Self::default()
    }

    /// Set the Song entity to be rendered.
    pub fn root_set(&mut self, song: u32) {
        let song = self.world.entities().entity(song);
        if let Some(root) = self.world.get_mut::<Root>() {
            if root.0.is_none() {
                *root = Root(Some(song));
            }
        }
    }

    /// Clear the Song entity to be rendered.
    pub fn root_clear(&mut self, song: u32) {
        let song = self.world.entities().entity(song);
        if let Some(root) = self.world.get_mut::<Root>() {
            if root.0 == Some(song) {
                *root = Root(None);
            }
        }
    }

    /// Append `child` to the `parent` ordered container.
    ///
    /// If `child` already has a parent, nothing will happen.
    pub fn child_append(&mut self, parent: u32, child: u32) {
        let parent = self.world.entities().entity(parent);
        let child = self.world.entities().entity(child);

        let mut parents = self.world.write_component::<Parent>();
        let mut children = self.world.write_component::<Children>();

        if parents.contains(child) {
            return;
        }

        if let Some(ordered_children) = children.get_mut(parent) {
            ordered_children.0.push(child);
            parents.insert(child, Parent(parent)).unwrap();
        }
    }

    /// Insert `child` to the `parent` ordered container, before `before`.
    ///
    /// If `child` already has a parent, or `before` is not a child of `parent`, nothing will
    /// happen.
    pub fn child_insert_before(&mut self, parent: u32, before: u32, child: u32) {
        let before = self.world.entities().entity(before);
        let child = self.world.entities().entity(child);
        let parent = self.world.entities().entity(parent);

        let mut parents = self.world.write_component::<Parent>();
        let mut children = self.world.write_component::<Children>();

        if parents.contains(child) {
            return;
        }

        if let Some(ordered_children) = children.get_mut(parent) {
            if let Some(idx) = ordered_children.0.iter().position(|&x| x == before) {
                ordered_children.0.insert(idx, child);
                parents.insert(child, Parent(parent)).unwrap();
            }
        }
    }

    /// Remove `child` from the `parent` ordered container.
    ///
    /// If `child` is not a child of `parent`, nothing will happen.
    pub fn child_remove(&mut self, parent: u32, exchild: u32) {
        let parent = self.world.entities().entity(parent);
        let exchild = self.world.entities().entity(exchild);

        let mut parents = self.world.write_component::<Parent>();
        let mut children = self.world.write_component::<Children>();

        if let Some(ordered_children) = children.get_mut(parent) {
            if let Some(entity_idx) = ordered_children.0.iter().position(|&x| x == exchild) {
                ordered_children.0.remove(entity_idx);
                parents.remove(exchild);
            }
        }
    }

    /// Create a song, without attaching it as the document root.
    pub fn song_create(&mut self) -> u32 {
        self.world
            .create_entity()
            .with(Song::default())
            .with(Children::default())
            .with(StencilMap::default())
            .build()
            .id()
    }

    pub fn song_set_freeze_spacing(&mut self, song: u32, freeze_spacing: Option<isize>) {
        let song = self.world.entities().entity(song);
        let mut songs = self.world.write_component::<Song>();

        if let Some(song) = songs.get_mut(song) {
            song.freeze_spacing = freeze_spacing;
        }
    }

    pub fn song_set_size(&mut self, song: u32, width: f64, height: f64) {
        let song = self.world.entities().entity(song);
        let mut songs = self.world.write_component::<Song>();

        if let Some(song) = songs.get_mut(song) {
            song.width = width;
            song.height = height;
        }
    }

    pub fn song_set_title(&mut self, song: u32, title: &str, width: f64) {
        let song = self.world.entities().entity(song);
        let mut songs = self.world.write_component::<Song>();

        if let Some(song) = songs.get_mut(song) {
            song.title = title.to_owned();
            song.title_width = width;
        }
    }

    pub fn song_set_author(&mut self, song: u32, author: &str, width: f64) {
        let song = self.world.entities().entity(song);
        let mut songs = self.world.write_component::<Song>();

        if let Some(song) = songs.get_mut(song) {
            song.author = author.to_owned();
            song.author_width = width;
        }
    }

    /// Create a staff, without attaching it to a song.
    pub fn staff_create(&mut self) -> u32 {
        self.world
            .create_entity()
            .with(Staff::default())
            .with(StencilMap::default())
            .with(Children::default())
            .build()
            .id()
    }

    fn bar_by_index(&self, staff_children: &Vec<Entity>, idx: usize) -> Option<Entity> {
        let bars = self.world.read_component::<Bar>();

        let mut i = 0;
        for &child in staff_children {
            if bars.contains(child) {
                if i == idx {
                    return Some(child);
                }
                i += 1;
            }
        }

        None
    }

    pub fn staff_time_cursor_add(
        &self,
        staff: u32,
        bar_idx: usize,
        time_num: isize,
        time_den: isize,
        add_num: isize,
        add_den: isize,
    ) -> Option<Vec<isize>> {
        let staff = self.world.entities().entity(staff);

        let children = self.world.read_component::<Children>();
        let bars = self.world.read_component::<Bar>();
        let chords = self.world.read_component::<Chord>();

        let staff_bars = children.get(staff)?;
        let bar = bars.get(self.bar_by_index(&staff_bars.0, bar_idx)?)?;
        let add = Rational::new(add_num, add_den);
        let t = Rational::new(time_num, time_den) + add;
        let mut t = if t < Rational::new(0, 1) {
            let prev_bar = bars.get(self.bar_by_index(&staff_bars.0, bar_idx - 1)?)?;
            let t = prev_bar.metre().duration() + t;
            if t < Rational::new(0, 1) {
                None
            } else {
                Some(((bar_idx - 1).try_into().unwrap(), t))
            }
        } else if t >= bar.metre().duration() {
            let next_bar = bars.get(self.bar_by_index(&staff_bars.0, bar_idx + 1)?)?;
            let t = t - bar.metre().duration();
            if t >= next_bar.metre().duration() {
                None
            } else {
                Some(((bar_idx + 1).try_into().unwrap(), t))
            }
        } else {
            Some((bar_idx.try_into().unwrap(), t))
        }?;

        let bar = bars.get(self.bar_by_index(&staff_bars.0, t.0 as usize)?)?;

        // Make sure we are not in the middle of a note.
        for BarChild {
            duration,
            start,
            lifetime,
            stencil,
        } in bar.children()
        {
            if !lifetime.is_temporary()
                && !chords.get(stencil).unwrap().pitch.is_rest()
                && t.1 > start
                && t.1 < start + duration.duration()
            {
                if add >= Rational::new(0, 1) {
                    t = (t.0, start + duration.duration());
                } else {
                    t = (t.0, start);
                }
            }
        }

        Some(vec![t.0, *t.1.numer(), *t.1.denom()])
    }

    /// Create a bar, without attaching it to a staff.
    ///
    /// `numer` and `denom` are the numerator and denominator of the time signature in this bar.
    pub fn bar_create(&mut self, numer: u8, denom: u8) -> u32 {
        self.world
            .create_entity()
            .with(Bar::new(Metre::new(numer, denom)))
            .with(StencilMap::default())
            .with(Context::default())
            .build()
            .id()
    }

    /// Inserts a Chord into a bar.
    ///
    /// Note that children of bars are not ordered, instead children have a `start` property.
    pub fn bar_insert(&mut self, bar: u32, child: u32, is_temporary: bool) {
        let child = self.world.entities().entity(child);
        let bar = self.world.entities().entity(bar);

        let mut parents = self.world.write_component::<Parent>();
        let mut bars = self.world.write_component::<Bar>();
        let chords = self.world.read_component::<Chord>();
        let contexts = self.world.read_component::<Context>();

        if parents.contains(child) {
            return;
        }

        let parent = Parent(bar);

        if let Some(bar) = bars.get_mut(bar) {
            if let (Some(chord), Some(start)) = (chords.get(child), contexts.get(child)) {
                bar.splice(
                    start.beat,
                    vec![(
                        chord.duration(),
                        if is_temporary {
                            Lifetime::Temporary(child)
                        } else {
                            Lifetime::Explicit(child)
                        },
                    )],
                );
                parents.insert(child, parent).unwrap();
            }
        }
    }

    /// Remove a Chord from a bar.
    ///
    /// Note that children of bars are not ordered, instead children have a `start` property.
    pub fn bar_remove(&mut self, bar: u32, child: u32) {
        let bar = self.world.entities().entity(bar);
        let child = self.world.entities().entity(child);

        {
            let mut parents = self.world.write_component::<Parent>();
            let mut bars = self.world.write_component::<Bar>();

            if let Some(bar) = bars.get_mut(bar) {
                bar.remove(child);
                parents.remove(child);
            } else {
                return;
            }
        }

        self.fixup_bar(bar);
    }

    pub fn bar_set_skip(&mut self, bar: u32, num: isize, den: isize) {
        let bar = self.world.entities().entity(bar);
        let mut bars = self.world.write_component::<Bar>();

        if let Some(bar) = bars.get_mut(bar) {
            bar.set_pickup_skip(Rational::new(num, den));
        }
    }

    pub fn bar_clear_skip(&mut self, bar: u32) {
        let bar = self.world.entities().entity(bar);
        let mut bars = self.world.write_component::<Bar>();

        if let Some(bar) = bars.get_mut(bar) {
            bar.clear_pickup_skip();
        }
    }

    /// Create a Chord, without attaching it to a bar.
    pub fn chord_create(
        &mut self,
        note_value: isize,
        dots: u8,
        start_numer: isize,
        start_denom: isize,
    ) -> u32 {
        let note_value = NoteValue::new(note_value).unwrap();
        let start = Rational::new(start_numer, start_denom);

        self.world
            .create_entity()
            .with(Spacing::default())
            .with(Chord::new(
                Duration::new(note_value, dots, None),
                PitchKind::Rest,
            ))
            .with(Children::default())
            .with(Context {
                beat: start,
                natural_beat: start,
                ..Default::default()
            })
            .with(FlagAttachment::default())
            .with(Stencil::default())
            .build()
            .id()
    }

    pub fn chord_set_rest(&mut self, chord: u32) {
        let chord = self.world.entities().entity(chord);
        let mut chords = self.world.write_component::<Chord>();

        if let Some(chord) = chords.get_mut(chord) {
            chord.pitch = PitchKind::Rest;
        }
    }

    pub fn chord_set_unpitched(&mut self, chord: u32) {
        let chord = self.world.entities().entity(chord);
        let mut chords = self.world.write_component::<Chord>();

        if let Some(chord) = chords.get_mut(chord) {
            chord.pitch = PitchKind::Unpitched;
        }
    }

    pub fn chord_set_pitch(&mut self, chord: u32, midi: u8, modifier: i8) {
        let chord = self.world.entities().entity(chord);
        let mut chords = self.world.write_component::<Chord>();

        if let Some(chord) = chords.get_mut(chord) {
            chord.pitch = PitchKind::Pitch(
                Pitch::from_base_midi(midi, NoteModifier::from_raw(modifier)).unwrap_or_default(),
            );
        }
    }

    pub fn chord_update_time(
        &mut self,
        chord_ent: u32,
        note_value: isize,
        dots: u8,
        start_numer: isize,
        start_denom: isize,
        is_temporary: bool,
    ) {
        let note_value = NoteValue::new(note_value).unwrap();
        let chord_ent = self.world.entities().entity(chord_ent);
        let mut chords = self.world.write_component::<Chord>();
        let mut bars = self.world.write_component::<Bar>();
        let mut contexts = self.world.write_component::<Context>();
        let parents = self.world.read_component::<Parent>();

        if let Some(parent) = parents.get(chord_ent).copied() {
            if let (Some(chord), Some(bar), Some(start)) = (
                chords.get_mut(chord_ent),
                bars.get_mut(parent.0),
                contexts.get_mut(chord_ent),
            ) {
                bar.remove(chord_ent);
                start.beat = Rational::new(start_numer, start_denom);
                start.natural_beat = start.beat;
                chord.duration = Duration::new(note_value, dots, None);
                chord.natural_duration = chord.duration;
                bar.splice(
                    start.beat,
                    vec![(
                        chord.duration(),
                        if is_temporary {
                            Lifetime::Temporary(chord_ent)
                        } else {
                            Lifetime::Explicit(chord_ent)
                        },
                    )],
                );
            }

            drop(chords);
            drop(bars);
            drop(contexts);
            drop(parents);
            self.fixup_bar(parent.0);
        }
    }

    pub fn cursor_create(&mut self) -> u32 {
        self.world
            .create_entity()
            .with(Cursor {})
            .with(Stencil::default())
            .build()
            .id()
    }

    fn fixup_bar(&mut self, parent_id: Entity) {
        let parents = self.world.read_component::<Parent>();
        let mut bars = self.world.write_component::<Bar>();
        let mut chords = self.world.write_component::<Chord>();
        let mut contexts = self.world.write_component::<Context>();

        // Fix previously overlapping notes.
        if let Some(bar) = bars.get_mut(parent_id) {
            for (other_chord_id, chord, parent, start) in
                (&self.world.entities(), &mut chords, &parents, &mut contexts).join()
            {
                if (chord.natural_duration != chord.duration || start.natural_beat != start.beat)
                    && parent.0 == parent_id
                {
                    chord.duration = chord.natural_duration;
                    start.beat = start.natural_beat;
                    if let Some(lifetime) = bar.remove(other_chord_id) {
                        bar.splice(start.beat, vec![(chord.duration(), lifetime)]);
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
    ) -> u32 {
        let stencil_start = self.world.create_entity().with(Stencil::default()).build();
        let stencil_middle = self.world.create_entity().with(Stencil::default()).build();
        let stencil_end = self.world.create_entity().with(Stencil::default()).build();

        let time = if let (Some(time_numer), Some(time_denom)) = (time_numer, time_denom) {
            Some((time_numer, time_denom))
        } else {
            None
        };

        let between_bars = self
            .world
            .create_entity()
            .with(Context::default())
            .with(BetweenBars {
                barline,
                clef,
                time,
                key,
                stencil_start,
                stencil_middle,
                stencil_end,
            })
            .build();

        let mut parents = self.world.write_component::<Parent>();

        parents.insert(stencil_start, Parent(between_bars)).unwrap();
        parents
            .insert(stencil_middle, Parent(between_bars))
            .unwrap();
        parents.insert(stencil_end, Parent(between_bars)).unwrap();

        between_bars.id()
    }

    pub fn between_bars_update(
        &mut self,
        between_bar: u32,
        barline: Option<Barline>,
        clef: Option<Clef>,
        time_numer: Option<u8>,
        time_denom: Option<u8>,
        key: Option<i8>,
    ) {
        let between_bar = self.world.entities().entity(between_bar);
        let mut between_bars = self.world.write_storage::<BetweenBars>();

        let time = if let (Some(time_numer), Some(time_denom)) = (time_numer, time_denom) {
            Some((time_numer, time_denom))
        } else {
            None
        };

        let bb = between_bars.remove(between_bar).unwrap();

        between_bars
            .insert(
                between_bar,
                BetweenBars {
                    barline,
                    clef,
                    time,
                    key,
                    ..bb
                },
            )
            .unwrap();
    }

    /* Frame */

    pub fn exec(&mut self) {
        self.systems.delete_orphans.run_now(&self.world);
        self.world.maintain();

        self.systems.update_keep_spacing.run_now(&self.world);
        self.systems.update_timing.run_now(&self.world);
        self.world.maintain();
        self.systems.draft_beam.run_now(&self.world);
        self.systems.update_context.run_now(&self.world);

        self.systems.print_chord.run_now(&self.world);
        self.systems.print_between_bar.run_now(&self.world);

        self.systems.apply_space_time_warp.run_now(&self.world);
        self.systems.break_into_lines.run_now(&self.world);
        self.systems.record_space_time_warp.run_now(&self.world);

        self.systems.space_beam.run_now(&self.world);
        self.systems.print_beam.run_now(&self.world);
        self.systems.print_staff_lines.run_now(&self.world);
        self.systems.print_cursor.run_now(&self.world);

        self.systems.print_staff.run_now(&self.world);
        self.systems.print_staff_lines.run_now(&self.world);
        self.systems.print_meta.run_now(&self.world);
        self.systems.print_song.run_now(&self.world);

        self.systems.update_world_bbox.run_now(&self.world);
        self.world.maintain();

        if let Some(root) = self.world.read_resource::<Root>().0 {
            if let Some(root) = self.world.write_component::<Song>().get_mut(root) {
                root.prev_freeze_spacing = root.freeze_spacing;
            }
        }
    }

    pub fn stencils(&self) -> String {
        let mut lines: Vec<String> = Vec::new();
        for (entity, stencil) in (
            &self.world.entities(),
            &self.world.read_component::<Stencil>(),
        )
            .join()
        {
            lines.push(entity.id().to_string());
            lines.push(stencil.to_svg());
        }
        lines.join("\n")
    }

    pub fn parents(&self) -> String {
        let mut lines: Vec<String> = Vec::new();
        for (child, parent) in (
            &self.world.entities(),
            &self.world.read_component::<Parent>(),
        )
            .join()
        {
            lines.push(child.id().to_string());
            lines.push(parent.0.id().to_string());
        }
        lines.join("\n")
    }

    pub fn stencil_maps(&self) -> String {
        let mut lines: Vec<String> = Vec::new();
        for (entity, stencil) in (
            &self.world.entities(),
            &self.world.read_component::<StencilMap>(),
        )
            .join()
        {
            lines.push(entity.id().to_string());
            lines.push(stencil.to_json());
        }
        lines.join("\n")
    }

    pub fn get_root_id(&self) -> Option<u32> {
        self.world.read_resource::<Root>().0.map(|root| root.id())
    }

    pub fn get_song_width(&self, song: u32) -> Option<f64> {
        let song = self.world.entities().entity(song);
        self.world
            .read_component::<Song>()
            .get(song)
            .map(|song| (song.width / song.scale()).round())
    }

    pub fn get_song_height(&self, song: u32) -> Option<f64> {
        let song = self.world.entities().entity(song);
        self.world
            .read_component::<Song>()
            .get(song)
            .map(|song| (song.height / song.scale()).round())
    }

    pub fn get_stencil_bboxes(&self) -> String {
        let mut lines: Vec<String> = Vec::new();
        for (entity, bbox) in (
            &self.world.entities(),
            &self.world.read_component::<WorldBbox>(),
        )
            .join()
        {
            let contexts = self.world.read_component::<Context>();
            let start = contexts.get(entity);
            lines.push(entity.id().to_string());
            let kind = if self.world.read_component::<Chord>().contains(entity) {
                0
            } else if self.world.read_component::<BetweenBars>().contains(entity) {
                1
            } else {
                -1
            };
            lines.push(format!(
                "[{},{},{},{},{},{},{},{}]",
                bbox.0.x0,
                bbox.0.y0,
                bbox.0.x1,
                bbox.0.y1,
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
        for (WorldBbox(bbox), bar, context) in (
            &self.world.read_component::<WorldBbox>(),
            &self.world.read_component::<Bar>(),
            &self.world.read_component::<Context>(),
        )
            .join()
        {
            if x >= bbox.x0 && x <= bbox.x1 && y >= bbox.y0 && y <= bbox.y1 {
                let child_contexts: Vec<_> = bar
                    .children()
                    .into_iter()
                    .map(|c| {
                        (
                            self.world
                                .read_component::<WorldBbox>()
                                .get(c.stencil)
                                .map(|rect| rect.0.x0)
                                .unwrap_or_default(),
                            c.start,
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
        bar: u32,
        start_num: isize,
        start_den: isize,
        duration_num: isize,
        duration_den: isize,
    ) -> Vec<isize> {
        let bars = self.world.read_component::<Bar>();

        if let Some(bar) = bars.get(self.world.entities().entity(bar)) {
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

    pub fn util_duration_to_frac(&self, note_value: isize, dots: u8) -> Vec<isize> {
        let note_value = NoteValue::new(note_value).unwrap();
        let d = Duration::new(note_value, dots, None).duration();
        vec![*d.numer(), *d.denom()]
    }

    pub fn util_frac_add(
        &self,
        a_num: isize,
        a_den: isize,
        b_num: isize,
        b_den: isize,
    ) -> Vec<isize> {
        let f = Rational::new(a_num, a_den) + Rational::new(b_num, b_den);
        vec![*f.numer(), *f.denom()]
    }

    // pub fn print_for_demo(&mut self) -> String {
    //     self.exec();

    //     let song = &self.songs[&self.root.unwrap()];

    //     if let Some(root) = self.root.and_then(|root| self.stencil_maps.get(&root)) {
    //         root.clone()
    //             .to_svg_doc_for_testing(song.scale(), &self.stencil_maps, &self.stencils)
    //     } else {
    //         String::default()
    //     }
    // }

    pub fn to_pdf(&self, embed_file: Option<String>) -> Option<String> {
        let songs = self.world.read_component::<Song>();
        let stencils = self.world.read_component::<Stencil>();
        let stencil_maps = self.world.read_component::<StencilMap>();
        let root = self.world.read_resource::<Root>().0?;
        let song = songs.get(root)?;

        let mut pdf = Pdf::new();
        let scale = song.scale();
        pdf.add_page(Size::new(215.9, 279.4));
        if let Some(embed_file) = embed_file {
            pdf.add_file(&embed_file);
        }

        pdf.write_stencil_map(
            stencil_maps.get(root)?,
            Affine::translate(Vec2::new(0.0, 279.4)) * Affine::scale(scale) * Affine::FLIP_Y,
            &stencils,
            &stencil_maps,
        );
        Some(base64::encode(pdf.into_binary()))
    }
}
//
// #[cfg(test)]
// mod tests {
//     use super::*;
//
//     #[test]
//     fn it_works() {
//         use rhythm::NoteValue;
//         use stencil::snapshot;
//
//         let mut render = Render::default();
//         let song = render.song_create();
//         render.song_set_size(song, 215.9, 279.4);
//         render.song_set_title(song, "Six Eight", 26.4f64);
//         render.song_set_author(song, "Six Eight", 26.4f64 * 5f64 / 7f64);
//
//         let staff = render.staff_create();
//         let clef =
//             render.between_bars_create(None, Some(Clef::Percussion), Some(4), Some(4), Some(0));
//         render.child_append(staff, clef);
//
//         let bar1 = render.bar_create(4, 4);
//         render.child_append(staff, bar1);
//
//         let chord1 = render.chord_create(NoteValue::Eighth.log2() as isize, 0, 1, 8);
//         render.chord_set_unpitched(chord1);
//
//         render.bar_insert(bar1, chord1, false);
//         let barline = render.between_bars_create(Some(Barline::Normal), None, None, None, Some(0));
//         render.child_append(staff, barline);
//
//         let bar2 = render.bar_create(4, 4);
//         render.child_append(staff, bar2);
//
//         let chord2 = render.chord_create(NoteValue::SixtyFourth.log2() as isize, 0, 1, 4);
//         render.chord_set_unpitched(chord2);
//
//         render.bar_insert(bar2, chord2, false);
//
//         let final_barline =
//             render.between_bars_create(Some(Barline::Final), None, None, None, Some(0));
//         render.child_append(staff, final_barline);
//
//         render.child_append(song, staff);
//         render.root_set(song);
//
//         render.exec();
//
//         render.chord_update_time(chord1, NoteValue::Eighth.log2() as isize, 0, 4, 16, false);
//
//         snapshot("./snapshots/hello_world.svg", &render.print_for_demo());
//
//         // Make sure we can clean up and no entities are left over.
//         render.root_clear(song);
//         render.exec();
//
//         assert_eq!(render.parents.len(), 0);
//         assert_eq!(render.songs.len(), 0);
//         assert_eq!(render.staffs.len(), 0);
//         assert_eq!(render.line_of_staffs.len(), 0);
//         assert_eq!(render.bars.len(), 0);
//         assert_eq!(render.between_bars.len(), 0);
//         assert_eq!(render.chords.len(), 0);
//         assert_eq!(render.stencils.len(), 0);
//         assert_eq!(render.stencil_maps.len(), 0);
//         assert_eq!(render.world_bbox.len(), 0);
//         assert_eq!(render.contexts.len(), 0);
//         assert_eq!(render.spacing.len(), 0);
//         assert_eq!(render.ordered_children.len(), 0);
//     }
//
//     #[test]
//     fn beaming_1() {
//         use rhythm::NoteValue;
//         use stencil::snapshot;
//
//         let mut render = Render::default();
//         let song = render.song_create();
//         render.song_set_size(song, 215.9, 279.4);
//         render.song_set_title(song, "Six Eight", 26.4f64);
//         render.song_set_author(song, "Six Eight", 26.4f64 * 5f64 / 7f64);
//
//         let staff = render.staff_create();
//         let clef = render.between_bars_create(None, Some(Clef::G), Some(4), Some(4), Some(0));
//         render.child_append(staff, clef);
//
//         let bar1 = render.bar_create(4, 4);
//         render.child_append(staff, bar1);
//
//         let chord1 = render.chord_create(NoteValue::Eighth.log2() as isize, 0, 0, 1);
//         let chord2 = render.chord_create(NoteValue::Eighth.log2() as isize, 0, 1, 8);
//
//         render.chord_set_pitch(chord1, 60, 0);
//         render.bar_insert(bar1, chord1, false);
//         render.chord_set_pitch(chord2, 60, 0);
//         render.bar_insert(bar1, chord2, false);
//
//         let final_barline =
//             render.between_bars_create(Some(Barline::Final), None, None, None, Some(0));
//         render.child_append(staff, final_barline);
//
//         render.child_append(song, staff);
//         render.root_set(song);
//
//         render.exec();
//
//         snapshot("./snapshots/beaming_1.svg", &render.print_for_demo());
//     }
// }
