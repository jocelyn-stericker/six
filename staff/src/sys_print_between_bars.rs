use std::collections::HashMap;

use crate::BetweenBars;
use entity::{Entity, Join};
use rest_note_chord::Context;
use stencil::Stencil;

pub fn sys_print_between_bars(
    between_bars: &HashMap<Entity, BetweenBars>,
    contexts: &HashMap<Entity, Context>,
    render: &mut HashMap<Entity, Stencil>,
) {
    for (between_bar, stencil, context) in (between_bars, render, contexts).join().values_mut() {
        **stencil = between_bar.render(context);
    }
}
