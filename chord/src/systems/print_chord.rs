#![allow(clippy::type_complexity)]

use stencil::components::Stencil;

use crate::components::{BeamForChord, Chord, Context, FlagAttachment};
use specs::{Join, ReadStorage, System, WriteStorage};

#[derive(Debug, Default)]
pub struct PrintChord;

impl<'a> System<'a> for PrintChord {
    type SystemData = (
        ReadStorage<'a, Chord>,
        ReadStorage<'a, Context>,
        ReadStorage<'a, BeamForChord>,
        WriteStorage<'a, FlagAttachment>,
        WriteStorage<'a, Stencil>,
    );

    fn run(
        &mut self,
        (chords, contexts, beam_for_chord, mut attachments, mut stencils): Self::SystemData,
    ) {
        for (chord, context, beam, attachment, stencil) in (
            &chords,
            &contexts,
            beam_for_chord.maybe(),
            &mut attachments,
            &mut stencils,
        )
            .join()
        {
            let result = chord.print(context, beam.is_some());
            *stencil = result.0;
            attachment.0 = result.1;
        }
    }
}
