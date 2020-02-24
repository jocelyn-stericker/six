use std::collections::HashMap;

use crate::BetweenBars;
use entity::{Entity, Join};
use rest_note_chord::Context;
use stencil::Stencil;

pub fn sys_print_between_bars(
    between_bars: &HashMap<Entity, BetweenBars>,
    contexts: &HashMap<Entity, Context>,
    stencils: &mut HashMap<Entity, Stencil>,
) {
    for (between_bar, context) in (between_bars, contexts).join().values_mut() {
        *stencils.get_mut(&between_bar.stencil_start).unwrap() = between_bar.render_start(context);
        *stencils.get_mut(&between_bar.stencil_middle).unwrap() = between_bar.render_mid(context);
        *stencils.get_mut(&between_bar.stencil_end).unwrap() = between_bar.render_end(context);
    }
}
