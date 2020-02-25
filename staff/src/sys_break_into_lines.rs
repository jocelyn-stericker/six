use num_rational::Rational;

use crate::{BetweenBars, LineOfStaff, Staff};
use entity::{EntitiesRes, Entity, Join};
use rhythm::{Bar, Duration, RelativeRhythmicSpacing};
use std::collections::HashMap;
use stencil::Stencil;

pub(crate) const STAFF_MARGIN: f64 = 2500f64;

#[derive(Debug, Clone)]
struct BetweenMeta {
    /// Stencil and width if at start of line.
    start: (Entity, f64),
    /// Stencil and width if in middle of line.
    mid: (Entity, f64),
    /// Stencil and width if at end of line.
    end: (Entity, f64),
}

#[derive(Debug, Clone)]
/// Line-splitting metadata for notes and betweens.
enum ItemMeta {
    Note(Duration, Entity, f64),
    Between(BetweenMeta),
}

impl ItemMeta {
    fn start_meta(&self) -> (Entity, f64) {
        match self {
            ItemMeta::Note(_, stencil, width) => (*stencil, *width),
            ItemMeta::Between(bm) => bm.start,
        }
    }

    fn mid_meta(&self) -> (Entity, f64) {
        match self {
            ItemMeta::Note(_, stencil, width) => (*stencil, *width),
            ItemMeta::Between(bm) => bm.mid,
        }
    }

    fn end_meta(&self) -> (Entity, f64) {
        match self {
            ItemMeta::Note(_, stencil, width) => (*stencil, *width),
            ItemMeta::Between(bm) => bm.end,
        }
    }

