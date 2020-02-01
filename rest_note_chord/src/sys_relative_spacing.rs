use num_rational::Rational;
use std::collections::HashMap;

use crate::RestNoteChord;
use entity::{Entity, Join};
use rhythm::{Bar, RelativeRhythmicSpacing};
use stencil::Stencil;

pub fn sys_relative_spacing(
    rnc: &HashMap<Entity, RestNoteChord>,
    parents: &HashMap<Entity, Entity>,
    bars: &HashMap<Entity, Bar>,
    stencils: &HashMap<Entity, Stencil>,
    spacing: &mut HashMap<Entity, RelativeRhythmicSpacing>,
) {
    let mut shortest_per_bar: HashMap<Entity, Rational> = HashMap::new();

    for (_id, (rnc, parent)) in (rnc, parents).join() {
        let dur = rnc.duration.duration();
        let entry = shortest_per_bar
            .entry(*parent)
            .or_insert(Rational::new(1, 8));
        *entry = (*entry).min(dur);
    }

    for (id, (rnc, parent)) in (rnc, parents).join() {
        *spacing.get_mut(&id).unwrap() =
            RelativeRhythmicSpacing::new(shortest_per_bar[&parent], &rnc.duration);
    }

    for bar_id in shortest_per_bar.keys() {
        if let Some(bar) = bars.get(bar_id) {
            let mut advance_step = 0.0f64;
            for (_, _, entity, _) in bar.children() {
                let stencil = &stencils[&entity];
                let relative_spacing = spacing[&entity];
                advance_step = advance_step.max(stencil.rect().x1 / relative_spacing.relative());
            }

            let mut width_with_advance_step = 0.0;
            for (_, _, entity, _) in bar.children() {
                let relative_spacing = spacing[&entity];
                width_with_advance_step += advance_step * relative_spacing.relative();
            }

            // TODO: this should only apply when we are editing a bar.
            const MIN_WIDTH: f64 = 3000.0;
            if width_with_advance_step < MIN_WIDTH {
                advance_step *= MIN_WIDTH / width_with_advance_step;
            }
            drop(width_with_advance_step);

            let advance_step = advance_step + 100.0; // freeze

            let start = 200f64;
            let mut advance = start;
            for (_, t, entity, _) in bar.children() {
                let relative_spacing = spacing[&entity];
                let end = advance + advance_step * relative_spacing.relative();

                let rnc = &rnc[&entity];
                let spacing = spacing.get_mut(&entity).unwrap();

                *spacing = RelativeRhythmicSpacing::new(shortest_per_bar[&bar_id], &rnc.duration);
                spacing.t = t;
                spacing.start_x = advance;
                spacing.end_x = end;

                advance = end;
            }
        }
    }
}
