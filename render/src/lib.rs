use entity::{EntitiesRes, Entity};
use kurbo::Vec2;
use rest_note_chord::{sys_implicit_rests, sys_print_rnc, sys_relative_spacing, RestNoteChord};
use rhythm::{Bar, RelativeRhythmicSpacing};
use std::collections::HashMap;
use stencil::Stencil;

#[derive(Debug, Default)]
pub struct Render {
    entities: EntitiesRes,

    bars: HashMap<Entity, Bar>,
    rncs: HashMap<Entity, RestNoteChord>,
    stencils: HashMap<Entity, Stencil>,
    spacing: HashMap<Entity, RelativeRhythmicSpacing>,
}

impl Render {
    pub fn exec(&mut self) {
        sys_implicit_rests(
            &self.entities,
            &mut self.rncs,
            &mut self.bars,
            &mut self.spacing,
            &mut self.stencils,
        );
        sys_relative_spacing(&self.rncs, &mut self.spacing);
        sys_print_rnc(&self.rncs, &mut self.stencils);

        // TODO: support order :)
        // TODO: add a "container" stencil type
        for (bar_entity, bar) in &self.bars {
            let mut advance_step = 0.0f64;
            for entity in bar.children() {
                let stencil = &self.stencils[&entity];
                let relative_spacing = self.spacing[&entity];
                advance_step = advance_step.max(stencil.rect().x1 / relative_spacing.relative());
            }

            let advance_step = advance_step + 0.1; // freeze

            let mut bar_stencil = Stencil::default();
            let mut advance = 0.2;
            for entity in bar.children() {
                let stencil = &self.stencils[&entity];
                let relative_spacing = self.spacing[&entity];

                bar_stencil =
                    bar_stencil.and(stencil.clone().with_translation(Vec2::new(advance, 0.0)));
                advance += advance_step * relative_spacing.relative();
            }

            bar_stencil = bar_stencil.and(Stencil::staff_line(advance + 0.2));
            self.stencils.insert(*bar_entity, bar_stencil);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        use num_rational::Rational;
        use rhythm::{Duration, Metre, NoteValue};
        use stencil::snapshot;

        let mut render = Render::default();
        let bar_entity = render.entities.create();

        let note_entity = render.entities.create();
        render.spacing.insert(note_entity, Default::default());
        render.rncs.insert(
            note_entity,
            RestNoteChord::new(Duration::new(NoteValue::Eighth, 0, None), true),
        );
        render.stencils.insert(note_entity, Default::default());

        let mut bar = Bar::new(Metre::new(4, 4));
        bar.splice(
            Rational::new(1, 4),
            vec![(Duration::new(NoteValue::Half, 0, None), Some(note_entity))],
        );
        bar.splice(
            Rational::new(3, 8),
            vec![(Duration::new(NoteValue::Eighth, 0, None), None)],
        );

        render.bars.insert(bar_entity, bar);

        render.exec();

        snapshot(
            "./snapshots/hello_world.svg",
            &render
                .stencils
                .get(&bar_entity)
                .unwrap()
                .clone()
                .with_translation(Vec2::new(0.0, -1.5))
                .with_paper_size(3)
                .to_svg_doc_for_testing(),
        );
    }
}