    fn duration(&self) -> Option<Duration> {
        match self {
            ItemMeta::Note(duration, _, _) => Some(*duration),
            ItemMeta::Between(_) => None,
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
    fn add_bar(&mut self, entity: Entity, bar: &Bar, stencils: &HashMap<Entity, Stencil>) {
        self.entities.push(ConditionalChildren {
            start: entity,
            mid: entity,
            end: entity,
        });
        for (duration, _, entity, _) in bar.children() {
            let stencil = &stencils[&entity];
            self.shortest = self.shortest.min(duration.duration());
            self.children
                .push(ItemMeta::Note(duration, entity, stencil.rect().x1));
        }

        let mut advance_step = 400.0f64;
        for meta in &self.children {
            if let Some(ref duration) = meta.duration() {
                advance_step = advance_step.max(
                    meta.mid_meta().1
                        / RelativeRhythmicSpacing::new(self.shortest, duration).relative,
                );
            }
        }

        let advance_step = advance_step + 100.0;

        self.width = 0.0;
        for (i, meta) in self.children.iter().enumerate() {
            if let Some(ref duration) = meta.duration() {
                self.width +=
                    advance_step * RelativeRhythmicSpacing::new(self.shortest, duration).relative;
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

    fn add_between(&mut self, between: &BetweenBars, stencils: &HashMap<Entity, Stencil>) {
        self.entities.push(ConditionalChildren {
            start: between.stencil_start,
            mid: between.stencil_middle,
            end: between.stencil_end,
        });

        self.children.push(ItemMeta::Between(BetweenMeta {
            start: (
                between.stencil_start,
                stencils[&between.stencil_start].advance(),
            ),
            mid: (
                between.stencil_middle,
                stencils[&between.stencil_middle].advance(),
            ),
            end: (
                between.stencil_end,
                stencils[&between.stencil_end].advance(),
            ),
        }));
        let w = if self.entities.len() == 1 {
            stencils[&between.stencil_start].advance()
        } else {
            // TODO: should be end, but back to middle when adding another bar.
            stencils[&between.stencil_middle].advance()
        };
        self.width += w;
        self.is_valid = true;
    }

    // TODO(joshuan): This should just be bar widths, and spacing within a bar should be calculated
    // somewhere else.
    fn apply_spacing(
        &self,
        width: f64,
        bars: &HashMap<Entity, Bar>,
        spacing: &mut HashMap<Entity, RelativeRhythmicSpacing>,
    ) {
        let mut advance_step = 400.0f64;
        for meta in &self.children {
            if let Some(ref duration) = meta.duration() {
                advance_step = advance_step.max(
                    meta.mid_meta().1
                        / RelativeRhythmicSpacing::new(self.shortest, duration).relative,
                );
            }
        }

        advance_step += 100.0;

        let mut spring_width = 0.0;
        let mut strut_width = 0.0;
        let mut advances = 0.0;

        for (i, meta) in self.children.iter().enumerate() {
            if let Some(ref duration) = meta.duration() {
                spring_width +=
                    advance_step * RelativeRhythmicSpacing::new(self.shortest, duration).relative;
                advances += RelativeRhythmicSpacing::new(self.shortest, duration).relative;
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
            if let Some(bar) = bars.get(&maybe_bar.mid) {
                let mut advance = 200f64;
                for (dur, t, entity, _) in bar.children() {
                    let mut my_spacing = RelativeRhythmicSpacing::new(self.shortest, &dur);
                    my_spacing.t = t;
                    my_spacing.start_x = advance;
                    my_spacing.end_x = advance + advance_step * my_spacing.relative();

                    advance = my_spacing.end_x;

                    spacing.insert(entity, my_spacing);
                }
            }
        }
    }
}

pub fn sys_break_into_lines(
    entities: &EntitiesRes,
    page_size: Option<(f64, f64)>,
    bars: &HashMap<Entity, Bar>,
    between_bars: &HashMap<Entity, BetweenBars>,
    stencils: &HashMap<Entity, Stencil>,
    spacing: &mut HashMap<Entity, RelativeRhythmicSpacing>,
    staffs: &mut HashMap<Entity, Staff>,
    parents: &mut HashMap<Entity, Entity>,
    ordered_children: &mut HashMap<Entity, Vec<Entity>>,
    line_of_staffs: &mut HashMap<Entity, LineOfStaff>,
) {
    if page_size.is_none() {
        return;
    }

    let width = page_size.unwrap().0 - STAFF_MARGIN * 2f64;

    let mut to_add = vec![];
    for (staff_id, (staff, children)) in (staffs, &mut *ordered_children).join() {
        let mut chunks: Vec<Vec<ConditionalChildren>> = Vec::new();
        let mut current_solution = PartialSolution::default();
        let mut next_solution = PartialSolution::default();
        let mut good_solution = PartialSolution::default();
        let mut recent_between = None;

        // This is greedy.
        for child in children {
            if let Some(bar) = bars.get(child) {
                current_solution.add_bar(*child, bar, stencils);
                next_solution.add_bar(*child, bar, stencils);
            } else if let Some(between) = between_bars.get(child) {
                current_solution.add_between(between, &stencils);
                next_solution.add_between(between, &stencils);
                recent_between = Some(between);
            } else {
                panic!();
            }

            if current_solution.is_valid {
                if current_solution.width < width {
                    good_solution = current_solution.clone();
                    next_solution = PartialSolution::default();
                    if let Some(between) = recent_between {
                        next_solution.add_between(between, &stencils);
                    }
                } else {
                    good_solution.apply_spacing(width, bars, spacing);
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
            current_solution.apply_spacing(current_solution.width + extra_space, bars, spacing);
            chunks.push(current_solution.entities);
        }

        while staff.lines.len() > chunks.len() {
            staff.lines.pop();
        }

        for (line_number, line) in chunks.into_iter().enumerate() {
            if staff.lines.len() == line_number {
                // This is a line of Staff.
                let line_of_staff_id = entities.create();
                // This is the 5 staff lines for the line of Staff.
                let staff_lines_id = entities.create();

                parents.insert(staff_lines_id, line_of_staff_id);

                line_of_staffs.insert(line_of_staff_id, LineOfStaff::new(staff_lines_id));

                staff.lines.push(line_of_staff_id);
                parents.insert(line_of_staff_id, staff_id);
            }

            let line_len = line.len();
            to_add.push((
                staff.lines[line_number],
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
            ));
        }
    }

    for (entity, val) in to_add {
        ordered_children.insert(entity, val);
    }
}
