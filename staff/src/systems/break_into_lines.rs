#![allow(clippy::type_complexity)]

use num_rational::Rational;

use crate::{
    components::{Children, LineOfStaff, Signature, Song, Staff},
    resources::{KeepSpacing, Root},
};
use rhythm::{components::Bar, components::Spacing, BarChild, Duration};
use specs::{Entities, Entity, Join, Read, ReadStorage, System, WriteStorage};
use stencil::components::{Parent, Stencil};

#[derive(Debug, Default)]
pub struct BreakIntoLines;

impl<'a> System<'a> for BreakIntoLines {
    type SystemData = (
        Entities<'a>,
        Read<'a, Root>,
        Read<'a, KeepSpacing>,
        ReadStorage<'a, Song>,
        ReadStorage<'a, Bar>,
        ReadStorage<'a, Signature>,
        ReadStorage<'a, Stencil>,
        WriteStorage<'a, Spacing>,
        WriteStorage<'a, Staff>,
        WriteStorage<'a, Parent>,
        WriteStorage<'a, Children>,
        WriteStorage<'a, LineOfStaff>,
    );

    fn run(
        &mut self,
        (
            entities,
            root,
            keep_spacing,
            songs,
            bars,
            signatures,
            stencils,
            mut spacings,
            mut staffs,
            mut parents,
            mut children,
            mut line_of_staffs,
        ): Self::SystemData,
    ) {
        if keep_spacing.0 {
            return;
        }

        let song = root.0.and_then(|root| songs.get(root));

        // TODO(joshuan): scale is fixed as rastal size 3.
        let width = song.map(|song| song.width / 7.0 * 1000.0).unwrap_or(0.0) - STAFF_MARGIN * 2f64;

        let mut to_add = vec![];
        for (id, staff, children) in (&entities, &mut staffs, &mut children).join() {
            let mut chunks: Vec<Vec<ConditionalChildren>> = Vec::new();
            let mut current_solution = PartialSolution::default();
            let mut next_solution = PartialSolution::default();
            let mut good_solution = PartialSolution::default();
            let mut recent_signature = None;

            // This is greedy.
            for &child in &children.0 {
                if let Some(bar) = bars.get(child) {
                    current_solution.add_bar(child, bar, &stencils);
                    next_solution.add_bar(child, bar, &stencils);
                } else if let Some(signature) = signatures.get(child) {
                    current_solution.add_signature(signature, &stencils);
                    next_solution.add_signature(signature, &stencils);
                    recent_signature = Some(signature);
                } else {
                    panic!();
                }

                if current_solution.is_valid {
                    if current_solution.width < width {
                        good_solution = current_solution.clone();
                        next_solution = PartialSolution::default();
                        if let Some(signature) = recent_signature {
                            next_solution.add_signature(signature, &stencils);
                        }
                    } else {
                        good_solution.apply_spacing(width, &bars, &mut spacings);
                        let PartialSolution { entities, .. } = good_solution;
                        current_solution = next_solution.clone();
                        good_solution = PartialSolution::default();

                        if !entities.is_empty() {
                            chunks.push(entities);
                        }
                    }
                }
            }

            if !current_solution.entities.is_empty() {
                // Pad the spacing a bit.
                let extra_space = (width - current_solution.width) / 8f64;
                current_solution.apply_spacing(
                    current_solution.width + extra_space,
                    &bars,
                    &mut spacings,
                );
                chunks.push(current_solution.entities);
            }

            while staff.lines.len() > chunks.len() {
                staff.lines.pop();
            }

            for (line_number, line) in chunks.into_iter().enumerate() {
                if staff.lines.len() == line_number {
                    // This is the 5 staff lines for the line of Staff.
                    let staff_lines = entities.create();

                    // This is a line of Staff.
                    let line_of_staff = entities
                        .build_entity()
                        .with(LineOfStaff::new(staff_lines), &mut line_of_staffs)
                        .with(Parent(id), &mut parents)
                        .build();

                    parents
                        .insert(staff_lines, Parent(line_of_staff))
                        .expect("Could not get init staff line entity");

                    staff.lines.push(line_of_staff);
                }

                let line_len = line.len();
                to_add.push((
                    staff.lines[line_number],
                    Children(
                        line.into_iter()
                            .enumerate()
                            .map(|(i, cond)| {
                                if i == 0 {
                                    cond.start
                                } else if i + 1 == line_len {
                                    cond.end
                                } else {
                                    cond.mid
                                }
                            })
                            .collect(),
                    ),
                ));
            }
        }

        for (entity, val) in to_add {
            children.insert(entity, val).unwrap();
        }
    }
}

pub(crate) const STAFF_MARGIN: f64 = 2500f64;

#[derive(Debug, Clone)]
struct SignatureMeta {
    /// Stencil and width if at start of line.
    start: (Entity, f64),
    /// Stencil and width if in middle of line.
    mid: (Entity, f64),
    /// Stencil and width if at end of line.
    end: (Entity, f64),
}

#[derive(Debug, Clone)]
/// Line-splitting metadata for notes and signatures.
enum ItemMeta {
    Note(Duration, Entity, f64),
    Signature(SignatureMeta),
}

impl ItemMeta {
    fn start_meta(&self) -> (Entity, f64) {
        match self {
            ItemMeta::Note(_, stencil, width) => (*stencil, *width),
            ItemMeta::Signature(bm) => bm.start,
        }
    }

