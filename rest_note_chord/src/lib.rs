#![allow(clippy::implicit_hasher, clippy::blacklisted_name)]

mod sys_implicit_rests;
mod sys_print_rnc;
mod sys_relative_spacing;

use kurbo::Vec2;
use num_rational::Rational;
use rhythm::{Duration, NoteValue};
use stencil::Stencil;

pub use sys_implicit_rests::sys_implicit_rests;
pub use sys_print_rnc::sys_print_rnc;
pub use sys_relative_spacing::sys_relative_spacing;

#[derive(Debug)]
pub struct RestNoteChord {
    duration: Duration,
    is_note: bool,
    start: Rational,
}

impl RestNoteChord {
    pub fn new(duration: Duration, is_note: bool, start: Rational) -> RestNoteChord {
        RestNoteChord {
            duration,
            is_note,
            start,
        }
    }

    pub fn duration(&self) -> Duration {
        self.duration
    }

    pub fn is_note(&self) -> bool {
        self.is_note
    }

    pub fn start(&self) -> Rational {
        self.start
    }

    pub fn print(&self) -> Stencil {
        let mut stencil;
        let head_right;

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

            head_right = head.rect().x1;
            stencil = head;

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

                let top = attachment.y + 875.0;
                let stem = Stencil::stem_line(
                    attachment.x,
                    attachment.y,
                    top + flag.as_ref().map(|a| a.1.y).unwrap_or(0.0),
                );
                let stem_width = stem.rect().width();
                stencil = stencil.and(stem);

                if let Some((flag, _)) = flag {
                    stencil = stencil.and(
                        flag.with_translation(Vec2::new(attachment.x - stem_width / 2.0, top)),
                    );
                }
            }
        } else {
            stencil = match self.duration.duration_display_base() {
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
                None => Stencil::padding(200.0),
            };
            head_right = stencil.rect().x1;
        };

        if let Some(dots) = self.duration.display_dots() {
            let mut dot_stencil = Stencil::default();
            for i in 0..dots {
                if i == 0 {
                    dot_stencil = dot_stencil.and_right(Stencil::padding(112.5));
                } else {
                    dot_stencil = dot_stencil.and_right(Stencil::padding(12.5));
                }
                dot_stencil = dot_stencil.and_right(Stencil::augmentation_dot());
            }
            stencil = stencil.and(dot_stencil.with_translation(Vec2::new(head_right, 125.0)));
        }

        stencil
    }
}

