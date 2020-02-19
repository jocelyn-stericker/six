use num_rational::Rational;

use crate::{LineOfStaff, Staff};
use entity::{EntitiesRes, Entity, Join};
use rhythm::{Bar, Duration, RelativeRhythmicSpacing};
use std::collections::HashMap;
use stencil::Stencil;

pub(crate) const STAFF_MARGIN: f64 = 2500f64;

#[derive(Debug, Clone)]
struct PartialSolution {
    shortest: Rational,
    entities: Vec<Entity>,
    children: Vec<(Entity, Option<Duration>, f64)>,
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
        self.entities.push(entity);
        for (duration, _, entity, _) in bar.children() {
            let stencil = &stencils[&entity];
            self.shortest = self.shortest.min(duration.duration());
            self.children
                .push((entity, Some(duration), stencil.rect().x1));
        }

        let mut advance_step = 400.0f64;
        for (_, time, min_width) in &self.children {
            if let Some(time) = time {
                advance_step = advance_step
                    .max(min_width / RelativeRhythmicSpacing::new(self.shortest, time).relative);
            }
        }

        let advance_step = advance_step + 100.0;

        self.width = 0.0;
        for (_, time, min_width) in &self.children {
            if let Some(time) = time {
                self.width +=
                    advance_step * RelativeRhythmicSpacing::new(self.shortest, time).relative;
            } else {
                self.width += min_width;
            }
        }

        self.is_valid = false;
    }

    fn add_between(&mut self, entity: Entity, between: &Stencil) {
        self.entities.push(entity);

        let w = between.advance();
        self.children.push((entity, None, w));
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
        for (_, duration, min_width) in &self.children {
            if let Some(duration) = duration {
                advance_step = advance_step.max(
                    min_width / RelativeRhythmicSpacing::new(self.shortest, duration).relative,
                );
            }
        }

        advance_step += 100.0;

        let mut spring_width = 0.0;
        let mut strut_width = 0.0;
        let mut advances = 0.0;

        for (i, (_, duration, min_width)) in self.children.iter().enumerate() {
            if let Some(duration) = duration {
                spring_width +=
                    advance_step * RelativeRhythmicSpacing::new(self.shortest, duration).relative;
                advances += RelativeRhythmicSpacing::new(self.shortest, duration).relative;
            } else {
                if i != 0 {
                    // HACK: padding after bar.
                    strut_width += 200f64;
                }
                strut_width += min_width;
            }
        }

        let extra_width_to_allocate = width - spring_width - strut_width;

        advance_step += extra_width_to_allocate / advances;

        for maybe_bar in &self.entities {
            if let Some(bar) = bars.get(maybe_bar) {
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
        let mut chunks: Vec<Vec<Entity>> = Vec::new();
        let mut current_solution = PartialSolution::default();
        let mut next_solution = PartialSolution::default();
        let mut good_solution = PartialSolution::default();

        // This is greedy.
        for child in children {
            if let Some(bar) = bars.get(child) {
                current_solution.add_bar(*child, bar, stencils);
                next_solution.add_bar(*child, bar, stencils);
            } else if let Some(stencil) = stencils.get(child) {
                current_solution.add_between(*child, stencil);
                next_solution.add_between(*child, stencil);
            } else {
                panic!();
            }

            if current_solution.is_valid {
                eprintln!("{:?} {:?}", current_solution.width, width);
                if current_solution.width < width {
                    good_solution = current_solution.clone();
                    next_solution = PartialSolution::default();
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

            to_add.push((staff.lines[line_number], line));
        }
    }

    for (entity, val) in to_add {
        ordered_children.insert(entity, val);
    }
}