    fn mid_meta(&self) -> (Entity, f64) {
        match self {
            ItemMeta::Note(_, stencil, width) => (*stencil, *width),
            ItemMeta::Signature(bm) => bm.mid,
        }
    }

    fn end_meta(&self) -> (Entity, f64) {
        match self {
            ItemMeta::Note(_, stencil, width) => (*stencil, *width),
            ItemMeta::Signature(bm) => bm.end,
        }
    }

    fn duration(&self) -> Option<Duration> {
        match self {
            ItemMeta::Note(duration, _, _) => Some(*duration),
            ItemMeta::Signature(_) => None,
        }
    }
}

#[derive(Debug, Clone)]
struct ConditionalChildren {
    start: Entity,
    mid: Entity,
    end: Entity,
}

#[derive(Debug, Clone)]
struct PartialSolution {
    shortest: Rational,
    entities: Vec<ConditionalChildren>,
    children: Vec<ItemMeta>,
    width: f64,
    is_valid: bool,
}

impl Default for PartialSolution {
    fn default() -> PartialSolution {
        PartialSolution {
            shortest: Rational::new(1, 8),
            entities: vec![],
            children: vec![],
            width: 0f64,
            is_valid: true,
        }
    }
}

impl PartialSolution {
    fn add_bar(&mut self, entity: Entity, bar: &Bar, stencils: &ReadStorage<Stencil>) {
        self.entities.push(ConditionalChildren {
            start: entity,
            mid: entity,
            end: entity,
        });
        for BarChild {
            duration, stencil, ..
        } in bar.children()
        {
            let stencil = &stencils.get(stencil).unwrap();
            self.shortest = self.shortest.min(duration.duration());
            self.children
                .push(ItemMeta::Note(duration, entity, stencil.rect().x1));
        }

        let mut advance_step = 400.0f64;
        for meta in &self.children {
            if let Some(ref duration) = meta.duration() {
                advance_step = advance_step
                    .max(meta.mid_meta().1 / Spacing::new(self.shortest, duration).relative);
            }
        }

        let advance_step = advance_step + 100.0;

        self.width = 0.0;
        for (i, meta) in self.children.iter().enumerate() {
            if let Some(ref duration) = meta.duration() {
                self.width += advance_step * Spacing::new(self.shortest, duration).relative;
            } else {
                self.width += if i == 0 {
                    meta.start_meta().1
                } else {
                    meta.mid_meta().1
                };
            }
        }

        self.is_valid = false;
    }

    fn add_signature(&mut self, signature: &Signature, stencils: &ReadStorage<Stencil>) {
        self.entities.push(ConditionalChildren {
            start: signature.stencil_start,
            mid: signature.stencil_middle,
            end: signature.stencil_end,
        });

        self.children.push(ItemMeta::Signature(SignatureMeta {
            start: (
                signature.stencil_start,
                stencils.get(signature.stencil_start).unwrap().advance(),
            ),
            mid: (
                signature.stencil_middle,
                stencils.get(signature.stencil_middle).unwrap().advance(),
            ),
            end: (
                signature.stencil_end,
                stencils.get(signature.stencil_end).unwrap().advance(),
            ),
        }));
        let w = if self.entities.len() == 1 {
            stencils.get(signature.stencil_start).unwrap().advance()
        } else {
            // TODO: should be end, but back to middle when adding another bar.
            stencils.get(signature.stencil_middle).unwrap().advance()
        };
        self.width += w;
        self.is_valid = true;
    }

    // TODO(joshuan): This should just be bar widths, and spacing within a bar should be calculated
    // somewhere else.
    fn apply_spacing(
        &self,
        width: f64,
        bars: &ReadStorage<Bar>,
        spacing: &mut WriteStorage<Spacing>,
    ) {
        let mut advance_step = 400.0f64;
        for meta in &self.children {
            if let Some(ref duration) = meta.duration() {
                advance_step = advance_step
                    .max(meta.mid_meta().1 / Spacing::new(self.shortest, duration).relative);
            }
        }

        advance_step += 100.0;

        let mut spring_width = 0.0;
        let mut strut_width = 0.0;
        let mut advances = 0.0;

        for (i, meta) in self.children.iter().enumerate() {
            if let Some(ref duration) = meta.duration() {
                spring_width += advance_step * Spacing::new(self.shortest, duration).relative;
                advances += Spacing::new(self.shortest, duration).relative;
            } else if i == 0 {
                strut_width += meta.start_meta().1;
            } else if i + 1 == self.children.len() {
                // HACK: padding after bar.
                strut_width += 200f64 + meta.end_meta().1;
            } else {
                // HACK: padding after bar.
                strut_width += 200f64 + meta.mid_meta().1;
            }
        }

        let extra_width_to_allocate = width - spring_width - strut_width;

        advance_step += extra_width_to_allocate / advances;

        for maybe_bar in &self.entities {
            if let Some(bar) = bars.get(maybe_bar.mid) {
                let mut advance = 200f64;
                for BarChild {
                    duration,
                    start,
                    stencil,
                    ..
                } in bar.children()
                {
                    let mut my_spacing = Spacing::new(self.shortest, &duration);
                    my_spacing.t = start;
                    my_spacing.start_x = advance;
                    my_spacing.end_x = advance + advance_step * my_spacing.relative();

                    advance = my_spacing.end_x;

                    spacing.insert(stencil, my_spacing).unwrap();
                }
            }
        }
    }
}
