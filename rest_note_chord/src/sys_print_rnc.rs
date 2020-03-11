use std::collections::HashMap;

use crate::{Context, RestNoteChord};
use entity::{Entity, Join};
use stencil::Stencil;

pub fn sys_print_rnc(
    rnc: &HashMap<Entity, RestNoteChord>,
    contexts: &HashMap<Entity, Context>,
    render: &mut HashMap<Entity, Stencil>,
) {
    for (_id, (rnc, context, render)) in (rnc, contexts, render).join() {
        *render = rnc.print(context);
    }
}
