#![allow(clippy::blacklisted_name)]

use entity::{EntitiesRes, Entity};
use rest_note_chord::{sys_implicit_rests, sys_print_rnc, sys_relative_spacing, RestNoteChord};
use rhythm::{Bar, RelativeRhythmicSpacing};
use staff::{
    sys_print_between_bars, sys_print_staff, sys_print_staff_lines, Barline, BetweenBars, Staff,
};
use std::collections::HashMap;
use stencil::{Stencil, StencilMap};
use wasm_bindgen::prelude::*;

#[derive(Debug, Default)]
pub struct Render {
    entities: EntitiesRes,

    staffs: HashMap<Entity, Staff>,
    bars: HashMap<Entity, Bar>,
    between_bars: HashMap<Entity, BetweenBars>,
    rncs: HashMap<Entity, RestNoteChord>,

    stencils: HashMap<Entity, Stencil>,
    stencil_maps: HashMap<Entity, StencilMap>,
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
        sys_print_between_bars(&self.between_bars, &mut self.stencils);

        sys_print_staff(
            &self.entities,
            &mut self.staffs,
            &self.bars,
            &self.spacing,
            &self.stencils,
            &mut self.stencil_maps,
        );
        sys_print_staff_lines(&self.staffs, &mut self.stencils);
    }
}

#[wasm_bindgen]
pub fn hello_world() -> String {
    use kurbo::Vec2;

    use num_rational::Rational;
    use rhythm::{Duration, Metre, NoteValue};

    let mut render = Render::default();
    let staff_entity = render.entities.create();

    let mut staff = Staff::new();

    {
        let clef_entity = render.entities.create();
        render.stencils.insert(clef_entity, Default::default());
        render.between_bars.insert(
            clef_entity,
            BetweenBars {
                barline: None,
                clef: true,
            },
        );
        staff.children.push(clef_entity);
    }

    {
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
        render.stencil_maps.insert(bar_entity, Default::default());

        staff.children.push(bar_entity);
    }

    {
        let barline_entity = render.entities.create();
        render.stencils.insert(barline_entity, Default::default());
        render.between_bars.insert(
            barline_entity,
            BetweenBars {
                barline: Some(Barline::Normal),
                clef: false,
            },
        );
        staff.children.push(barline_entity);
    }

    {
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
        render.stencil_maps.insert(bar_entity, Default::default());

        staff.children.push(bar_entity);
    }

    {
        let barline_entity = render.entities.create();
        render.stencils.insert(barline_entity, Default::default());
        render.between_bars.insert(
            barline_entity,
            BetweenBars {
                barline: Some(Barline::Final),
                clef: false,
            },
        );
        staff.children.push(barline_entity);
    }

    render.staffs.insert(staff_entity, staff);
    render.stencils.insert(staff_entity, Default::default());
    render.stencil_maps.insert(staff_entity, Default::default());

    render.exec();

    render
        .stencil_maps
        .get(&staff_entity)
        .unwrap()
        .clone()
        .with_translation(Vec2::new(0.0, -1500.0))
        .with_paper_size(3)
        .to_svg_doc_for_testing(&render.stencil_maps, &render.stencils)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        use kurbo::Vec2;

        use num_rational::Rational;
        use rhythm::{Duration, Metre, NoteValue};
        use stencil::snapshot;

        let mut render = Render::default();
        let staff_entity = render.entities.create();

        let mut staff = Staff::new();

        {
            let clef_entity = render.entities.create();
            render.stencils.insert(clef_entity, Default::default());
            render.between_bars.insert(
                clef_entity,
                BetweenBars {
                    barline: None,
                    clef: true,
                },
            );
            staff.children.push(clef_entity);
        }

        {
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
            render.stencil_maps.insert(bar_entity, Default::default());

            staff.children.push(bar_entity);
        }

        {
            let barline_entity = render.entities.create();
            render.stencils.insert(barline_entity, Default::default());
            render.between_bars.insert(
                barline_entity,
                BetweenBars {
                    barline: Some(Barline::Normal),
                    clef: false,
                },
            );
            staff.children.push(barline_entity);
        }

        {
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
            render.stencil_maps.insert(bar_entity, Default::default());

            staff.children.push(bar_entity);
        }

        {
            let barline_entity = render.entities.create();
            render.stencils.insert(barline_entity, Default::default());
            render.between_bars.insert(
                barline_entity,
                BetweenBars {
                    barline: Some(Barline::Final),
                    clef: false,
                },
            );
            staff.children.push(barline_entity);
        }

        render.staffs.insert(staff_entity, staff);
        render.stencils.insert(staff_entity, Default::default());
        render.stencil_maps.insert(staff_entity, Default::default());

        render.exec();

        snapshot(
            "./snapshots/hello_world.svg",
            &render
                .stencil_maps
                .get(&staff_entity)
                .unwrap()
                .clone()
                .with_translation(Vec2::new(0.0, -1500.0))
                .with_paper_size(3)
                .to_svg_doc_for_testing(&render.stencil_maps, &render.stencils),
        );
    }
}
