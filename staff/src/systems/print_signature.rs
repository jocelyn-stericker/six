use crate::components::{Children, Context, Signature};
use specs::{Entities, Join, ReadStorage, System, WriteStorage};
use stencil::components::Stencil;

#[derive(Debug, Default)]
pub struct PrintSignature;

impl<'a> System<'a> for PrintSignature {
    type SystemData = (
        Entities<'a>,
        ReadStorage<'a, Signature>,
        ReadStorage<'a, Context>,
        WriteStorage<'a, Stencil>,
        WriteStorage<'a, Children>,
    );

    fn run(&mut self, (ents, signatures, contexts, mut stencils, mut children): Self::SystemData) {
        for (ent, signature, context) in (&ents, &signatures, &contexts).join() {
            *stencils.get_mut(signature.stencil_start).unwrap() = signature.render_start(context);
            *stencils.get_mut(signature.stencil_middle).unwrap() = signature.render_mid(context);
            *stencils.get_mut(signature.stencil_end).unwrap() = signature.render_end(context);
            if let Some(a_children) = children.get(ent).cloned() {
                // Copy the children on the Stencil to the rendered outputs.
                // "end" renders at the end of the line.
                // "middle" renders if the barline is neither at the begining or end of a line.
                // They are never both visible.
                children
                    .insert(signature.stencil_end, a_children.clone())
                    .unwrap();
                children
                    .insert(signature.stencil_middle, a_children)
                    .unwrap();
            }
        }
    }
}
