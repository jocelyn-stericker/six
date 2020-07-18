use kurbo::{Point, Vec2};
use rhythm::{Duration, NoteValue};
use specs::{Component, VecStorage};
use stencil::components::Stencil;

use crate::{components::Context, PitchKind};
use pitch::NoteModifier;

/// A Rest, Note, or Chord.
///
/// There are two kinds of RNCs:
///  - explicit RNCs, created by the user.
///  - managed RNCs: rests created sys_update_chord_timing.
#[derive(Debug)]
pub struct Chord {
    pub duration: Duration,
    pub natural_duration: Duration,
    pub pitch: PitchKind,
}

impl Component for Chord {
    type Storage = VecStorage<Self>;
}

impl Chord {
    pub fn new(duration: Duration, pitch: PitchKind) -> Chord {
        Chord {
            natural_duration: duration,
            duration,
            pitch,
        }
    }

    pub fn duration(&self) -> Duration {
        self.duration
    }

    pub fn is_note(&self) -> bool {
        match self.pitch {
            PitchKind::Rest => false,
            PitchKind::Unpitched | PitchKind::Pitch(_) => true,
        }
    }

    pub fn print(&self, context: &Context, has_beam: bool) -> (Stencil, Option<Point>) {
        let mut stencil = Stencil::default();
        let head_right;
        let mut attachment_for_beam = None;
        let pitch_y;

        match self.pitch {
            PitchKind::Pitch(pitch) => {
                pitch_y = pitch.y(context.clef);
                let is_up = pitch_y > 0.0 || has_beam;
                let (head, mut attachment) = match (self.duration.duration_display_base(), is_up) {
                    (Some(NoteValue::Maxima), _)
                    | (Some(NoteValue::Longa), _)
                    | (Some(NoteValue::DoubleWhole), _) => Stencil::notehead_double_whole(),
                    (Some(NoteValue::Whole), _) => Stencil::notehead_whole(),
                    (Some(NoteValue::Half), true) => Stencil::notehead_half_up(),
                    (Some(NoteValue::Quarter), true)
                    | (Some(NoteValue::Eighth), true)
                    | (Some(NoteValue::Sixteenth), true)
                    | (Some(NoteValue::ThirtySecond), true)
                    | (Some(NoteValue::SixtyFourth), true)
                    | (Some(NoteValue::HundredTwentyEighth), true)
                    | (Some(NoteValue::TwoHundredFiftySixth), true)
                    | (None, true) => Stencil::notehead_black_up(),
                    (Some(NoteValue::Half), false) => Stencil::notehead_half_down(),
                    (Some(NoteValue::Quarter), false)
                    | (Some(NoteValue::Eighth), false)
                    | (Some(NoteValue::Sixteenth), false)
                    | (Some(NoteValue::ThirtySecond), false)
                    | (Some(NoteValue::SixtyFourth), false)
                    | (Some(NoteValue::HundredTwentyEighth), false)
                    | (Some(NoteValue::TwoHundredFiftySixth), false)
                    | (None, false) => Stencil::notehead_black_down(),
                };

                attachment_for_beam = attachment.map(|a| a + Vec2::new(0.0, pitch_y));
                if has_beam {
                    attachment = None;
                }

                // TODO(joshuan): accidentals should be their own entity.
                if context
                    .accidentals
                    .get(&(pitch.name(), pitch.octave()))
                    .cloned()
                    != pitch.modifier()
                {
                    let accidentals = match pitch.modifier() {
                        None => Stencil::natural(),
                        Some(NoteModifier::SemiUp) => Stencil::sharp(),
                        Some(NoteModifier::SemiDown) => Stencil::flat(),
                    };
                    let x1 = accidentals.rect().x1;
                    stencil =
                        stencil.and(accidentals.with_translation(Vec2::new(-x1 - 20.0, pitch_y)));
                }

                head_right = head.rect().x1;

                // TODO(joshuan): Determine direction elsewhere. Be clever with middle stems.
                if let Some(attachment) = attachment {
                    if is_up {
                        stencil = stencil.and(head.with_translation(Vec2::new(0.0, pitch_y)));
                        let flag = match self.duration.duration_display_base() {
                            Some(NoteValue::Eighth) => Some(Stencil::flag_up_8()),
                            Some(NoteValue::Sixteenth) => Some(Stencil::flag_up_16()),
                            Some(NoteValue::ThirtySecond) => Some(Stencil::flag_up_32()),
                            Some(NoteValue::SixtyFourth) => Some(Stencil::flag_up_64()),
                            Some(NoteValue::HundredTwentyEighth) => Some(Stencil::flag_up_128()),
                            Some(NoteValue::TwoHundredFiftySixth) => Some(Stencil::flag_up_256()),
                            _ => None,
                        };

                        let top = (attachment.y + pitch_y - 875.0).min(0.0);
                        let stem = Stencil::stem_line(
                            attachment.x,
                            attachment.y + pitch_y,
                            top + flag.as_ref().map(|a| a.1.y).unwrap_or(0.0),
                        );
                        let stem_width = stem.rect().width();
                        stencil = stencil.and(stem);

                        if let Some((flag, _)) = flag {
                            stencil =
                                stencil.and(flag.with_translation(Vec2::new(
                                    attachment.x - stem_width / 2.0,
                                    top,
                                )));
                        }
                    } else {
                        let flag = match self.duration.duration_display_base() {
                            Some(NoteValue::Eighth) => Some(Stencil::flag_down_8()),
                            Some(NoteValue::Sixteenth) => Some(Stencil::flag_down_16()),
                            Some(NoteValue::ThirtySecond) => Some(Stencil::flag_down_32()),
                            Some(NoteValue::SixtyFourth) => Some(Stencil::flag_down_64()),
                            Some(NoteValue::HundredTwentyEighth) => Some(Stencil::flag_down_128()),
                            Some(NoteValue::TwoHundredFiftySixth) => Some(Stencil::flag_down_256()),
                            _ => None,
                        };

                        stencil = stencil.and(head.with_translation(Vec2::new(0.0, pitch_y)));

                        let bottom = (attachment.y + pitch_y + 875.0).max(0.0);
                        let stem = Stencil::stem_line(
                            0.0,
                            attachment.y + pitch_y,
                            bottom + flag.as_ref().map(|a| a.1.y).unwrap_or(0.0),
                        );
                        let stem_width = stem.rect().width();
                        stencil = stencil.and(stem);

                        if let Some((flag, _)) = flag {
                            stencil = stencil.and(flag.with_translation(Vec2::new(
                                attachment.x - stem_width / 2.0,
                                bottom,
                            )));
                        }
                    }
                } else {
                    stencil = stencil.and(head.with_translation(Vec2::new(0.0, pitch_y)));
                }

                // TODO(joshuan): Leger lines should be their own entities.
                if pitch_y >= 750.0 {
                    let bottom_legers = (pitch_y / 250.0).floor() as usize - 2;
                    for i in 0..bottom_legers {
                        stencil = stencil.and(Stencil::leger_line(
                            0.0,
                            head_right,
                            ((i + 3) as f64) * 250.0,
                        ));
                    }
                }
                if pitch_y <= -750.0 {
                    let top_legers = (pitch_y / -250.0).floor() as usize - 2;
                    for i in 0..top_legers {
                        stencil = stencil.and(Stencil::leger_line(
                            0.0,
                            head_right,
                            ((i + 3) as f64) * -250.0,
                        ));
                    }
                }
            }
            PitchKind::Unpitched => {
                pitch_y = 0.0;
                let (head, attachment) = match self.duration.duration_display_base() {
                    Some(NoteValue::Maxima)
                    | Some(NoteValue::Longa)
                    | Some(NoteValue::DoubleWhole) => Stencil::notehead_x_double_whole(),
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

                attachment_for_beam = attachment;
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

                    let top = attachment.y - 875.0;
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
            }
            PitchKind::Rest => {
                pitch_y = 0.0;
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
            }
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
            stencil = stencil.and(dot_stencil.with_translation(Vec2::new(
                head_right,
                if (pitch_y as i64) % 250 == 0 {
                    pitch_y - 125.0
                } else {
                    pitch_y
                },
            )));
        }

        (stencil, attachment_for_beam)
    }
}

impl Default for Chord {
    fn default() -> Chord {
        Chord {
            duration: Duration::new(NoteValue::Quarter, 0, None),
            natural_duration: Duration::new(NoteValue::Quarter, 0, None),
            pitch: PitchKind::Rest,
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
        let context = Context::default();

        let notes = Stencil::padding(200.0)
            .and_right(
                Chord::new(
                    Duration::new(NoteValue::DoubleWhole, 0, None),
                    PitchKind::Unpitched,
                )
                .print(&context, false)
                .0,
            )
            .and_right(Stencil::padding(200.0))
            .and_right(
                Chord::new(
                    Duration::new(NoteValue::Whole, 0, None),
                    PitchKind::Unpitched,
                )
                .print(&context, false)
                .0,
            )
            .and_right(Stencil::padding(200.0))
            .and_right(
                Chord::new(
                    Duration::new(NoteValue::Half, 0, None),
                    PitchKind::Unpitched,
                )
                .print(&context, false)
                .0,
            )
            .and_right(Stencil::padding(200.0))
            .and_right(
                Chord::new(
                    Duration::new(NoteValue::Quarter, 0, None),
                    PitchKind::Unpitched,
                )
                .print(&context, false)
                .0,
            )
            .and_right(Stencil::padding(200.0))
            .and_right(
                Chord::new(
                    Duration::new(NoteValue::Eighth, 0, None),
                    PitchKind::Unpitched,
                )
                .print(&context, false)
                .0,
            )
            .and_right(Stencil::padding(200.0))
            .and_right(
                Chord::new(
                    Duration::new(NoteValue::Sixteenth, 0, None),
                    PitchKind::Unpitched,
                )
                .print(&context, false)
                .0,
            )
            .and_right(Stencil::padding(200.0))
            .and_right(
                Chord::new(
                    Duration::new(NoteValue::ThirtySecond, 0, None),
                    PitchKind::Unpitched,
                )
                .print(&context, false)
                .0,
            )
            .and_right(Stencil::padding(200.0))
            .and_right(
                Chord::new(
                    Duration::new(NoteValue::SixtyFourth, 0, None),
                    PitchKind::Unpitched,
                )
                .print(&context, false)
                .0,
            )
            .and_right(Stencil::padding(200.0))
            .and_right(
                Chord::new(
                    Duration::new(NoteValue::HundredTwentyEighth, 0, None),
                    PitchKind::Unpitched,
                )
                .print(&context, false)
                .0,
            )
            .and_right(Stencil::padding(200.0))
            .and_right(
                Chord::new(
                    Duration::new(NoteValue::TwoHundredFiftySixth, 0, None),
                    PitchKind::Unpitched,
                )
                .print(&context, false)
                .0,
            )
            .and_right(Stencil::padding(200.0))
            .and_right(
                Chord::new(
                    Duration::new(NoteValue::DoubleWhole, 1, None),
                    PitchKind::Unpitched,
                )
                .print(&context, false)
                .0,
            )
            .and_right(Stencil::padding(200.0))
            .and_right(
                Chord::new(
                    Duration::new(NoteValue::Whole, 1, None),
                    PitchKind::Unpitched,
                )
                .print(&context, false)
                .0,
            )
            .and_right(Stencil::padding(200.0))
            .and_right(
                Chord::new(
                    Duration::new(NoteValue::Half, 1, None),
                    PitchKind::Unpitched,
                )
                .print(&context, false)
                .0,
            )
            .and_right(Stencil::padding(200.0))
            .and_right(
                Chord::new(
                    Duration::new(NoteValue::Quarter, 1, None),
                    PitchKind::Unpitched,
                )
                .print(&context, false)
                .0,
            )
            .and_right(Stencil::padding(200.0))
            .and_right(
                Chord::new(
                    Duration::new(NoteValue::Eighth, 1, None),
                    PitchKind::Unpitched,
                )
                .print(&context, false)
                .0,
            )
            .and_right(Stencil::padding(200.0))
            .and_right(
                Chord::new(
                    Duration::new(NoteValue::Sixteenth, 1, None),
                    PitchKind::Unpitched,
                )
                .print(&context, false)
                .0,
            )
            .and_right(Stencil::padding(200.0))
            .and_right(
                Chord::new(
                    Duration::new(NoteValue::ThirtySecond, 1, None),
                    PitchKind::Unpitched,
                )
                .print(&context, false)
                .0,
            )
            .and_right(Stencil::padding(200.0))
            .and_right(
                Chord::new(
                    Duration::new(NoteValue::SixtyFourth, 1, None),
                    PitchKind::Unpitched,
                )
                .print(&context, false)
                .0,
            )
            .and_right(Stencil::padding(200.0))
            .and_right(
                Chord::new(
                    Duration::new(NoteValue::HundredTwentyEighth, 1, None),
                    PitchKind::Unpitched,
                )
                .print(&context, false)
                .0,
            )
            .and_right(Stencil::padding(200.0))
            .and_right(
                Chord::new(
                    Duration::new(NoteValue::TwoHundredFiftySixth, 1, None),
                    PitchKind::Unpitched,
                )
                .print(&context, false)
                .0,
            )
            .and_right(Stencil::padding(200.0))
            .and_right(
                Chord::new(
                    Duration::new(NoteValue::DoubleWhole, 0, None),
                    PitchKind::Rest,
                )
                .print(&context, false)
                .0,
            )
            .and_right(Stencil::padding(200.0))
            .and_right(
                Chord::new(Duration::new(NoteValue::Whole, 0, None), PitchKind::Rest)
                    .print(&context, false)
                    .0,
            )
            .and_right(Stencil::padding(200.0))
            .and_right(
                Chord::new(Duration::new(NoteValue::Half, 0, None), PitchKind::Rest)
                    .print(&context, false)
                    .0,
            )
            .and_right(Stencil::padding(200.0))
            .and_right(
                Chord::new(Duration::new(NoteValue::Quarter, 0, None), PitchKind::Rest)
                    .print(&context, false)
                    .0,
            )
            .and_right(Stencil::padding(200.0))
            .and_right(
                Chord::new(Duration::new(NoteValue::Eighth, 0, None), PitchKind::Rest)
                    .print(&context, false)
                    .0,
            )
            .and_right(Stencil::padding(200.0))
            .and_right(
                Chord::new(
                    Duration::new(NoteValue::Sixteenth, 0, None),
                    PitchKind::Rest,
                )
                .print(&context, false)
                .0,
            )
            .and_right(Stencil::padding(200.0))
            .and_right(
                Chord::new(
                    Duration::new(NoteValue::ThirtySecond, 0, None),
                    PitchKind::Rest,
                )
                .print(&context, false)
                .0,
            )
            .and_right(Stencil::padding(200.0))
            .and_right(
                Chord::new(
                    Duration::new(NoteValue::SixtyFourth, 0, None),
                    PitchKind::Rest,
                )
                .print(&context, false)
                .0,
            )
            .and_right(Stencil::padding(200.0))
            .and_right(
                Chord::new(
                    Duration::new(NoteValue::HundredTwentyEighth, 0, None),
                    PitchKind::Rest,
                )
                .print(&context, false)
                .0,
            )
            .and_right(Stencil::padding(200.0))
            .and_right(
                Chord::new(
                    Duration::new(NoteValue::TwoHundredFiftySixth, 0, None),
                    PitchKind::Rest,
                )
                .print(&context, false)
                .0,
            )
            .and_right(Stencil::padding(200.0))
            .and_right(
                Chord::new(
                    Duration::new(NoteValue::DoubleWhole, 2, None),
                    PitchKind::Rest,
                )
                .print(&context, false)
                .0,
            )
            .and_right(Stencil::padding(200.0))
            .and_right(
                Chord::new(Duration::new(NoteValue::Whole, 2, None), PitchKind::Rest)
                    .print(&context, false)
                    .0,
            )
            .and_right(Stencil::padding(200.0))
            .and_right(
                Chord::new(Duration::new(NoteValue::Half, 2, None), PitchKind::Rest)
                    .print(&context, false)
                    .0,
            )
            .and_right(Stencil::padding(200.0))
            .and_right(
                Chord::new(Duration::new(NoteValue::Quarter, 2, None), PitchKind::Rest)
                    .print(&context, false)
                    .0,
            )
            .and_right(Stencil::padding(200.0))
            .and_right(
                Chord::new(Duration::new(NoteValue::Eighth, 2, None), PitchKind::Rest)
                    .print(&context, false)
                    .0,
            )
            .and_right(Stencil::padding(200.0))
            .and_right(
                Chord::new(
                    Duration::new(NoteValue::Sixteenth, 2, None),
                    PitchKind::Rest,
                )
                .print(&context, false)
                .0,
            )
            .and_right(Stencil::padding(200.0))
            .and_right(
                Chord::new(
                    Duration::new(NoteValue::ThirtySecond, 2, None),
                    PitchKind::Rest,
                )
                .print(&context, false)
                .0,
            )
            .and_right(Stencil::padding(200.0))
            .and_right(
                Chord::new(
                    Duration::new(NoteValue::SixtyFourth, 2, None),
                    PitchKind::Rest,
                )
                .print(&context, false)
                .0,
            )
            .and_right(Stencil::padding(200.0))
            .and_right(
                Chord::new(
                    Duration::new(NoteValue::HundredTwentyEighth, 2, None),
                    PitchKind::Rest,
                )
                .print(&context, false)
                .0,
            )
            .and_right(Stencil::padding(200.0))
            .and_right(
                Chord::new(
                    Duration::new(NoteValue::TwoHundredFiftySixth, 2, None),
                    PitchKind::Rest,
                )
                .print(&context, false)
                .0,
            );

        let right = notes.rect().x1;

        snapshot(
            "./snapshots/chord.svg",
            &notes
                .and(Stencil::staff_line(right + 200.0))
                .with_translation(Vec2::new(0.0, 2000.0))
                .to_svg_doc_for_testing(),
        );
    }
}
