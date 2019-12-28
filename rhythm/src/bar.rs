use crate::duration::Duration;
use crate::metre::Metre;
use num_rational::Rational;
use num_traits::{sign::Signed, Zero};

#[derive(Clone)]
/// The rhythm and metre of a voice in a single bar.
pub struct Bar {
    metre: Metre,
    // true means a note, chord, or implicit rest. This will be an Entity eventually.
    rhythm: Vec<(Duration, bool)>,
}

impl Bar {
    /// Create a bar with a whole rest.
    pub fn new(metre: Metre) -> Bar {
        Bar {
            metre,
            rhythm: vec![],
        }
    }

    pub fn whole_rest(&self) -> bool {
        self.rhythm.is_empty()
    }

    pub fn metre(&self) -> &Metre {
        &self.metre
    }

    pub fn rhythm(&self) -> &Vec<(Duration, bool)> {
        &self.rhythm
    }

    /// Replace whole rests with segment-length rests.
    fn fill(&mut self) {
        if self.whole_rest() {
            for (from, to) in self
                .metre
                .division_starts()
                .into_iter()
                .zip(self.metre.division_starts().into_iter().skip(1))
            {
                self.rhythm.push((Duration::exact(to - from, None), false));
            }
        }
    }

    /// Starting at the `splice_start`, in whole notes, replace existing notes with `replacement`.
    ///
    /// Durations are real, as opposed to display. If the duration should (or must) be
    ///
    /// If the splice starts after the bar ends, nothing is changed.
    ///
    /// If the splice goes beyond the end of the bar, parts of the splice after the bar are
    /// ignored.
    ///
    /// Existing notes before the splice are shortened to be entirely before the splice.
    /// Existing notes starting after the splice starts and ending before the splice ends are
    /// removed, and non-overlapping parts are replaced with rests.
    /// Existing notes entirely after the splice are kept.
    ///
    /// TODO: Rests will be organized according to the metre.
    /// TODO: Handle tuplets
    pub fn splice(&mut self, splice_start: Rational, replacement: Vec<(Duration, bool)>) {
        let bar_duration = self.metre.duration();

        if splice_start >= bar_duration {
            return;
        }

        self.fill();

        // The time, in whole notes, at which `existing_note` started, before this change.
        let mut t_read = Rational::zero();
        // The time up to which we have written to `new`.
        let mut t_write = Rational::zero();
        // The notes and rests, after the change.
        let mut new = Vec::new();

        for existing_note in &self.rhythm {
            let existing_note_end = t_read + existing_note.0.duration();
            if t_read <= splice_start && splice_start < existing_note_end {
                let new_duration = splice_start - t_read;

                if new_duration.is_positive() {
                    // We are splicing part of the current note. The current note is shortened.
                    new.push((Duration::exact(new_duration, None), existing_note.1));
                    t_write += new_duration;
                } else {
                    // We are splicing the entire current note. The current note is skipped.
                }

                // In either case, we advance the time at which we are reading existing notes.
                t_read = existing_note_end;

                assert_eq!(t_write, splice_start);

                for new_note in &replacement {
                    let new_note_end = t_write + new_note.0.duration();
                    if new_note_end <= bar_duration {
                        new.push(*new_note);
                        t_write = new_note_end;
                    } else {
                        let new_duration = bar_duration - t_write;
                        if new_duration.is_positive() {
                            new.push((Duration::exact(new_duration, None), new_note.1));
                        }
                    }
                }

                let pad_end = t_read - t_write;
                if pad_end.is_positive() {
                    new.push((Duration::exact(pad_end, None), false));
                    t_write += pad_end;
                }

                assert!(t_write >= t_read);
            } else if t_read < t_write {
                let new_duration = existing_note_end - t_write;

                if new_duration.is_positive() {
                    // We are splicing only the start of this note. We are creating a new rest with
                    // the remaining duration.
                    new.push((Duration::exact(new_duration, None), false));
                    t_read = existing_note_end;
                    t_write += new_duration;
                    assert_eq!(t_read, t_write);
                } else {
                    // We are splicing the entire current note. The current note is skipped.
                    t_read = existing_note_end;
                }
            } else {
                assert_eq!(t_read, t_write);
                new.push(*existing_note);
                t_read = existing_note_end;
                t_write = existing_note_end;
            }
        }

        self.rhythm = new;
    }
}

#[cfg(test)]
mod bar_tests {
    use super::*;
    use crate::duration::NoteValue;

    #[test]
    fn four_four_quarters() {
        {
            let mut bar = Bar::new(Metre::new(4, 4));
            assert!(bar.whole_rest());
            bar.splice(
                Rational::zero(),
                vec![(Duration::new(NoteValue::Quarter, 0, None), true)],
            );
            assert_eq!(
                bar.rhythm(),
                &vec![
                    (Duration::new(NoteValue::Quarter, 0, None), true),
                    (Duration::new(NoteValue::Quarter, 0, None), false),
                    (Duration::new(NoteValue::Half, 0, None), false),
                ],
            );
            assert!(!bar.whole_rest());
        }
        {
            let mut bar = Bar::new(Metre::new(4, 4));
            bar.splice(
                Rational::new(1, 4),
                vec![(Duration::new(NoteValue::Quarter, 0, None), true)],
            );
            assert_eq!(
                bar.rhythm(),
                &vec![
                    (Duration::new(NoteValue::Quarter, 0, None), false),
                    (Duration::new(NoteValue::Quarter, 0, None), true),
                    (Duration::new(NoteValue::Half, 0, None), false),
                ],
            );
        }
        {
            let mut bar = Bar::new(Metre::new(4, 4));
            bar.splice(
                Rational::new(1, 2),
                vec![(Duration::new(NoteValue::Quarter, 0, None), true)],
            );
            assert_eq!(
                bar.rhythm(),
                &vec![
                    (Duration::new(NoteValue::Half, 0, None), false),
                    (Duration::new(NoteValue::Quarter, 0, None), true),
                    (Duration::new(NoteValue::Quarter, 0, None), false),
                ],
            );
        }
        {
            let mut bar = Bar::new(Metre::new(4, 4));
            bar.splice(
                Rational::new(3, 4),
                vec![
                    (Duration::new(NoteValue::Quarter, 0, None), true),
                    (Duration::new(NoteValue::Quarter, 0, None), true),
                ],
            );
            assert_eq!(
                bar.rhythm(),
                &vec![
                    (Duration::new(NoteValue::Half, 0, None), false),
                    (Duration::new(NoteValue::Quarter, 0, None), false),
                    (Duration::new(NoteValue::Quarter, 0, None), true),
                ],
            );
        }
        {
            let mut bar = Bar::new(Metre::new(4, 4));
            assert!(bar.whole_rest());
            bar.splice(
                Rational::new(4, 4),
                vec![(Duration::new(NoteValue::Quarter, 0, None), true)],
            );
            assert_eq!(bar.rhythm(), &vec![]);
            assert!(bar.whole_rest());
        }
        {
            let mut bar = Bar::new(Metre::new(4, 4));
            bar.splice(
                Rational::new(1, 4),
                vec![(Duration::new(NoteValue::Half, 0, None), true)],
            );
            assert_eq!(
                bar.rhythm(),
                &vec![
                    (Duration::new(NoteValue::Quarter, 0, None), false),
                    (Duration::new(NoteValue::Half, 0, None), true),
                    (Duration::new(NoteValue::Quarter, 0, None), false),
                ],
            );
        }
    }
}