impl Default for RestNoteChord {
    fn default() -> RestNoteChord {
        RestNoteChord {
            duration: Duration::new(NoteValue::Quarter, 0, None),
            is_note: false,
            start: Rational::new(0, 1),
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

        let notes = Stencil::padding(200.0)
            .and_right(
                RestNoteChord::new(
                    Duration::new(NoteValue::DoubleWhole, 0, None),
                    true,
                    Rational::new(0, 1),
                )
                .print(),
            )
            .and_right(Stencil::padding(200.0))
            .and_right(
                RestNoteChord::new(
                    Duration::new(NoteValue::Whole, 0, None),
                    true,
                    Rational::new(0, 1),
                )
                .print(),
            )
            .and_right(Stencil::padding(200.0))
            .and_right(
                RestNoteChord::new(
                    Duration::new(NoteValue::Half, 0, None),
                    true,
                    Rational::new(0, 1),
                )
                .print(),
            )
            .and_right(Stencil::padding(200.0))
            .and_right(
                RestNoteChord::new(
                    Duration::new(NoteValue::Quarter, 0, None),
                    true,
                    Rational::new(0, 1),
                )
                .print(),
            )
            .and_right(Stencil::padding(200.0))
            .and_right(
                RestNoteChord::new(
                    Duration::new(NoteValue::Eighth, 0, None),
                    true,
                    Rational::new(0, 1),
                )
                .print(),
            )
            .and_right(Stencil::padding(200.0))
            .and_right(
                RestNoteChord::new(
                    Duration::new(NoteValue::Sixteenth, 0, None),
                    true,
                    Rational::new(0, 1),
                )
                .print(),
            )
            .and_right(Stencil::padding(200.0))
            .and_right(
                RestNoteChord::new(
                    Duration::new(NoteValue::ThirtySecond, 0, None),
                    true,
                    Rational::new(0, 1),
                )
                .print(),
            )
            .and_right(Stencil::padding(200.0))
            .and_right(
                RestNoteChord::new(
                    Duration::new(NoteValue::SixtyFourth, 0, None),
                    true,
                    Rational::new(0, 1),
                )
                .print(),
            )
            .and_right(Stencil::padding(200.0))
            .and_right(
                RestNoteChord::new(
                    Duration::new(NoteValue::HundredTwentyEighth, 0, None),
                    true,
                    Rational::new(0, 1),
                )
                .print(),
            )
            .and_right(Stencil::padding(200.0))
            .and_right(
                RestNoteChord::new(
                    Duration::new(NoteValue::TwoHundredFiftySixth, 0, None),
                    true,
                    Rational::new(0, 1),
                )
                .print(),
            )
            .and_right(Stencil::padding(200.0))
            .and_right(
                RestNoteChord::new(
                    Duration::new(NoteValue::DoubleWhole, 1, None),
                    true,
                    Rational::new(0, 1),
                )
                .print(),
            )
            .and_right(Stencil::padding(200.0))
            .and_right(
                RestNoteChord::new(
                    Duration::new(NoteValue::Whole, 1, None),
                    true,
                    Rational::new(0, 1),
                )
                .print(),
            )
            .and_right(Stencil::padding(200.0))
            .and_right(
                RestNoteChord::new(
                    Duration::new(NoteValue::Half, 1, None),
                    true,
                    Rational::new(0, 1),
                )
                .print(),
            )
            .and_right(Stencil::padding(200.0))
            .and_right(
                RestNoteChord::new(
                    Duration::new(NoteValue::Quarter, 1, None),
                    true,
                    Rational::new(0, 1),
                )
                .print(),
            )
            .and_right(Stencil::padding(200.0))
            .and_right(
                RestNoteChord::new(
                    Duration::new(NoteValue::Eighth, 1, None),
                    true,
                    Rational::new(0, 1),
                )
                .print(),
            )
            .and_right(Stencil::padding(200.0))
            .and_right(
                RestNoteChord::new(
                    Duration::new(NoteValue::Sixteenth, 1, None),
                    true,
                    Rational::new(0, 1),
                )
                .print(),
            )
            .and_right(Stencil::padding(200.0))
            .and_right(
                RestNoteChord::new(
                    Duration::new(NoteValue::ThirtySecond, 1, None),
                    true,
                    Rational::new(0, 1),
                )
                .print(),
            )
            .and_right(Stencil::padding(200.0))
            .and_right(
                RestNoteChord::new(
                    Duration::new(NoteValue::SixtyFourth, 1, None),
                    true,
                    Rational::new(0, 1),
                )
                .print(),
            )
            .and_right(Stencil::padding(200.0))
            .and_right(
                RestNoteChord::new(
                    Duration::new(NoteValue::HundredTwentyEighth, 1, None),
                    true,
                    Rational::new(0, 1),
                )
                .print(),
            )
            .and_right(Stencil::padding(200.0))
            .and_right(
                RestNoteChord::new(
                    Duration::new(NoteValue::TwoHundredFiftySixth, 1, None),
                    true,
                    Rational::new(0, 1),
                )
                .print(),
            )
            .and_right(Stencil::padding(200.0))
            .and_right(
                RestNoteChord::new(
                    Duration::new(NoteValue::DoubleWhole, 0, None),
                    false,
                    Rational::new(0, 1),
                )
                .print(),
            )
            .and_right(Stencil::padding(200.0))
            .and_right(
                RestNoteChord::new(
                    Duration::new(NoteValue::Whole, 0, None),
                    false,
                    Rational::new(0, 1),
                )
                .print(),
            )
            .and_right(Stencil::padding(200.0))
            .and_right(
                RestNoteChord::new(
                    Duration::new(NoteValue::Half, 0, None),
                    false,
                    Rational::new(0, 1),
                )
                .print(),
            )
            .and_right(Stencil::padding(200.0))
            .and_right(
                RestNoteChord::new(
                    Duration::new(NoteValue::Quarter, 0, None),
                    false,
                    Rational::new(0, 1),
                )
                .print(),
            )
            .and_right(Stencil::padding(200.0))
            .and_right(
                RestNoteChord::new(
                    Duration::new(NoteValue::Eighth, 0, None),
                    false,
                    Rational::new(0, 1),
                )
                .print(),
            )
            .and_right(Stencil::padding(200.0))
            .and_right(
                RestNoteChord::new(
                    Duration::new(NoteValue::Sixteenth, 0, None),
                    false,
                    Rational::new(0, 1),
                )
                .print(),
            )
            .and_right(Stencil::padding(200.0))
            .and_right(
                RestNoteChord::new(
                    Duration::new(NoteValue::ThirtySecond, 0, None),
                    false,
                    Rational::new(0, 1),
                )
                .print(),
            )
            .and_right(Stencil::padding(200.0))
            .and_right(
                RestNoteChord::new(
                    Duration::new(NoteValue::SixtyFourth, 0, None),
                    false,
                    Rational::new(0, 1),
                )
                .print(),
            )
            .and_right(Stencil::padding(200.0))
            .and_right(
                RestNoteChord::new(
                    Duration::new(NoteValue::HundredTwentyEighth, 0, None),
                    false,
                    Rational::new(0, 1),
                )
                .print(),
            )
            .and_right(Stencil::padding(200.0))
            .and_right(
                RestNoteChord::new(
                    Duration::new(NoteValue::TwoHundredFiftySixth, 0, None),
                    false,
                    Rational::new(0, 1),
                )
                .print(),
            )
            .and_right(Stencil::padding(200.0))
            .and_right(
                RestNoteChord::new(
                    Duration::new(NoteValue::DoubleWhole, 2, None),
                    false,
                    Rational::new(0, 1),
                )
                .print(),
            )
            .and_right(Stencil::padding(200.0))
            .and_right(
                RestNoteChord::new(
                    Duration::new(NoteValue::Whole, 2, None),
                    false,
                    Rational::new(0, 1),
                )
                .print(),
            )
            .and_right(Stencil::padding(200.0))
            .and_right(
                RestNoteChord::new(
                    Duration::new(NoteValue::Half, 2, None),
                    false,
                    Rational::new(0, 1),
                )
                .print(),
            )
            .and_right(Stencil::padding(200.0))
            .and_right(
                RestNoteChord::new(
                    Duration::new(NoteValue::Quarter, 2, None),
                    false,
                    Rational::new(0, 1),
                )
                .print(),
            )
            .and_right(Stencil::padding(200.0))
            .and_right(
                RestNoteChord::new(
                    Duration::new(NoteValue::Eighth, 2, None),
                    false,
                    Rational::new(0, 1),
                )
                .print(),
            )
            .and_right(Stencil::padding(200.0))
            .and_right(
                RestNoteChord::new(
                    Duration::new(NoteValue::Sixteenth, 2, None),
                    false,
                    Rational::new(0, 1),
                )
                .print(),
            )
            .and_right(Stencil::padding(200.0))
            .and_right(
                RestNoteChord::new(
                    Duration::new(NoteValue::ThirtySecond, 2, None),
                    false,
                    Rational::new(0, 1),
                )
                .print(),
            )
            .and_right(Stencil::padding(200.0))
            .and_right(
                RestNoteChord::new(
                    Duration::new(NoteValue::SixtyFourth, 2, None),
                    false,
                    Rational::new(0, 1),
                )
                .print(),
            )
            .and_right(Stencil::padding(200.0))
            .and_right(
                RestNoteChord::new(
                    Duration::new(NoteValue::HundredTwentyEighth, 2, None),
                    false,
                    Rational::new(0, 1),
                )
                .print(),
            )
            .and_right(Stencil::padding(200.0))
            .and_right(
                RestNoteChord::new(
                    Duration::new(NoteValue::TwoHundredFiftySixth, 2, None),
                    false,
                    Rational::new(0, 1),
                )
                .print(),
            );

        let right = notes.rect().x1;

        snapshot(
            "./snapshots/rnc.svg",
            &notes
                .and(Stencil::staff_line(right + 200.0))
                .with_translation(Vec2::new(0.0, -2000.0))
                .with_paper_size(3)
                .to_svg_doc_for_testing(),
        );
    }
}
