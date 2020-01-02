use crate::duration::Duration;
use crate::metre::{Metre, Subdivision, Superdivision};
use num_integer::Integer;
use num_rational::Rational;
use num_traits::{sign::Signed, One, Zero};
use std::collections::{BTreeSet, BinaryHeap};

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

    fn try_amend(
        &self,
        division_start: Rational,
        amend_to: Option<&mut (Duration, bool)>,
        t: Rational,
        duration: Rational,
    ) -> bool {
        if let Some(amend_to) = amend_to {
            let prior_note_t = t - amend_to.0.duration();
            if amend_to.1 {
                return false;
            }

            let amended_duration = Duration::exact(amend_to.0.duration() + duration, None);
            let amended_starts_on_division = self
                .metre
                .on_division(prior_note_t)
                .filter(|d| d.superdivision() != Superdivision::Triple)
                .is_some();

            let amended_ends_on_division = self
                .metre
                .on_division(t + duration)
                .filter(|d| d.superdivision() != Superdivision::Triple)
                .is_some()
                || (t + duration == self.metre().duration());

            let amended_does_not_cross_quad = self
                .metre
                .on_division(t)
                .filter(|d| d.superdivision() == Superdivision::Quadruple)
                .is_none();

            let valid_multi_segment = amended_starts_on_division
                && amended_ends_on_division
                && amended_does_not_cross_quad
                && amended_duration.printable();

            if prior_note_t >= division_start || valid_multi_segment {
                amend_to.0 = amended_duration;
                return true;
            }
        }

        return false;
    }

    /// Merge implicit notes in the same division or spanning multiple divisions (except in triple
    /// metre)
    ///
    /// This will merge some rests that should not be merged. optimize() will split those up.
    fn simplify(&mut self) {
        let mut division_start = Rational::zero();
        let mut division_starts = self.metre.division_starts().into_iter().peekable();

        // The time, in whole notes, at which `existing_note` started, before this change.
        let mut t_read = Rational::zero();
        // The notes and rests, after the change.
        let mut new: Vec<(Duration, bool)> = Vec::new();

        for existing_note in &self.rhythm {
            let mut next_division_start = *division_starts.peek().unwrap();
            let existing_note_end = t_read + existing_note.0.duration();

            while t_read >= next_division_start {
                division_start = division_starts.next().unwrap();
                if division_starts.peek().is_none() {
                    panic!("Overfilled bar.");
                }
                next_division_start = *division_starts.peek().unwrap();
            }

            if !existing_note.1
                && t_read < next_division_start
                && next_division_start < existing_note_end
            {
                let new_duration = next_division_start - t_read;
                assert!(new_duration.is_positive());
                if !self.try_amend(division_start, new.last_mut(), t_read, new_duration) {
                    new.push((Duration::exact(new_duration, None), false));
                }
                // The time up to which we have written to `new`.
                let mut t_write = t_read + new_duration;
                t_read = existing_note_end;
                while t_write < t_read {
                    division_start = division_starts.next().unwrap();
                    next_division_start = *division_starts.peek().unwrap();

                    let p = t_read.min(next_division_start);
                    if !self.try_amend(division_start, new.last_mut(), t_write, new_duration) {
                        new.push((Duration::exact(new_duration, None), false));
                    }

                    t_write = p;
                }

                assert_eq!(t_read, t_write);
            } else {
                if existing_note.1
                    || !self.try_amend(
                        division_start,
                        new.last_mut(),
                        t_read,
                        existing_note.0.duration(),
                    )
                {
                    new.push(*existing_note);
                }

                t_read = existing_note_end;
            }
        }

        if self.rhythm.iter().any(|rhythm| rhythm.1) {
            self.rhythm = new;
        } else {
            self.rhythm.clear();
        }
    }

    fn optimize(&mut self) {
        let quant = Rational::new(
            1,
            self.rhythm
                .iter()
                .fold(1isize, |acc, note| acc.lcm(note.0.duration().denom()))
                .lcm(&self.metre.lcm()),
        );

        let tuplet_kinds: BTreeSet<Rational> =
            self.rhythm.iter().map(|rhy| rhy.0.tuplet()).collect();

        let mut q: BinaryHeap<(
            Rational,
            Rational,
            Vec<(Duration, bool)>,
            Vec<(Duration, bool)>,
        )> = BinaryHeap::new();

        q.push((
            Rational::zero(),
            Rational::zero(),
            self.rhythm.clone(),
            vec![],
        ));

        while let Some((score, time, input, mut output)) = q.pop() {
            if let Some((first, remainder)) = input.split_first() {
                if first.1 {
                    output.push(*first);
                    q.push((
                        score,
                        time + first.0.duration(),
                        remainder.iter().cloned().collect(),
                        output,
                    ));
                } else {
                    let quants = (first.0.duration() / quant).to_integer();
                    for i in (1..=quants).rev() {
                        for tuplet_kind in &tuplet_kinds {
                            let mut output = output.clone();
                            let (div_start, div) = self.metre().division(time);
                            let dur = quant * i;
                            let displayed = Duration::exact(dur * tuplet_kind, Some(*tuplet_kind));

                            if let Some(dots) = displayed.display_dots() {
                                // The longest permitted dotted rest in simple time is one value
                                // smaller than the beat.
                                if dots > 0
                                    && div.subdivision() == Subdivision::Simple
                                    && dur > div.subdivision_duration()
                                {
                                    continue;
                                }
                            }

                            let remainder_t = first.0.duration() - dur;
                            let new_input: Vec<(Duration, bool)> = if remainder_t > Rational::zero()
                            {
                                let mut x: Vec<(Duration, bool)> = input.iter().cloned().collect();
                                x[0].0 =
                                    Duration::exact(remainder_t * tuplet_kind, Some(*tuplet_kind));
                                x
                            } else {
                                input.iter().skip(1).cloned().collect()
                            };
                            output.push((displayed, false));
                            let mut score = score;
                            if !displayed.printable() {
                                score -= 10000;
                            }

                            // shorter is better.
                            score -= 1;

                            let second_div = div_start + div.subdivision_duration() * 2;
                            let mut beats_covered = BTreeSet::new();
                            let mut beats_exposed = BTreeSet::new();
                            {
                                let mut t = Rational::zero();
                                for note in &output {
                                    let t_next = t + note.0.duration();

                                    if t >= div_start && t <= div_start + div.duration() {
                                        let q = div.duration()
                                            / Rational::from_integer(div.subdivisions().into());
                                        for i in 1..=div.subdivisions() {
                                            let t_beat: Rational =
                                                q * Rational::from_integer(i.into()) + div_start;
                                            if t < t_beat && t_beat < t_next {
                                                beats_covered.insert(i);
                                            } else if t_beat == t_next && !note.1 && t == div_start
                                            {
                                                beats_exposed.insert(i);
                                            }
                                        }
                                    }
                                    t = t_next;
                                }
                            }

                            if div.subdivision() == Subdivision::Compound
                                && time <= second_div
                                && second_div <= time + dur
                                && beats_exposed.contains(&2)
                                && beats_covered.contains(&1)
                            {
                                // If we can do it for one fewer rest, it's worth it.
                                score -= Rational::new(99, 100);
                            }

                            {
                                let div_parts = (div.duration() / quant).to_integer() as usize;
                                let start_q = ((time - div_start) / quant).to_integer() as usize;
                                let end_q =
                                    ((time + dur - div_start) / quant).to_integer() as usize;
                                let mut powers: Vec<Rational> =
                                    (0..div_parts).map(|_| Rational::zero()).collect();
                                // eprintln!(">> {} {} {:?}", end_q, div_parts, tuplet_kind);
                                if end_q <= div_parts {
                                    for p in 1..=div_parts {
                                        if div_parts % p != 0 {
                                            continue;
                                        }
                                        if Duration::exact(
                                            quant * (p as isize) * tuplet_kind,
                                            Some(*tuplet_kind),
                                        )
                                        .display_dots()
                                            != Some(0)
                                        {
                                            continue;
                                        }
                                        for i in 0..div_parts {
                                            if i % p == 0 {
                                                if div_parts / p == 3 {
                                                    powers[i] += match (i / p) % 3 {
                                                        0 => {
                                                            if div.subdivision()
                                                                == Subdivision::Compound
                                                            {
                                                                Rational::new(22, 20)
                                                            } else {
                                                                Rational::new(21, 20)
                                                            }
                                                        }
                                                        1 => Rational::new(21, 20),
                                                        2 => {
                                                            if beats_exposed.contains(&1)
                                                                && tuplet_kind.is_one()
                                                            {
                                                                Rational::one()
                                                            } else {
                                                                Rational::new(21, 20)
                                                            }
                                                        }
                                                        _ => unreachable!(),
                                                    };
                                                } else {
                                                    powers[i] += 1;
                                                }
                                            }
                                        }
                                    }
                                    for q in (start_q + 1)..end_q {
                                        if powers[q as usize] >= powers[start_q] {
                                            score -= (Rational::one() + powers[q as usize]
                                                - powers[start_q])
                                                * 2;
                                        }
                                    }
                                    // eprintln!("{:?} {} {} {}", powers, start_q, end_q, score);
                                }
                            }

                            q.push((score, time + dur, new_input, output));
                        }
                    }
                }
            } else {
                self.rhythm = output;
                return;
            }
        }

        panic!("No solution");
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
                            t_write = bar_duration;
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

        self.simplify();
        self.optimize();
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

    #[test]
    fn simplify() {
        {
            let mut bar = Bar::new(Metre::new(4, 4));
            bar.splice(
                Rational::new(1, 4),
                vec![(Duration::new(NoteValue::Half, 0, None), true)],
            );
            bar.splice(
                Rational::new(1, 4),
                vec![(Duration::new(NoteValue::Half, 0, None), false)],
            );
            assert_eq!(bar.rhythm(), &vec![],);
        }

        {
            let mut bar = Bar::new(Metre::new(4, 4));
            bar.splice(
                Rational::new(1, 4),
                vec![(Duration::new(NoteValue::Half, 0, None), true)],
            );
            bar.splice(
                Rational::new(3, 8),
                vec![(Duration::new(NoteValue::Eighth, 0, None), false)],
            );
            assert_eq!(
                bar.rhythm(),
                &vec![
                    (Duration::new(NoteValue::Quarter, 0, None), false),
                    (Duration::new(NoteValue::Eighth, 0, None), true),
                    (Duration::new(NoteValue::Eighth, 0, None), false),
                    (Duration::new(NoteValue::Half, 0, None), false),
                ],
            );
        }

        {
            let mut bar = Bar::new(Metre::new(4, 4));
            bar.splice(
                Rational::new(1, 4),
                vec![(Duration::new(NoteValue::Half, 0, None), true)],
            );
            bar.splice(
                Rational::new(5, 8),
                vec![(Duration::new(NoteValue::Eighth, 0, None), false)],
            );
            // This is not a good rhythm, but we do not adjust explicit rhythms.
            assert_eq!(
                bar.rhythm(),
                &vec![
                    (Duration::new(NoteValue::Quarter, 0, None), false),
                    (Duration::new(NoteValue::Quarter, 1, None), true),
                    (Duration::new(NoteValue::Eighth, 0, None), false),
                    (Duration::new(NoteValue::Quarter, 0, None), false),
                ],
            );
        }

        {
            let mut bar = Bar::new(Metre::new(4, 4));
            bar.splice(
                Rational::new(0, 1),
                vec![(Duration::new(NoteValue::Half, 0, None), true)],
            );
            bar.splice(
                Rational::new(1, 8),
                vec![(Duration::new(NoteValue::Eighth, 0, None), true)],
            );
            bar.splice(
                Rational::new(2, 8),
                vec![(Duration::new(NoteValue::Eighth, 0, None), true)],
            );
            bar.splice(
                Rational::new(3, 8),
                vec![(Duration::new(NoteValue::Eighth, 0, None), true)],
            );
            bar.splice(
                Rational::new(1, 8),
                vec![(Duration::new(NoteValue::Eighth, 0, None), false)],
            );
            bar.splice(
                Rational::new(2, 8),
                vec![(Duration::new(NoteValue::Eighth, 0, None), false)],
            );
            bar.splice(
                Rational::new(3, 8),
                vec![(Duration::new(NoteValue::Eighth, 0, None), false)],
            );
            assert_eq!(
                bar.rhythm(),
                &vec![
                    (Duration::new(NoteValue::Eighth, 0, None), true),
                    (Duration::new(NoteValue::Eighth, 0, None), false),
                    (Duration::new(NoteValue::Quarter, 0, None), false),
                    (Duration::new(NoteValue::Half, 0, None), false),
                ],
            );
        }
    }

    #[test]
    fn simple_time_exposes_middle() {
        // From Beyond Bars, by Elaine Gould (2011), p. 161

        for bar in &mut [Bar::new(Metre::new(4, 8)), Bar::new(Metre::new(2, 4))] {
            bar.splice(
                Rational::new(3, 8),
                vec![(Duration::new(NoteValue::Eighth, 0, None), true)],
            );
            assert_eq!(
                bar.rhythm(),
                &vec![
                    (Duration::new(NoteValue::Quarter, 0, None), false),
                    (Duration::new(NoteValue::Eighth, 0, None), false),
                    (Duration::new(NoteValue::Eighth, 0, None), true),
                ],
            );
        }

        for bar in &mut [Bar::new(Metre::new(4, 8)), Bar::new(Metre::new(2, 4))] {
            bar.splice(
                Rational::zero(),
                vec![(Duration::new(NoteValue::Eighth, 0, None), true)],
            );
            bar.splice(
                Rational::new(3, 8),
                vec![(Duration::new(NoteValue::Eighth, 0, None), true)],
            );
            assert_eq!(
                bar.rhythm(),
                &vec![
                    (Duration::new(NoteValue::Eighth, 0, None), true),
                    (Duration::new(NoteValue::Eighth, 0, None), false),
                    (Duration::new(NoteValue::Eighth, 0, None), false),
                    (Duration::new(NoteValue::Eighth, 0, None), true),
                ],
            );
        }
        for bar in &mut [Bar::new(Metre::new(4, 8)), Bar::new(Metre::new(2, 4))] {
            bar.splice(
                Rational::zero(),
                vec![(Duration::new(NoteValue::Eighth, 0, None), true)],
            );
            assert_eq!(
                bar.rhythm(),
                &vec![
                    (Duration::new(NoteValue::Eighth, 0, None), true),
                    (Duration::new(NoteValue::Eighth, 0, None), false),
                    (Duration::new(NoteValue::Quarter, 0, None), false),
                ],
            );
        }

        for bar in &mut [Bar::new(Metre::new(4, 4)), Bar::new(Metre::new(2, 2))] {
            bar.splice(
                Rational::new(3, 4),
                vec![(Duration::new(NoteValue::Quarter, 0, None), true)],
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

        for bar in &mut [Bar::new(Metre::new(4, 4)), Bar::new(Metre::new(2, 2))] {
            bar.splice(
                Rational::zero(),
                vec![(Duration::new(NoteValue::Quarter, 0, None), true)],
            );
            bar.splice(
                Rational::new(3, 4),
                vec![(Duration::new(NoteValue::Quarter, 0, None), true)],
            );
            assert_eq!(
                bar.rhythm(),
                &vec![
                    (Duration::new(NoteValue::Quarter, 0, None), true),
                    (Duration::new(NoteValue::Quarter, 0, None), false),
                    (Duration::new(NoteValue::Quarter, 0, None), false),
                    (Duration::new(NoteValue::Quarter, 0, None), true),
                ],
            );
        }
        for bar in &mut [Bar::new(Metre::new(4, 4)), Bar::new(Metre::new(2, 2))] {
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
        }
    }

    #[test]
    fn compound_time_exposes_middle() {
        // From Beyond Bars, by Elaine Gould (2011), p. 161

        {
            let mut bar = Bar::new(Metre::new(6, 16));
            bar.splice(
                Rational::new(5, 16),
                vec![(Duration::new(NoteValue::Sixteenth, 0, None), true)],
            );
            assert_eq!(
                bar.rhythm(),
                &vec![
                    (Duration::new(NoteValue::Eighth, 1, None), false),
                    (Duration::new(NoteValue::Eighth, 0, None), false),
                    (Duration::new(NoteValue::Sixteenth, 0, None), true),
                ],
            );
        }
        {
            let mut bar = Bar::new(Metre::new(6, 16));
            bar.splice(
                Rational::zero(),
                vec![(Duration::new(NoteValue::Eighth, 0, None), true)],
            );
            bar.splice(
                Rational::new(4, 16),
                vec![(Duration::new(NoteValue::Eighth, 0, None), true)],
            );
            assert_eq!(
                bar.rhythm(),
                &vec![
                    (Duration::new(NoteValue::Eighth, 0, None), true),
                    (Duration::new(NoteValue::Sixteenth, 0, None), false),
                    (Duration::new(NoteValue::Sixteenth, 0, None), false),
                    (Duration::new(NoteValue::Eighth, 0, None), true),
                ],
            );
        }
        {
            let mut bar = Bar::new(Metre::new(6, 16));
            bar.splice(
                Rational::zero(),
                vec![(Duration::new(NoteValue::Sixteenth, 0, None), true)],
            );
            assert_eq!(
                bar.rhythm(),
                &vec![
                    (Duration::new(NoteValue::Sixteenth, 0, None), true),
                    (Duration::new(NoteValue::Sixteenth, 0, None), false),
                    (Duration::new(NoteValue::Sixteenth, 0, None), false),
                    (Duration::new(NoteValue::Eighth, 1, None), false),
                ],
            );
        }

        {
            let mut bar = Bar::new(Metre::new(6, 8));
            bar.splice(
                Rational::new(5, 8),
                vec![(Duration::new(NoteValue::Eighth, 0, None), true)],
            );
            assert_eq!(
                bar.rhythm(),
                &vec![
                    (Duration::new(NoteValue::Quarter, 1, None), false),
                    (Duration::new(NoteValue::Quarter, 0, None), false),
                    (Duration::new(NoteValue::Eighth, 0, None), true),
                ],
            );
        }
        {
            let mut bar = Bar::new(Metre::new(6, 8));
            bar.splice(
                Rational::zero(),
                vec![(Duration::new(NoteValue::Quarter, 0, None), true)],
            );
            bar.splice(
                Rational::new(4, 8),
                vec![(Duration::new(NoteValue::Quarter, 0, None), true)],
            );
            assert_eq!(
                bar.rhythm(),
                &vec![
                    (Duration::new(NoteValue::Quarter, 0, None), true),
                    (Duration::new(NoteValue::Eighth, 0, None), false),
                    (Duration::new(NoteValue::Eighth, 0, None), false),
                    (Duration::new(NoteValue::Quarter, 0, None), true),
                ],
            );
        }
        {
            let mut bar = Bar::new(Metre::new(6, 8));
            bar.splice(
                Rational::zero(),
                vec![(Duration::new(NoteValue::Eighth, 0, None), true)],
            );
            assert_eq!(
                bar.rhythm(),
                &vec![
                    (Duration::new(NoteValue::Eighth, 0, None), true),
                    (Duration::new(NoteValue::Eighth, 0, None), false),
                    (Duration::new(NoteValue::Eighth, 0, None), false),
                    (Duration::new(NoteValue::Quarter, 1, None), false),
                ],
            );
        }

        {
            let mut bar = Bar::new(Metre::new(6, 4));
            bar.splice(
                Rational::new(5, 4),
                vec![(Duration::new(NoteValue::Quarter, 0, None), true)],
            );
            assert_eq!(
                bar.rhythm(),
                &vec![
                    (Duration::new(NoteValue::Half, 1, None), false),
                    (Duration::new(NoteValue::Half, 0, None), false),
                    (Duration::new(NoteValue::Quarter, 0, None), true),
                ],
            );
        }
        {
            let mut bar = Bar::new(Metre::new(6, 4));
            bar.splice(
                Rational::zero(),
                vec![(Duration::new(NoteValue::Half, 0, None), true)],
            );
            bar.splice(
                Rational::new(4, 4),
                vec![(Duration::new(NoteValue::Half, 0, None), true)],
            );
            assert_eq!(
                bar.rhythm(),
                &vec![
                    (Duration::new(NoteValue::Half, 0, None), true),
                    (Duration::new(NoteValue::Quarter, 0, None), false),
                    (Duration::new(NoteValue::Quarter, 0, None), false),
                    (Duration::new(NoteValue::Half, 0, None), true),
                ],
            );
        }
        {
            let mut bar = Bar::new(Metre::new(6, 4));
            bar.splice(
                Rational::zero(),
                vec![(Duration::new(NoteValue::Quarter, 0, None), true)],
            );
            assert_eq!(
                bar.rhythm(),
                &vec![
                    (Duration::new(NoteValue::Quarter, 0, None), true),
                    (Duration::new(NoteValue::Quarter, 0, None), false),
                    (Duration::new(NoteValue::Quarter, 0, None), false),
                    (Duration::new(NoteValue::Half, 1, None), false),
                ],
            );
        }

        {
            let mut bar = Bar::new(Metre::new(12, 8));
            bar.splice(
                Rational::new(11, 8),
                vec![(Duration::new(NoteValue::Eighth, 0, None), true)],
            );
            assert_eq!(
                bar.rhythm(),
                &vec![
                    (Duration::new(NoteValue::Half, 1, None), false),
                    (Duration::new(NoteValue::Quarter, 1, None), false),
                    (Duration::new(NoteValue::Quarter, 0, None), false),
                    (Duration::new(NoteValue::Eighth, 0, None), true),
                ],
            );
        }
    }

    #[test]
    fn three_shows_all_beats() {
        // From Beyond Bars, by Elaine Gould (2011), p. 161

        {
            let mut bar = Bar::new(Metre::new(3, 4));
            bar.splice(
                Rational::new(2, 4),
                vec![(Duration::new(NoteValue::Quarter, 0, None), true)],
            );
            assert_eq!(
                bar.rhythm(),
                &vec![
                    (Duration::new(NoteValue::Quarter, 0, None), false),
                    (Duration::new(NoteValue::Quarter, 0, None), false),
                    (Duration::new(NoteValue::Quarter, 0, None), true),
                ],
            );
        }

        {
            let mut bar = Bar::new(Metre::new(3, 8));
            bar.splice(
                Rational::new(2, 8),
                vec![(Duration::new(NoteValue::Eighth, 0, None), true)],
            );
            assert_eq!(
                bar.rhythm(),
                &vec![
                    (Duration::new(NoteValue::Eighth, 0, None), false),
                    (Duration::new(NoteValue::Eighth, 0, None), false),
                    (Duration::new(NoteValue::Eighth, 0, None), true),
                ],
            );
        }

        {
            let mut bar = Bar::new(Metre::new(9, 8));
            bar.splice(
                Rational::new(6, 8),
                vec![(Duration::new(NoteValue::Quarter, 1, None), true)],
            );
            assert_eq!(
                bar.rhythm(),
                &vec![
                    (Duration::new(NoteValue::Quarter, 1, None), false),
                    (Duration::new(NoteValue::Quarter, 1, None), false),
                    (Duration::new(NoteValue::Quarter, 1, None), true),
                ],
            );
        }
    }

    #[test]
    fn dotted_rests() {
        // From Beyond Bars, by Elaine Gould (2011), p. 161

        {
            let mut bar = Bar::new(Metre::new(2, 4));
            bar.splice(
                Rational::new(3, 16),
                vec![(Duration::new(NoteValue::Sixteenth, 0, None), true)],
            );
            bar.splice(
                Rational::new(7, 16),
                vec![(Duration::new(NoteValue::Sixteenth, 0, None), true)],
            );
            assert_eq!(
                bar.rhythm(),
                &vec![
                    (Duration::new(NoteValue::Eighth, 1, None), false),
                    (Duration::new(NoteValue::Sixteenth, 0, None), true),
                    (Duration::new(NoteValue::Eighth, 1, None), false),
                    (Duration::new(NoteValue::Sixteenth, 0, None), true),
                ],
            );
        }

        {
            let mut bar = Bar::new(Metre::new(9, 8));
            bar.splice(
                Rational::new(8, 8),
                vec![(Duration::new(NoteValue::Eighth, 0, None), true)],
            );
            assert_eq!(
                bar.rhythm(),
                &vec![
                    (Duration::new(NoteValue::Quarter, 1, None), false),
                    (Duration::new(NoteValue::Quarter, 1, None), false),
                    (Duration::new(NoteValue::Quarter, 0, None), false),
                    (Duration::new(NoteValue::Eighth, 0, None), true),
                ],
            );
        }
    }

    #[test]
    fn no_dotted_rests_at_end_of_simple_time_beat() {
        // From Beyond Bars, by Elaine Gould (2011), p. 162

        {
            let mut bar = Bar::new(Metre::new(2, 4));
            bar.splice(
                Rational::new(3, 16),
                vec![(Duration::new(NoteValue::Sixteenth, 0, None), true)],
            );
            bar.splice(
                Rational::new(4, 16),
                vec![(Duration::new(NoteValue::Sixteenth, 0, None), true)],
            );
            assert_eq!(
                bar.rhythm(),
                &vec![
                    (Duration::new(NoteValue::Eighth, 1, None), false),
                    (Duration::new(NoteValue::Sixteenth, 0, None), true),
                    (Duration::new(NoteValue::Sixteenth, 0, None), true),
                    (Duration::new(NoteValue::Sixteenth, 0, None), false),
                    (Duration::new(NoteValue::Eighth, 0, None), false),
                ],
            );
        }
        {
            let mut bar = Bar::new(Metre::new(2, 2));
            bar.splice(
                Rational::new(3, 8),
                vec![(Duration::new(NoteValue::Eighth, 0, None), true)],
            );
            bar.splice(
                Rational::new(4, 8),
                vec![(Duration::new(NoteValue::Eighth, 0, None), true)],
            );
            assert_eq!(
                bar.rhythm(),
                &vec![
                    (Duration::new(NoteValue::Quarter, 1, None), false),
                    (Duration::new(NoteValue::Eighth, 0, None), true),
                    (Duration::new(NoteValue::Eighth, 0, None), true),
                    (Duration::new(NoteValue::Eighth, 0, None), false),
                    (Duration::new(NoteValue::Quarter, 0, None), false),
                ],
            );
        }
    }

    #[test]
    fn maximum_dotted_rest() {
        // From Beyond Bars, by Elaine Gould (2011), p. 162

        {
            let mut bar = Bar::new(Metre::new(4, 4));
            bar.splice(
                Rational::new(7, 8),
                vec![(Duration::new(NoteValue::Eighth, 0, None), true)],
            );
            assert_eq!(
                bar.rhythm(),
                &vec![
                    (Duration::new(NoteValue::Half, 0, None), false),
                    (Duration::new(NoteValue::Quarter, 0, None), false),
                    (Duration::new(NoteValue::Eighth, 0, None), false),
                    (Duration::new(NoteValue::Eighth, 0, None), true),
                ],
            );
        }

        {
            let mut bar = Bar::new(Metre::new(4, 4));
            bar.splice(
                Rational::new(3, 16),
                vec![(Duration::new(NoteValue::Sixteenth, 0, None), true)],
            );
            assert_eq!(
                bar.rhythm(),
                &vec![
                    (Duration::new(NoteValue::Eighth, 1, None), false),
                    (Duration::new(NoteValue::Sixteenth, 0, None), true),
                    (Duration::new(NoteValue::Quarter, 0, None), false),
                    (Duration::new(NoteValue::Half, 0, None), false),
                ],
            );
        }

        {
            let mut bar = Bar::new(Metre::new(2, 2));
            bar.splice(
                Rational::new(3, 8),
                vec![(Duration::new(NoteValue::Eighth, 0, None), true)],
            );
            bar.splice(
                Rational::new(7, 8),
                vec![(Duration::new(NoteValue::Eighth, 0, None), true)],
            );
            assert_eq!(
                bar.rhythm(),
                &vec![
                    (Duration::new(NoteValue::Quarter, 1, None), false),
                    (Duration::new(NoteValue::Eighth, 0, None), true),
                    (Duration::new(NoteValue::Quarter, 1, None), false),
                    (Duration::new(NoteValue::Eighth, 0, None), true),
                ],
            );
        }

        {
            let mut bar = Bar::new(Metre::new(2, 2));
            bar.splice(
                Rational::zero(),
                vec![(Duration::new(NoteValue::Eighth, 0, None), true)],
            );
            assert_eq!(
                bar.rhythm(),
                &vec![
                    (Duration::new(NoteValue::Eighth, 0, None), true),
                    (Duration::new(NoteValue::Eighth, 0, None), false),
                    (Duration::new(NoteValue::Quarter, 0, None), false),
                    (Duration::new(NoteValue::Half, 0, None), false),
                ],
            );
        }
    }

    #[test]
    fn double_dotted_rest() {
        // From Beyond Bars, by Elaine Gould (2011), p. 162

        {
            let mut bar = Bar::new(Metre::new(2, 4));
            bar.splice(
                Rational::new(7, 32),
                vec![(Duration::new(NoteValue::ThirtySecond, 0, None), true)],
            );
            assert_eq!(
                bar.rhythm(),
                &vec![
                    (Duration::new(NoteValue::Eighth, 2, None), false),
                    (Duration::new(NoteValue::ThirtySecond, 0, None), true),
                    (Duration::new(NoteValue::Quarter, 0, None), false),
                ],
            );
        }

        {
            let mut bar = Bar::new(Metre::new(2, 4));
            bar.splice(
                Rational::new(7, 32),
                vec![(Duration::new(NoteValue::ThirtySecond, 0, None), true)],
            );
            bar.splice(
                Rational::new(8, 32),
                vec![(Duration::new(NoteValue::ThirtySecond, 0, None), true)],
            );
            assert_eq!(
                bar.rhythm(),
                &vec![
                    (Duration::new(NoteValue::Eighth, 2, None), false),
                    (Duration::new(NoteValue::ThirtySecond, 0, None), true),
                    (Duration::new(NoteValue::ThirtySecond, 0, None), true),
                    (Duration::new(NoteValue::ThirtySecond, 0, None), false),
                    (Duration::new(NoteValue::Sixteenth, 0, None), false),
                    (Duration::new(NoteValue::Eighth, 0, None), false),
                ],
            );
        }
    }

    #[test]
    fn expose_middle_of_beat() {
        // From Beyond Bars, by Elaine Gould (2011), p. 163

        {
            let mut bar = Bar::new(Metre::new(2, 4));
            bar.splice(
                Rational::zero(),
                vec![(Duration::new(NoteValue::ThirtySecond, 0, None), true)],
            );
            bar.splice(
                Rational::new(7, 32),
                vec![(Duration::new(NoteValue::ThirtySecond, 0, None), true)],
            );
            assert_eq!(
                bar.rhythm(),
                &vec![
                    (Duration::new(NoteValue::ThirtySecond, 0, None), true),
                    (Duration::new(NoteValue::ThirtySecond, 0, None), false),
                    (Duration::new(NoteValue::Sixteenth, 0, None), false),
                    (Duration::new(NoteValue::Sixteenth, 1, None), false),
                    (Duration::new(NoteValue::ThirtySecond, 0, None), true),
                    (Duration::new(NoteValue::Quarter, 0, None), false),
                ],
            );
        }

        // TODO: "may be combined when rhythms are straightforward".
    }

    #[test]
    fn compound_combine_segments() {
        // From Beyond Bars, by Elaine Gould (2011), p. 163

        {
            let mut bar = Bar::new(Metre::new(12, 8));
            bar.splice(
                Rational::new(9, 8),
                vec![(Duration::new(NoteValue::Quarter, 1, None), true)],
            );
            assert_eq!(
                bar.rhythm(),
                &vec![
                    (Duration::new(NoteValue::Half, 1, None), false),
                    (Duration::new(NoteValue::Quarter, 1, None), false),
                    (Duration::new(NoteValue::Quarter, 1, None), true),
                ],
            );
        }

        {
            let mut bar = Bar::new(Metre::new(12, 8));
            bar.splice(
                Rational::zero(),
                vec![(Duration::new(NoteValue::Quarter, 1, None), true)],
            );
            assert_eq!(
                bar.rhythm(),
                &vec![
                    (Duration::new(NoteValue::Quarter, 1, None), true),
                    (Duration::new(NoteValue::Quarter, 1, None), false),
                    (Duration::new(NoteValue::Half, 1, None), false),
                ],
            );
        }
    }

    #[test]
    fn compound_combine_start() {
        // From Beyond Bars, by Elaine Gould (2011), p. 163

        {
            let mut bar = Bar::new(Metre::new(6, 8));
            bar.splice(
                Rational::new(1, 4),
                vec![(Duration::new(NoteValue::Eighth, 0, None), true)],
            );
            bar.splice(
                Rational::new(5, 8),
                vec![(Duration::new(NoteValue::Eighth, 0, None), true)],
            );
            assert_eq!(
                bar.rhythm(),
                &vec![
                    (Duration::new(NoteValue::Quarter, 0, None), false),
                    (Duration::new(NoteValue::Eighth, 0, None), true),
                    (Duration::new(NoteValue::Quarter, 0, None), false),
                    (Duration::new(NoteValue::Eighth, 0, None), true),
                ],
            );
        }

        {
            let mut bar = Bar::new(Metre::new(6, 8));
            bar.splice(
                Rational::new(5, 16),
                vec![(Duration::new(NoteValue::Sixteenth, 0, None), true)],
            );
            assert_eq!(
                bar.rhythm(),
                &vec![
                    (Duration::new(NoteValue::Eighth, 0, None), false),
                    (Duration::new(NoteValue::Eighth, 1, None), false),
                    (Duration::new(NoteValue::Sixteenth, 0, None), true),
                    (Duration::new(NoteValue::Quarter, 1, None), false),
                ],
            );
        }
    }

    #[test]
    fn compound_spell_out_later_beats() {
        // From Beyond Bars, by Elaine Gould (2011), p. 163
        {
            let mut bar = Bar::new(Metre::new(6, 8));
            bar.splice(
                Rational::zero(),
                vec![(Duration::new(NoteValue::Eighth, 0, None), true)],
            );
            bar.splice(
                Rational::new(3, 8),
                vec![(Duration::new(NoteValue::Sixteenth, 0, None), true)],
            );
            assert_eq!(
                bar.rhythm(),
                &vec![
                    (Duration::new(NoteValue::Eighth, 0, None), true),
                    (Duration::new(NoteValue::Eighth, 0, None), false),
                    (Duration::new(NoteValue::Eighth, 0, None), false),
                    (Duration::new(NoteValue::Sixteenth, 0, None), true),
                    (Duration::new(NoteValue::Sixteenth, 0, None), false),
                    (Duration::new(NoteValue::Eighth, 0, None), false),
                    (Duration::new(NoteValue::Eighth, 0, None), false),
                ],
            );
        }
    }

    #[test]
    fn overfill() {
        let mut bar = Bar::new(Metre::new(4, 4));
        bar.splice(
            Rational::new(3, 4),
            vec![(Duration::new(NoteValue::Half, 0, None), true)],
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

    #[test]
    fn compound_combine_initial_rests_unless_confusing() {
        // p163
        {
            let mut bar = Bar::new(Metre::new(6, 8));
            bar.splice(
                Rational::new(2, 8),
                vec![(Duration::new(NoteValue::Eighth, 0, None), true)],
            );
            bar.splice(
                Rational::new(11, 16),
                vec![(Duration::new(NoteValue::Sixteenth, 0, None), true)],
            );
            assert_eq!(
                bar.rhythm(),
                &vec![
                    (Duration::new(NoteValue::Quarter, 0, None), false),
                    (Duration::new(NoteValue::Eighth, 0, None), true),
                    (Duration::new(NoteValue::Eighth, 0, None), false),
                    (Duration::new(NoteValue::Eighth, 1, None), false),
                    (Duration::new(NoteValue::Sixteenth, 0, None), true),
                ],
            );
        }

        // p164
        {
            let mut bar = Bar::new(Metre::new(9, 8));
            bar.splice(
                Rational::zero(),
                vec![(Duration::new(NoteValue::Eighth, 0, None), true)],
            );
            bar.splice(
                Rational::new(5, 8),
                vec![(Duration::new(NoteValue::Eighth, 0, None), true)],
            );
            bar.splice(
                Rational::new(8, 8),
                vec![(Duration::new(NoteValue::Eighth, 0, None), true)],
            );
            assert_eq!(
                bar.rhythm(),
                &vec![
                    (Duration::new(NoteValue::Eighth, 0, None), true),
                    (Duration::new(NoteValue::Eighth, 0, None), false),
                    (Duration::new(NoteValue::Eighth, 0, None), false),
                    (Duration::new(NoteValue::Quarter, 0, None), false),
                    (Duration::new(NoteValue::Eighth, 0, None), true),
                    (Duration::new(NoteValue::Quarter, 0, None), false),
                    (Duration::new(NoteValue::Eighth, 0, None), true),
                ],
            );
        }

        {
            let mut bar = Bar::new(Metre::new(6, 8));
            bar.splice(
                Rational::zero(),
                vec![(Duration::new(NoteValue::Sixteenth, 0, None), true)],
            );
            bar.splice(
                Rational::new(5, 16),
                vec![(Duration::new(NoteValue::Sixteenth, 0, None), true)],
            );
            assert_eq!(
                bar.rhythm(),
                &vec![
                    (Duration::new(NoteValue::Sixteenth, 0, None), true),
                    (Duration::new(NoteValue::Sixteenth, 0, None), false),
                    // TODO: join?
                    (Duration::new(NoteValue::Eighth, 0, None), false),
                    (Duration::new(NoteValue::Sixteenth, 0, None), false),
                    (Duration::new(NoteValue::Sixteenth, 0, None), true),
                    (Duration::new(NoteValue::Quarter, 1, None), false),
                ],
            );
        }
    }

    #[test]
    fn triplets() {
        // Triplet rests are described on p211, but these are different.
        {
            let mut bar = Bar::new(Metre::new(4, 4));
            bar.splice(
                Rational::zero(),
                vec![(
                    Duration::new(NoteValue::Quarter, 0, Some(Rational::new(3, 2))),
                    true,
                )],
            );
            assert_eq!(
                bar.rhythm(),
                &vec![
                    (
                        Duration::new(NoteValue::Quarter, 0, Some(Rational::new(3, 2))),
                        true
                    ),
                    (
                        Duration::new(NoteValue::Quarter, 0, Some(Rational::new(3, 2))),
                        false
                    ),
                    (
                        Duration::new(NoteValue::Quarter, 0, Some(Rational::new(3, 2))),
                        false
                    ),
                    (Duration::new(NoteValue::Half, 0, None), false),
                ],
            );
        }
        {
            let mut bar = Bar::new(Metre::new(4, 4));
            bar.splice(
                Rational::new(2, 6),
                vec![(
                    Duration::new(NoteValue::Quarter, 0, Some(Rational::new(3, 2))),
                    true,
                )],
            );
            assert_eq!(
                bar.rhythm(),
                &vec![
                    (
                        Duration::new(NoteValue::Quarter, 0, Some(Rational::new(3, 2))),
                        false
                    ),
                    (
                        Duration::new(NoteValue::Quarter, 0, Some(Rational::new(3, 2))),
                        false
                    ),
                    (
                        Duration::new(NoteValue::Quarter, 0, Some(Rational::new(3, 2))),
                        true
                    ),
                    (Duration::new(NoteValue::Half, 0, None), false),
                ],
            );
        }
    }
}
