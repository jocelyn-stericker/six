use std::collections::HashMap;

use crate::RestNoteChord;
use entity::{Entity, Join};
use stencil::Stencil;

pub fn sys_print_rnc(rnc: &HashMap<Entity, RestNoteChord>, render: &mut HashMap<Entity, Stencil>) {
    for (_id, (rnc, render)) in (rnc, render).join() {
        *render = rnc.print();
    }
}
