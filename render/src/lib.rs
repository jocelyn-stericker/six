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

    pub fn remove_staff(&mut self, staff: usize) {
        let staff = Entity::new(staff);

        self.staffs.remove(&staff);
        self.stencil_maps.remove(&staff);
    }

    pub fn create_bar(&mut self, numer: u8, denom: u8) -> Option<usize> {
        let entity = self.entities.create();

        self.bars.insert(entity, Bar::new(Metre::new(numer, denom)));
        self.stencil_maps.insert(entity, StencilMap::default());

        Some(entity.id())
    }

    pub fn append_to_staff(&mut self, staff: usize, child: usize) {
        let child = Entity::new(child);
        if let Some(staff) = self.staffs.get_mut(&Entity::new(staff)) {
            staff.children.push(child);
        }
    }

    pub fn insert_to_staff_before(&mut self, staff: usize, before: usize, child: usize) {
        let before = Entity::new(before);
        let child = Entity::new(child);
        if let Some(staff) = self.staffs.get_mut(&Entity::new(staff)) {
            if let Some(idx) = staff.children.iter().position(|&x| x == before) {
                staff.children.insert(idx, child);
            }
        }
    }

    pub fn remove_bar(&mut self, staff: usize, bar: usize) {
        let bar = Entity::new(bar);
        let staff = Entity::new(staff);

        if let Some(staff) = self.staffs.get_mut(&staff) {
            if let Some(bar_idx) = staff.children.iter().position(|&x| x == bar) {
                staff.children.remove(bar_idx);
            }
        }
    }

    pub fn create_barline(&mut self, barline: Barline) -> Option<usize> {
        let entity = self.entities.create();

        self.between_bars.insert(
            entity,
            BetweenBars {
                barline: Some(barline),
                clef: false,
            },
        );
        self.stencils.insert(entity, Stencil::default());

        Some(entity.id())
    }

    pub fn create_clef(&mut self) -> Option<usize> {
        let entity = self.entities.create();

        self.between_bars.insert(
            entity,
            BetweenBars {
                barline: None,
                clef: true,
            },
        );
        self.stencils.insert(entity, Stencil::default());

        Some(entity.id())
    }

    pub fn remove_between_bar(&mut self, staff: usize, between_bar: usize) {
        let between_bar = Entity::new(between_bar);
        let staff = Entity::new(staff);

        if let Some(staff) = self.staffs.get_mut(&staff) {
            if let Some(bar_idx) = staff.children.iter().position(|&x| x == between_bar) {
                staff.children.remove(bar_idx);
            }
        }
    }

    pub fn create_rnc(
        &mut self,
        note_value: isize,
        dots: u8,
        _start: &[isize],
        is_note: bool,
    ) -> Option<usize> {
        let note_value = NoteValue::new(note_value).unwrap();

        let entity = self.entities.create();

        self.spacing
            .insert(entity, RelativeRhythmicSpacing::default());
        self.rncs.insert(
            entity,
            RestNoteChord::new(Duration::new(note_value, dots, None), is_note),
        );
        self.stencils.insert(entity, Stencil::default());

        Some(entity.id())
    }

    pub fn append_rnc(&mut self, bar: usize, entity: usize, start: &[isize]) {
        let entity = Entity::new(entity);
        let start = Rational::new(start[0], start[1]);

        if let Some(bar) = self.bars.get_mut(&Entity::new(bar)) {
            if let Some(rnc) = self.rncs.get(&entity) {
                bar.splice(start, vec![(rnc.duration(), Some(entity))]);
            }
        }
    }

    pub fn remove_rnc(&mut self, bar: usize, rnc: usize) {
        let bar = Entity::new(bar);
        let rnc = Entity::new(rnc);

        if let Some(bar) = self.bars.get_mut(&bar) {
            bar.remove(rnc);
        }
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
        let clef = render.create_clef().unwrap();
        render.append_to_staff(staff_entity, clef);

        let bar1_entity = render.create_bar(4, 4).unwrap();
        render.append_to_staff(staff_entity, bar1_entity);

        let rnc1 = render
            .create_rnc(NoteValue::Eighth.log2() as isize, 0, &[1, 4], true)
            .unwrap();

        render.append_rnc(bar1_entity, rnc1, &[1, 4]);
        let barline = render.create_barline(Barline::Normal).unwrap();
        render.append_to_staff(staff_entity, barline);

        let bar2_entity = render.create_bar(4, 4).unwrap();
        render.append_to_staff(staff_entity, bar2_entity);

        let rnc2 = render
            .create_rnc(NoteValue::Eighth.log2() as isize, 0, &[1, 4], true)
            .unwrap();

        render.append_rnc(bar2_entity, rnc2, &[1, 4]);

        let final_barline = render.create_barline(Barline::Final).unwrap();
        render.append_to_staff(staff_entity, final_barline);

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
