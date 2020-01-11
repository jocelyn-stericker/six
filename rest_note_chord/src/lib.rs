use kurbo::Vec2;
use rhythm::{Duration, NoteValue};
use stencil::Stencil;

#[derive(Debug)]
pub struct RestNoteChord {
    duration: Duration,
    is_note: bool,
}

impl RestNoteChord {
    pub fn new(duration: Duration, is_note: bool) -> RestNoteChord {
        RestNoteChord { duration, is_note }
    }

    pub fn print(&self) -> Stencil {
        if self.is_note {
            let (head, attachment) = match self.duration.duration_display_base() {
                Some(NoteValue::Maxima) | Some(NoteValue::Longa) | Some(NoteValue::DoubleWhole) => {
                    Stencil::notehead_x_double_whole()
                }
                Some(NoteValue::Whole) => Stencil::notehead_x_whole(),
                Some(NoteValue::Half) => Stencil::notehead_x_half_up(),
                Some(NoteValue::Quarter)
                | Some(NoteValue::Eighth)
                | Some(NoteValue::Sixteenth)
                | Some(NoteValue::ThirtySecond)
                | Some(NoteValue::SixtyFourth)
                | Some(NoteValue::HundredTwentyEighth)
                | Some(NoteValue::TwoHundredFiftySixth)
                | None => Stencil::notehead_x_black_up(),
            };

            let mut stencil = head;

            if let Some(attachment) = attachment {
                let flag = match self.duration.duration_display_base() {
                    Some(NoteValue::Eighth) => Some(Stencil::flag_up_8()),
                    Some(NoteValue::Sixteenth) => Some(Stencil::flag_up_16()),
                    Some(NoteValue::ThirtySecond) => Some(Stencil::flag_up_32()),
                    Some(NoteValue::SixtyFourth) => Some(Stencil::flag_up_64()),
                    Some(NoteValue::HundredTwentyEighth) => Some(Stencil::flag_up_128()),
                    Some(NoteValue::TwoHundredFiftySixth) => Some(Stencil::flag_up_256()),
                    _ => None,
                };

                if let Some((flag, flag_attachment)) = flag {
                    let top = attachment.y + 3.5 / 4.0;
                    let stem =
                        Stencil::stem_line(attachment.x, attachment.y, top + flag_attachment.y);
                    let stem_width = stem.rect().width();
                    stencil = stencil.and(stem);

                    stencil = stencil.and(
                        flag.with_translation(Vec2::new(attachment.x - stem_width / 2.0, top)),
                    );
                }
            }

            stencil
        } else {
            match self.duration.duration_display_base() {
                Some(NoteValue::Maxima) => Stencil::rest_maxima(),
                Some(NoteValue::Longa) => Stencil::rest_longa(),
                Some(NoteValue::DoubleWhole) => Stencil::rest_double_whole(),
                Some(NoteValue::Whole) => Stencil::rest_whole(),
                Some(NoteValue::Half) => Stencil::rest_half(),
                Some(NoteValue::Quarter) => Stencil::rest_quarter(),
                Some(NoteValue::Eighth) => Stencil::rest_8(),
                Some(NoteValue::Sixteenth) => Stencil::rest_16(),
                Some(NoteValue::ThirtySecond) => Stencil::rest_32(),
                Some(NoteValue::SixtyFourth) => Stencil::rest_64(),
                Some(NoteValue::HundredTwentyEighth) => Stencil::rest_128(),
                Some(NoteValue::TwoHundredFiftySixth) => Stencil::rest_256(),
                None => Stencil::padding(0.2),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn print() {
        use kurbo::Vec2;
        use stencil::snapshot;

        let notes = Stencil::padding(0.2)
            .and_right(
                RestNoteChord::new(Duration::new(NoteValue::DoubleWhole, 0, None), true).print(),
            )
            .and_right(Stencil::padding(0.2))
            .and_right(RestNoteChord::new(Duration::new(NoteValue::Whole, 0, None), true).print())
            .and_right(Stencil::padding(0.2))
            .and_right(RestNoteChord::new(Duration::new(NoteValue::Half, 0, None), true).print())
            .and_right(Stencil::padding(0.2))
            .and_right(RestNoteChord::new(Duration::new(NoteValue::Quarter, 0, None), true).print())
            .and_right(Stencil::padding(0.2))
            .and_right(RestNoteChord::new(Duration::new(NoteValue::Eighth, 0, None), true).print())
            .and_right(Stencil::padding(0.2))
            .and_right(
                RestNoteChord::new(Duration::new(NoteValue::Sixteenth, 0, None), true).print(),
            )
            .and_right(Stencil::padding(0.2))
            .and_right(
                RestNoteChord::new(Duration::new(NoteValue::ThirtySecond, 0, None), true).print(),
            )
            .and_right(Stencil::padding(0.2))
            .and_right(
                RestNoteChord::new(Duration::new(NoteValue::SixtyFourth, 0, None), true).print(),
            )
            .and_right(Stencil::padding(0.2))
            .and_right(
                RestNoteChord::new(Duration::new(NoteValue::HundredTwentyEighth, 0, None), true)
                    .print(),
            )
            .and_right(Stencil::padding(0.2))
            .and_right(
                RestNoteChord::new(
                    Duration::new(NoteValue::TwoHundredFiftySixth, 0, None),
                    true,
                )
                .print(),
            )
            .and_right(Stencil::padding(0.2))
            .and_right(
                RestNoteChord::new(Duration::new(NoteValue::DoubleWhole, 0, None), false).print(),
            )
            .and_right(Stencil::padding(0.2))
            .and_right(RestNoteChord::new(Duration::new(NoteValue::Whole, 0, None), false).print())
            .and_right(Stencil::padding(0.2))
            .and_right(RestNoteChord::new(Duration::new(NoteValue::Half, 0, None), false).print())
            .and_right(Stencil::padding(0.2))
            .and_right(
                RestNoteChord::new(Duration::new(NoteValue::Quarter, 0, None), false).print(),
            )
            .and_right(Stencil::padding(0.2))
            .and_right(RestNoteChord::new(Duration::new(NoteValue::Eighth, 0, None), false).print())
            .and_right(Stencil::padding(0.2))
            .and_right(
                RestNoteChord::new(Duration::new(NoteValue::Sixteenth, 0, None), false).print(),
            )
            .and_right(Stencil::padding(0.2))
            .and_right(
                RestNoteChord::new(Duration::new(NoteValue::ThirtySecond, 0, None), false).print(),
            )
            .and_right(Stencil::padding(0.2))
            .and_right(
                RestNoteChord::new(Duration::new(NoteValue::SixtyFourth, 0, None), false).print(),
            )
            .and_right(Stencil::padding(0.2))
            .and_right(
                RestNoteChord::new(
                    Duration::new(NoteValue::HundredTwentyEighth, 0, None),
                    false,
                )
                .print(),
            )
            .and_right(Stencil::padding(0.2))
            .and_right(
                RestNoteChord::new(
                    Duration::new(NoteValue::TwoHundredFiftySixth, 0, None),
                    false,
                )
                .print(),
            );

        let right = notes.rect().x1;

        snapshot(
            "./snapshots/rnc.svg",
            &notes
                .and(Stencil::staff_line(right + 0.2))
                .with_translation(Vec2::new(0.0, -2.0))
                .with_paper_size(3)
                .to_svg_doc_for_testing(),
        );
    }
}
