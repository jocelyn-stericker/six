use crate::components::{Context, Signature};
use specs::{Join, ReadStorage, System, WriteStorage};
use stencil::components::Stencil;

#[derive(Debug, Default)]
pub struct PrintSignature;

impl<'a> System<'a> for PrintSignature {
    type SystemData = (
        ReadStorage<'a, Signature>,
        ReadStorage<'a, Context>,
        WriteStorage<'a, Stencil>,
    );

    fn run(&mut self, (signatures, contexts, mut stencils): Self::SystemData) {
        for (signature, context) in (&signatures, &contexts).join() {
            *stencils.get_mut(signature.stencil_start).unwrap() = signature.render_start(context);
            *stencils.get_mut(signature.stencil_middle).unwrap() = signature.render_mid(context);
            *stencils.get_mut(signature.stencil_end).unwrap() = signature.render_end(context);
        }
    }
}
