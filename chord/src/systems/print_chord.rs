use kurbo::Point;
use stencil::components::Stencil;

use crate::components::{Chord, Context};
use specs::Entity;
use std::collections::HashMap;

pub fn sys_print_chord(
    chord: &HashMap<Entity, Chord>,
    contexts: &HashMap<Entity, Context>,
    beam_for_chord: &HashMap<Entity, Entity>,
    attachments: &mut HashMap<Entity, Option<Point>>,
    stencils: &mut HashMap<Entity, Stencil>,
) {
    for (id, (chord, context, stencil)) in (chord, contexts, stencils).join() {
        let has_beam = beam_for_chord.contains_key(&id);
        let result = chord.print(context, has_beam);
        *stencil = result.0;
        *attachments.entry(id).or_default() = result.1;
    }
}
