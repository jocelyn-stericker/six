use kurbo::Point;
use stencil::Stencil;

use crate::context::Context;
use crate::rest_note_chord::RestNoteChord;
use entity::{Entity, Join};
use std::collections::HashMap;

pub fn sys_print_rnc(
    rnc: &HashMap<Entity, RestNoteChord>,
    contexts: &HashMap<Entity, Context>,
    beam_for_rnc: &HashMap<Entity, Entity>,
    attachments: &mut HashMap<Entity, Option<Point>>,
    stencils: &mut HashMap<Entity, Stencil>,
) {
    for (id, (rnc, context, stencil)) in (rnc, contexts, stencils).join() {
        let has_beam = beam_for_rnc.contains_key(&id);
        let result = rnc.print(context, has_beam);
        *stencil = result.0;
        *attachments.entry(id).or_default() = result.1;
    }
}

