use std::collections::HashMap;

use crate::BetweenBars;
use entity::{Entity, Join};
use stencil::Stencil;

pub fn sys_print_between_bars(
    between_bars: &HashMap<Entity, BetweenBars>,
    render: &mut HashMap<Entity, Stencil>,
) {
    for (between_bar, stencil) in (between_bars, render).join().values_mut() {
        **stencil = between_bar.render();
    }
}
