#![allow(clippy::blacklisted_name)]

use entity::{EntitiesRes, Entity};
use num_rational::Rational;
use rest_note_chord::{sys_implicit_rests, sys_print_rnc, sys_relative_spacing, RestNoteChord};
use rhythm::{Bar, Duration, Metre, NoteValue, RelativeRhythmicSpacing};
use staff::{
    sys_print_between_bars, sys_print_staff, sys_print_staff_lines, Barline, BetweenBars, Staff,
};
use std::collections::HashMap;
use stencil::{Stencil, StencilMap};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
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

#[wasm_bindgen]
impl Render {
    pub fn new() -> Render {
        Self::default()
    }

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

    pub fn append_staff(&mut self) -> Option<usize> {
        let entity = self.entities.create();

        self.staffs.insert(entity, Staff::default());
        self.stencil_maps.insert(entity, StencilMap::default());

        Some(entity.id())
    }

    pub fn append_bar(&mut self, staff: usize, numer: u8, denom: u8) -> Option<usize> {
        let staff = self.staffs.get_mut(&Entity::new(staff))?;
        let entity = self.entities.create();

        self.bars.insert(entity, Bar::new(Metre::new(numer, denom)));
        self.stencil_maps.insert(entity, StencilMap::default());
        staff.children.push(entity);

        Some(entity.id())
    }

    pub fn insert_bar_before(
        &mut self,
        staff: usize,
        before: usize,
        numer: u8,
        denom: u8,
    ) -> Option<usize> {
        let before = Entity::new(before);
        let staff = self.staffs.get_mut(&Entity::new(staff))?;
        let idx = staff.children.iter().position(|&x| x == before)?;

        let entity = self.entities.create();

        self.bars.insert(entity, Bar::new(Metre::new(numer, denom)));
        self.stencil_maps.insert(entity, StencilMap::default());
        staff.children.insert(idx, entity);

        Some(entity.id())
    }

    pub fn append_clef(&mut self, staff: usize) -> Option<usize> {
        let staff = self.staffs.get_mut(&Entity::new(staff))?;
        let entity = self.entities.create();

        self.between_bars.insert(
            entity,
            BetweenBars {
                barline: None,
                clef: true,
            },
        );
        self.stencils.insert(entity, Stencil::default());
        staff.children.push(entity);

        Some(entity.id())
    }

    pub fn insert_clef_before(&mut self, staff: usize, before: usize) -> Option<usize> {
        let before = Entity::new(before);
        let staff = self.staffs.get_mut(&Entity::new(staff))?;
        let idx = staff.children.iter().position(|&x| x == before)?;
        let entity = self.entities.create();

        self.between_bars.insert(
            entity,
            BetweenBars {
                barline: None,
                clef: true,
            },
        );
        self.stencils.insert(entity, Stencil::default());
        staff.children.insert(idx, entity);

        Some(entity.id())
    }

    pub fn append_barline(&mut self, staff: usize, barline: Barline) -> Option<usize> {
        let staff = self.staffs.get_mut(&Entity::new(staff))?;
        let entity = self.entities.create();

        self.between_bars.insert(
            entity,
            BetweenBars {
                barline: Some(barline),
                clef: false,
            },
        );
        self.stencils.insert(entity, Stencil::default());
        staff.children.push(entity);

        Some(entity.id())
    }

    pub fn insert_barline_before(
        &mut self,
        staff: usize,
        before: usize,
        barline: Barline,
    ) -> Option<usize> {
        let before = Entity::new(before);
        let staff = self.staffs.get_mut(&Entity::new(staff))?;
        let idx = staff.children.iter().position(|&x| x == before)?;
        let entity = self.entities.create();

        self.between_bars.insert(
            entity,
            BetweenBars {
                barline: Some(barline),
                clef: false,
            },
        );
        self.stencils.insert(entity, Stencil::default());
        staff.children.insert(idx, entity);

        Some(entity.id())
    }

    pub fn append_rnc(
        &mut self,
        bar: usize,
        note_value: isize,
        dots: u8,
        start: &[isize],
        is_note: bool,
    ) -> Option<usize> {
        let start = Rational::new(start[0], start[1]);
        let note_value = NoteValue::new(note_value).unwrap();
        let bar = self.bars.get_mut(&Entity::new(bar))?;

        let entity = self.entities.create();

        self.spacing
            .insert(entity, RelativeRhythmicSpacing::default());
        self.rncs.insert(
            entity,
            RestNoteChord::new(Duration::new(note_value, dots, None), is_note),
        );
        self.stencils.insert(entity, Stencil::default());

        bar.splice(
            start,
            vec![(Duration::new(note_value, dots, None), Some(entity))],
        );

        Some(entity.id())
    }

    pub fn print_for_demo(&mut self, staff_entity: usize) -> String {
        use kurbo::Vec2;

        self.exec();

        self.stencil_maps
            .get(&Entity::new(staff_entity))
            .unwrap()
            .clone()
            .with_translation(Vec2::new(0.0, -1500.0))
            .with_paper_size(3)
            .to_svg_doc_for_testing(&self.stencil_maps, &self.stencils)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        use kurbo::Vec2;

        use rhythm::NoteValue;
        use stencil::snapshot;

        let mut render = Render::default();
        let staff_entity = render.append_staff().unwrap();
        render.append_clef(staff_entity).unwrap();
        let bar1_entity = render.append_bar(staff_entity, 4, 4).unwrap();
        render.append_rnc(
            bar1_entity,
            NoteValue::Eighth.log2() as isize,
            0,
            &[1, 4],
            true,
        );
        render.append_barline(staff_entity, Barline::Normal);
        let bar2_entity = render.append_bar(staff_entity, 4, 4).unwrap();
        render.append_rnc(
            bar2_entity,
            NoteValue::Eighth.log2() as isize,
            0,
            &[1, 4],
            true,
        );
        render.append_barline(staff_entity, Barline::Final);

        render.exec();

        snapshot(
            "./snapshots/hello_world.svg",
            &render
                .stencil_maps
                .get(&Entity::new(staff_entity))
                .unwrap()
                .clone()
                .with_translation(Vec2::new(0.0, -1500.0))
                .with_paper_size(3)
                .to_svg_doc_for_testing(&render.stencil_maps, &render.stencils),
        );
    }
}
