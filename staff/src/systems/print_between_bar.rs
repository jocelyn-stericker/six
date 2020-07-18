use crate::components::{BetweenBars, Context};
use specs::{Join, ReadStorage, System, WriteStorage};
use stencil::components::Stencil;

#[derive(Debug, Default)]
pub struct PrintBetweenBar;

impl<'a> System<'a> for PrintBetweenBar {
    type SystemData = (
        ReadStorage<'a, BetweenBars>,
        ReadStorage<'a, Context>,
        WriteStorage<'a, Stencil>,
    );

    fn run(&mut self, (between_bars, contexts, mut stencils): Self::SystemData) {
        for (between_bar, context) in (&between_bars, &contexts).join() {
            *stencils.get_mut(between_bar.stencil_start).unwrap() =
                between_bar.render_start(context);
            *stencils.get_mut(between_bar.stencil_middle).unwrap() =
                between_bar.render_mid(context);
            *stencils.get_mut(between_bar.stencil_end).unwrap() = between_bar.render_end(context);
        }
    }
}
