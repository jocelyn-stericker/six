#![allow(clippy::blacklisted_name)]

use crate::duration::Duration;
use crate::metre::{Metre, MetreSegment, Subdivision, Superdivision};
use crate::rhythmic_beaming::RhythmicBeaming;
use entity::{EntitiesRes, Entity};
use num_integer::Integer;
use num_rational::Rational;
use num_traits::{sign::Signed, One, Zero};
use std::collections::{BTreeSet, BinaryHeap};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
/// Why this this element created?
pub enum Lifetime {
    /// This is a rest that was automatically inserted by the program.
    ///
    /// This should be considered empty space, and can be replaced without explicit user intent.
    AutomaticRest,

    /// The user is hovering over a space and this is a preview.
    ///
    /// Moving the cursor will remove this note or rest.
    Temporary(Entity),

    /// The user has added this note.
    ///
    /// Explicit user intent is required to remove it.
    Explicit(Entity),
}

impl Lifetime {
    pub fn is_explicit(&self) -> bool {
        match self {
            Lifetime::Explicit(_) => true,
            _ => false,
        }
    }

    pub fn is_temporary(&self) -> bool {
        match self {
            Lifetime::Temporary(_) => true,
            _ => false,
        }
    }

    pub fn is_automatic(&self) -> bool {
        match self {
            Lifetime::AutomaticRest => true,
            _ => false,
        }
    }

    fn to_option(self) -> Option<Entity> {
        match self {
            Lifetime::AutomaticRest => None,
            Lifetime::Temporary(e) | Lifetime::Explicit(e) => Some(e),
        }
    }
}

impl Default for Lifetime {
    fn default() -> Self {
        Lifetime::AutomaticRest
    }
}

#[derive(Clone, Debug)]
/// The rhythm and metre of a voice in a single bar.
pub struct Bar {
    /// Time-signature and beat grouping spec.
    metre: Metre,

    /// The Rests, Notes, and Chords (RNCs) that fill up this bar.
    ///
    /// If the entity is None, that means it's an implicit rest that is managed
    /// by sys_update_rnc_timing.
    rhythm: Vec<(Duration, Lifetime)>,

    /// Automatically-generated ("managed") rests, managed by sys_update_rnc_timing.
    managed: Vec<Entity>,
}

impl Bar {
    /// Create a bar with a whole rest.
    pub fn new(metre: Metre) -> Bar {
        Bar {
            metre,
            rhythm: vec![],
            managed: vec![],
        }
    }

    pub fn whole_rest(&self) -> bool {
        self.rhythm.is_empty()
    }

    pub fn metre(&self) -> &Metre {
        &self.metre
    }

    pub fn rhythm(&self) -> &Vec<(Duration, Lifetime)> {
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
                self.rhythm
                    .push((Duration::exact(to - from, None), Lifetime::AutomaticRest));
            }
        }
    }

    fn try_amend(
        &self,
        division_start: Rational,
        amend_to: Option<&mut (Duration, Lifetime)>,
        t: Rational,
        duration: Rational,
    ) -> bool {
        if let Some(amend_to) = amend_to {
            let prior_note_t = t - amend_to.0.duration();
            if !amend_to.1.is_automatic() {
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

        false
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
        let mut new: Vec<(Duration, Lifetime)> = Vec::new();

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

            if existing_note.1.is_automatic()
                && t_read < next_division_start
                && next_division_start < existing_note_end
            {
                let new_duration = next_division_start - t_read;
                assert!(new_duration.is_positive());
                if !self.try_amend(division_start, new.last_mut(), t_read, new_duration) {
                    new.push((Duration::exact(new_duration, None), Lifetime::AutomaticRest));
                }
                // The time up to which we have written to `new`.
                let mut t_write = t_read + new_duration;
                t_read = existing_note_end;
                while t_write < t_read {
                    division_start = division_starts.next().unwrap();
                    next_division_start = *division_starts.peek().unwrap();

                    let p = t_read.min(next_division_start);
                    let new_duration = p - division_start;
                    if !self.try_amend(division_start, new.last_mut(), t_write, new_duration) {
                        new.push((Duration::exact(new_duration, None), Lifetime::AutomaticRest));
                    }

                    t_write = p;
                }

                assert_eq!(t_read, t_write);
            } else {
                if !existing_note.1.is_automatic()
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

        if self.rhythm.iter().any(|rhythm| !rhythm.1.is_automatic()) {
            self.rhythm = new;
        } else {
            self.rhythm.clear();
        }
    }

    /// For optimize(), get's how "important" each subdivision is.
    fn get_powers(
        &self,
        div_parts: usize,
        quant: Rational,
        tuplet_kind: Rational,
        div: MetreSegment,
        beats_exposed: BTreeSet<u8>,
    ) -> Vec<Rational> {
        let mut powers: Vec<Rational> = (0..div_parts).map(|_| Rational::zero()).collect();
        for p in 1..=div_parts {
            if div_parts % p != 0 {
                continue;
            }
            if Duration::exact(quant * (p as isize) * tuplet_kind, Some(tuplet_kind)).display_dots()
                != Some(0)
            {
                continue;
            }
            for (i, power) in powers.iter_mut().enumerate() {
                if i % p == 0 {
                    if div_parts / p == 3 {
                        *power += match (i / p) % 3 {
                            0 => {
                                if div.subdivision() == Subdivision::Compound {
                                    Rational::new(22, 20)
                                } else {
                                    Rational::new(21, 20)
                                }
                            }
                            1 => Rational::new(21, 20),
                            2 => {
                                if beats_exposed.contains(&1) && tuplet_kind.is_one() {
                                    Rational::one()
                                } else {
                                    Rational::new(21, 20)
                                }
                            }
                            _ => unreachable!(),
                        };
                    } else {
                        *power += 1;
                    }
                }
            }
        }

        powers
    }

    /// Split implicit rests in a reasonable way.
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

        #[derive(Ord, PartialOrd, Eq, PartialEq)]
        struct PartialSolution {
            score: Rational,
            done_time: Rational,
            todo: Vec<(Duration, Lifetime)>,
            done: Vec<(Duration, Lifetime)>,
        }

        let mut q: BinaryHeap<PartialSolution> = BinaryHeap::new();

        q.push(PartialSolution {
            score: Rational::zero(),
            done_time: Rational::zero(),
            todo: self.rhythm.clone(),
            done: vec![],
        });

        while let Some(PartialSolution {
            score,
            done_time,
            todo: input,
            done: mut output,
        }) = q.pop()
        {
            if let Some((first, remainder)) = input.split_first() {
                if !first.1.is_automatic() {
                    output.push(*first);
                    q.push(PartialSolution {
                        score,
                        done_time: done_time + first.0.duration(),
                        todo: remainder.to_vec(),
                        done: output,
                    });
                } else {
                    let quants = (first.0.duration() / quant).to_integer();
                    for i in (1..=quants).rev() {
                        for tuplet_kind in &tuplet_kinds {
                            let mut output = output.clone();
                            let (div_start, div) = self.metre().division(done_time);
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
                            let new_input: Vec<(Duration, Lifetime)> = if remainder_t
                                > Rational::zero()
                            {
                                let mut x: Vec<(Duration, Lifetime)> = input.to_vec();
                                x[0].0 =
                                    Duration::exact(remainder_t * tuplet_kind, Some(*tuplet_kind));
                                x
                            } else {
                                input.iter().skip(1).cloned().collect()
                            };
                            output.push((displayed, Lifetime::AutomaticRest));
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
                                            } else if t_beat == t_next
                                                && note.1.is_automatic()
                                                && t == div_start
                                            {
                                                beats_exposed.insert(i);
                                            }
                                        }
                                    }
                                    t = t_next;
                                }
                            }

                            if div.subdivision() == Subdivision::Compound
                                && done_time <= second_div
                                && second_div <= done_time + dur
                                && beats_exposed.contains(&2)
                                && beats_covered.contains(&1)
                            {
                                // If we can do it for one fewer rest, it's worth it.
                                score -= Rational::new(99, 100);
                            }

                            {
                                let div_parts = (div.duration() / quant).to_integer() as usize;
                                let start_q =
                                    ((done_time - div_start) / quant).to_integer() as usize;
                                let end_q =
                                    ((done_time + dur - div_start) / quant).to_integer() as usize;
                                if end_q <= div_parts {
                                    let powers = self.get_powers(
                                        div_parts,
                                        quant,
                                        *tuplet_kind,
                                        div,
                                        beats_exposed,
                                    );
                                    for q in (start_q + 1)..end_q {
                                        if powers[q as usize] >= powers[start_q] {
                                            score -= (Rational::one() + powers[q as usize]
                                                - powers[start_q])
                                                * 2;
                                        }
                                    }
                                }
                            }

                            q.push(PartialSolution {
                                score,
                                done_time: done_time + dur,
                                todo: new_input,
                                done: output,
                            });
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
    pub fn splice(&mut self, splice_start: Rational, replacement: Vec<(Duration, Lifetime)>) {
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
                    new.push((Duration::exact(pad_end, None), Lifetime::AutomaticRest));
                    t_write += pad_end;
                }

                assert!(t_write >= t_read);
            } else if t_read < t_write {
                let new_duration = existing_note_end - t_write;

                if new_duration.is_positive() {
                    // We are splicing only the start of this note. We are creating a new rest with
                    // the remaining duration.
                    new.push((Duration::exact(new_duration, None), Lifetime::AutomaticRest));
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

    pub fn remove(&mut self, rnc: Entity) -> Option<Lifetime> {
        let mut ret = None;

        for (_, entity) in &mut self.rhythm {
            if entity.to_option() == Some(rnc) {
                ret = Some(*entity);
                *entity = Lifetime::AutomaticRest;
            }
        }

        self.simplify();
        self.optimize();

        ret
    }

    /// Determine how a note at a given position time be spelled, rhythmically.
    pub fn split_note(&self, t: Rational, duration: Duration) -> Vec<Duration> {
        self.metre_split_note(t, duration, true)
    }

    fn metre_split_note(
        &self,
        t: Rational,
        mut duration: Duration,
        with_rhythm: bool,
    ) -> Vec<Duration> {
        if !duration.duration().is_positive() {
            return vec![];
        }

        let t_end = t + duration.duration();

        let mut existing_note_start = Rational::new(0, 1);
        if with_rhythm {
            for (existing_note, entity) in &self.rhythm {
                let existing_note_end = existing_note_start + existing_note.duration();
                if entity.is_explicit() {
                    if existing_note_start >= t && existing_note_start < t_end {
                        duration = Duration::exact(existing_note_start - t, None);
                        break;
                    }
                    if existing_note_start < t && existing_note_end > t {
                        return vec![];
                    }
                }

                existing_note_start = existing_note_end;
            }
        }

        // TODO(joshuan): Split results up into printable bits.
        // TODO(joshuan): p166-168
        let (div_start, mut segment) = self.metre.division(t);

        fn to_tuple(d: Rational) -> (isize, isize) {
            (*d.numer(), *d.denom())
        }

        // Check for common syncopation (Behind Bars, p. 171)
        match (
            to_tuple(div_start),
            to_tuple(segment.duration()),
            segment.subdivisions(),
            segment.superdivision(),
            to_tuple(t - div_start),
            to_tuple(duration.duration()),
        ) {
            (_, (1, 4), 1, Superdivision::Duple, (1, 8), (1, 4))
            | (_, (1, 4), 1, Superdivision::Duple, (1, 8), (3, 8))
            | ((0, 1), (1, 4), 1, Superdivision::Triple, (1, 8), (1, 4))
            | (_, (1, 2), 2, Superdivision::Duple, (1, 4), (1, 2))
            | (_, (1, 2), 2, Superdivision::Duple, (1, 4), (3, 4)) => {
                return vec![duration];
            }
            (_, _, _, _, _, _) => {
                // No match.
            }
        }

        if t == div_start {
            if segment.subdivision() == Subdivision::Simple {
                return vec![duration];
            } else {
                // Fill as many full segments as possible.
                let mut t_end = t;
                loop {
                    let t2 = t_end + segment.duration();
                    if t2 > t + duration.duration() || !Duration::exact(t2 - t, None).printable() {
                        break;
                    }
                    let next = self.metre.division(t2);
                    segment = next.1;
                    t_end = t2;
                }
                if t_end > t {
                    let mut split = vec![Duration::exact(t_end - t, None)];
                    split.append(&mut self.metre_split_note(
                        t_end,
                        Duration::exact(duration.duration() - t_end, Some(duration.tuplet())),
                        with_rhythm,
                    ));
                    return split;
                }
            }
        }

        let div_end = div_start + segment.duration();
        if t + duration.duration() <= div_end {
            vec![duration]
        } else {
            let mut split = vec![Duration::exact(div_end - t, Some(duration.tuplet()))];
            split.append(&mut self.metre_split_note(
                div_end,
                Duration::exact(duration.duration() - (div_end - t), Some(duration.tuplet())),
                with_rhythm,
            ));

            split
        }
    }

    /// Determine how to beam several notes
    pub fn beaming(&self, t0: Rational, durations: Vec<Duration>) -> Vec<Option<RhythmicBeaming>> {
        // TODO: For now, we're going to assume beaming follows the same rules as long-note
        // splitting. This isn't true, but it's a start.
        let mut beams = Vec::with_capacity(durations.len());
        let mut split_befores = Vec::with_capacity(durations.len());
        let long_note_split = self.metre_split_note(
            t0,
            Duration::exact(
                durations.iter().map(|dur| dur.duration()).sum(),
                // TODO: tuplets
                None,
            ),
            false,
        );

        let mut t = t0;
        for split in &long_note_split {
            let t_end = t + split.duration();

            let mut t_candidate = t0;
            let mut first_in_beam = true;
            for (i, candidate) in durations.iter().enumerate() {
                // Allows overshot.
                if t_candidate < t_end && i >= beams.len() {
                    if self
                        .metre()
                        .on_division(t_candidate)
                        .map(|t| t.subdivisions() >= 2)
                        .unwrap_or(false)
                    {
                        // 4/8, 4/4, and compound time must not cross beat.
                        first_in_beam = true;
                    }
                    beams.push(
                        candidate
                            .duration_display_base()
                            .map(|val| val.beam_count())
                            .unwrap_or(0),
                    );
                    split_befores.push(first_in_beam);
                    first_in_beam = false;
                }
                t_candidate += candidate.duration();
            }

            t = t_end;
        }

        let mut beaming = Vec::with_capacity(durations.len());

        for (i, (&beam, &split_before)) in beams.iter().zip(&split_befores).enumerate() {
            beaming.push(if beam == 0 {
                None
            } else {
                let prev = if !split_before {
                    assert_ne!(i, 0);
                    beams.get(i - 1).copied().unwrap_or(0)
                } else {
                    0
                };

                let split_after = split_befores.get(i + 1).copied().unwrap_or(true);
                let next = if !split_after {
                    beams.get(i + 1).copied().unwrap_or(0)
                } else {
                    0
                };

                if prev == 0 && next == 0 {
                    None
                } else {
                    Some(RhythmicBeaming {
                        entering: if prev > 0 { beam } else { 0 },
                        leaving: if next > 0 { beam } else { 0 },
                    })
                }
            });
        }

        beaming
    }

    fn target_managed_count(&self) -> usize {
        if self.whole_rest() {
            return 1;
        }

        let mut target_managed_count = 0;
        for rnc in &self.rhythm {
            if rnc.1.is_automatic() {
                target_managed_count += 1;
            }
        }

        target_managed_count
    }

    /// If target_managed_count < managed_count, create a new managed entity.
    ///
    /// Returns the duration and ID for the created managed entity.
    pub fn push_managed_entity(&mut self, entities: &EntitiesRes) -> Option<(Duration, Entity)> {
        let mut managed_idx = self.managed.len();

        if self.whole_rest() && managed_idx == 0 {
            let entity = entities.create();
            self.managed.push(entity);
            return Some((Duration::new_whole_rest(self.metre.duration()), entity));
        }

        for note in &self.rhythm {
            if note.1.is_automatic() {
                if managed_idx == 0 {
                    let entity = entities.create();
                    self.managed.push(entity);
                    return Some((note.0, entity));
                }
                managed_idx -= 1;
            }
        }

        None
    }

    /// If targed_managed_count > managed_count, remove a new managed entity.
    ///
    /// Returns the ID for the created managed entity.
    pub fn pop_managed_entity(&mut self) -> Option<Entity> {
        if self.target_managed_count() < self.managed.len() {
            return self.managed.pop();
        }

        None
    }

    pub fn managed(&self) -> &Vec<Entity> {
        &self.managed
    }

    /// Rest/note/chords (RNCs)
    pub fn children(&self) -> Vec<(Duration, Rational, Entity, bool)> {
        let mut managed = self.managed().iter();
        let mut start = Rational::zero();

        if self.whole_rest() {
            return vec![(
                Duration::new_whole_rest(self.metre.duration()),
                start,
                *managed.next().unwrap(),
                true,
            )];
        }

        self.rhythm
            .iter()
            .map(|(rhy, entity)| {
                let data = (
                    *rhy,
                    start,
                    entity
                        .to_option()
                        .unwrap_or_else(|| *managed.next().unwrap()),
                    entity.is_automatic(),
                );

                start += rhy.duration();

                data
            })
            .collect()
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
                vec![(
                    Duration::new(NoteValue::Quarter, 0, None),
                    Lifetime::Explicit(Entity::new(1)),
                )],
            );
            assert_eq!(
                bar.rhythm(),
                &vec![
                    (
                        Duration::new(NoteValue::Quarter, 0, None),
                        Lifetime::Explicit(Entity::new(1))
                    ),
                    (
                        Duration::new(NoteValue::Quarter, 0, None),
                        Lifetime::AutomaticRest
                    ),
                    (
                        Duration::new(NoteValue::Half, 0, None),
                        Lifetime::AutomaticRest
                    ),
                ],
            );
            assert!(!bar.whole_rest());
        }
        {
            let mut bar = Bar::new(Metre::new(4, 4));
            bar.splice(
                Rational::new(1, 4),
                vec![(
                    Duration::new(NoteValue::Quarter, 0, None),
                    Lifetime::Explicit(Entity::new(1)),
                )],
            );
            assert_eq!(
                bar.rhythm(),
                &vec![
                    (
                        Duration::new(NoteValue::Quarter, 0, None),
                        Lifetime::AutomaticRest
                    ),
                    (
                        Duration::new(NoteValue::Quarter, 0, None),
                        Lifetime::Explicit(Entity::new(1))
                    ),
                    (
                        Duration::new(NoteValue::Half, 0, None),
                        Lifetime::AutomaticRest
                    ),
                ],
            );
        }
        {
            let mut bar = Bar::new(Metre::new(4, 4));
            bar.splice(
                Rational::new(1, 2),
                vec![(
                    Duration::new(NoteValue::Quarter, 0, None),
                    Lifetime::Explicit(Entity::new(1)),
                )],
            );
            assert_eq!(
                bar.rhythm(),
                &vec![
                    (
                        Duration::new(NoteValue::Half, 0, None),
                        Lifetime::AutomaticRest
                    ),
                    (
                        Duration::new(NoteValue::Quarter, 0, None),
                        Lifetime::Explicit(Entity::new(1))
                    ),
                    (
                        Duration::new(NoteValue::Quarter, 0, None),
                        Lifetime::AutomaticRest
                    ),
                ],
            );
        }
        {
            let mut bar = Bar::new(Metre::new(4, 4));
            bar.splice(
                Rational::new(3, 4),
                vec![
                    (
                        Duration::new(NoteValue::Quarter, 0, None),
                        Lifetime::Explicit(Entity::new(1)),
                    ),
                    (
                        Duration::new(NoteValue::Quarter, 0, None),
                        Lifetime::Explicit(Entity::new(1)),
                    ),
                ],
            );
            assert_eq!(
                bar.rhythm(),
                &vec![
                    (
                        Duration::new(NoteValue::Half, 0, None),
                        Lifetime::AutomaticRest
                    ),
                    (
                        Duration::new(NoteValue::Quarter, 0, None),
                        Lifetime::AutomaticRest
                    ),
                    (
                        Duration::new(NoteValue::Quarter, 0, None),
                        Lifetime::Explicit(Entity::new(1))
                    ),
                ],
            );
        }
        {
            let mut bar = Bar::new(Metre::new(4, 4));
            assert!(bar.whole_rest());
            bar.splice(
                Rational::new(4, 4),
                vec![(
                    Duration::new(NoteValue::Quarter, 0, None),
                    Lifetime::Explicit(Entity::new(1)),
                )],
            );
            assert_eq!(bar.rhythm(), &vec![]);
            assert!(bar.whole_rest());
        }
        {
            let mut bar = Bar::new(Metre::new(4, 4));
            bar.splice(
                Rational::new(1, 4),
                vec![(
                    Duration::new(NoteValue::Half, 0, None),
                    Lifetime::Explicit(Entity::new(1)),
                )],
            );
            assert_eq!(
                bar.rhythm(),
                &vec![
                    (
                        Duration::new(NoteValue::Quarter, 0, None),
                        Lifetime::AutomaticRest
                    ),
                    (
                        Duration::new(NoteValue::Half, 0, None),
                        Lifetime::Explicit(Entity::new(1))
                    ),
                    (
                        Duration::new(NoteValue::Quarter, 0, None),
                        Lifetime::AutomaticRest
                    ),
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
                vec![(
                    Duration::new(NoteValue::Half, 0, None),
                    Lifetime::Explicit(Entity::new(1)),
                )],
            );
            bar.splice(
                Rational::new(1, 4),
                vec![(
                    Duration::new(NoteValue::Half, 0, None),
                    Lifetime::AutomaticRest,
                )],
            );
            assert_eq!(bar.rhythm(), &vec![],);
        }

        {
            let mut bar = Bar::new(Metre::new(4, 4));
            bar.splice(
                Rational::new(1, 4),
                vec![(
                    Duration::new(NoteValue::Half, 0, None),
                    Lifetime::Explicit(Entity::new(1)),
                )],
            );
            bar.splice(
                Rational::new(3, 8),
                vec![(
                    Duration::new(NoteValue::Eighth, 0, None),
                    Lifetime::AutomaticRest,
                )],
            );
            assert_eq!(
                bar.rhythm(),
                &vec![
                    (
                        Duration::new(NoteValue::Quarter, 0, None),
                        Lifetime::AutomaticRest
                    ),
                    (
                        Duration::new(NoteValue::Eighth, 0, None),
                        Lifetime::Explicit(Entity::new(1))
                    ),
                    (
                        Duration::new(NoteValue::Eighth, 0, None),
                        Lifetime::AutomaticRest
                    ),
                    (
                        Duration::new(NoteValue::Half, 0, None),
                        Lifetime::AutomaticRest
                    ),
                ],
            );
        }

        {
            let mut bar = Bar::new(Metre::new(4, 4));
            bar.splice(
                Rational::new(1, 4),
                vec![(
                    Duration::new(NoteValue::Half, 0, None),
                    Lifetime::Explicit(Entity::new(1)),
                )],
            );
            bar.splice(
                Rational::new(5, 8),
                vec![(
                    Duration::new(NoteValue::Eighth, 0, None),
                    Lifetime::AutomaticRest,
                )],
            );
            // This is not a good rhythm, but we do not adjust explicit rhythms.
            assert_eq!(
                bar.rhythm(),
                &vec![
                    (
                        Duration::new(NoteValue::Quarter, 0, None),
                        Lifetime::AutomaticRest
                    ),
                    (
                        Duration::new(NoteValue::Quarter, 1, None),
                        Lifetime::Explicit(Entity::new(1))
                    ),
                    (
                        Duration::new(NoteValue::Eighth, 0, None),
                        Lifetime::AutomaticRest
                    ),
                    (
                        Duration::new(NoteValue::Quarter, 0, None),
                        Lifetime::AutomaticRest
                    ),
                ],
            );
        }

        {
            let mut bar = Bar::new(Metre::new(4, 4));
            bar.splice(
                Rational::new(0, 1),
                vec![(
                    Duration::new(NoteValue::Half, 0, None),
                    Lifetime::Explicit(Entity::new(1)),
                )],
            );
            bar.splice(
                Rational::new(1, 8),
                vec![(
                    Duration::new(NoteValue::Eighth, 0, None),
                    Lifetime::Explicit(Entity::new(1)),
                )],
            );
            bar.splice(
                Rational::new(2, 8),
                vec![(
                    Duration::new(NoteValue::Eighth, 0, None),
                    Lifetime::Explicit(Entity::new(1)),
                )],
            );
            bar.splice(
                Rational::new(3, 8),
                vec![(
                    Duration::new(NoteValue::Eighth, 0, None),
                    Lifetime::Explicit(Entity::new(1)),
                )],
            );
            bar.splice(
                Rational::new(1, 8),
                vec![(
                    Duration::new(NoteValue::Eighth, 0, None),
                    Lifetime::AutomaticRest,
                )],
            );
            bar.splice(
                Rational::new(2, 8),
                vec![(
                    Duration::new(NoteValue::Eighth, 0, None),
                    Lifetime::AutomaticRest,
                )],
            );
            bar.splice(
                Rational::new(3, 8),
                vec![(
                    Duration::new(NoteValue::Eighth, 0, None),
                    Lifetime::AutomaticRest,
                )],
            );
            assert_eq!(
                bar.rhythm(),
                &vec![
                    (
                        Duration::new(NoteValue::Eighth, 0, None),
                        Lifetime::Explicit(Entity::new(1))
                    ),
                    (
                        Duration::new(NoteValue::Eighth, 0, None),
                        Lifetime::AutomaticRest
                    ),
                    (
                        Duration::new(NoteValue::Quarter, 0, None),
                        Lifetime::AutomaticRest
                    ),
                    (
                        Duration::new(NoteValue::Half, 0, None),
                        Lifetime::AutomaticRest
                    ),
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
                vec![(
                    Duration::new(NoteValue::Eighth, 0, None),
                    Lifetime::Explicit(Entity::new(1)),
                )],
            );
            assert_eq!(
                bar.rhythm(),
                &vec![
                    (
                        Duration::new(NoteValue::Quarter, 0, None),
                        Lifetime::AutomaticRest
                    ),
                    (
                        Duration::new(NoteValue::Eighth, 0, None),
                        Lifetime::AutomaticRest
                    ),
                    (
                        Duration::new(NoteValue::Eighth, 0, None),
                        Lifetime::Explicit(Entity::new(1))
                    ),
                ],
            );
        }

        for bar in &mut [Bar::new(Metre::new(4, 8)), Bar::new(Metre::new(2, 4))] {
            bar.splice(
                Rational::zero(),
                vec![(
                    Duration::new(NoteValue::Eighth, 0, None),
                    Lifetime::Explicit(Entity::new(1)),
                )],
            );
            bar.splice(
                Rational::new(3, 8),
                vec![(
                    Duration::new(NoteValue::Eighth, 0, None),
                    Lifetime::Explicit(Entity::new(1)),
                )],
            );
            assert_eq!(
                bar.rhythm(),
                &vec![
                    (
                        Duration::new(NoteValue::Eighth, 0, None),
                        Lifetime::Explicit(Entity::new(1))
                    ),
                    (
                        Duration::new(NoteValue::Eighth, 0, None),
                        Lifetime::AutomaticRest
                    ),
                    (
                        Duration::new(NoteValue::Eighth, 0, None),
                        Lifetime::AutomaticRest
                    ),
                    (
                        Duration::new(NoteValue::Eighth, 0, None),
                        Lifetime::Explicit(Entity::new(1))
                    ),
                ],
            );
        }
        for bar in &mut [Bar::new(Metre::new(4, 8)), Bar::new(Metre::new(2, 4))] {
            bar.splice(
                Rational::zero(),
                vec![(
                    Duration::new(NoteValue::Eighth, 0, None),
                    Lifetime::Explicit(Entity::new(1)),
                )],
            );
            assert_eq!(
                bar.rhythm(),
                &vec![
                    (
                        Duration::new(NoteValue::Eighth, 0, None),
                        Lifetime::Explicit(Entity::new(1))
                    ),
                    (
                        Duration::new(NoteValue::Eighth, 0, None),
                        Lifetime::AutomaticRest
                    ),
                    (
                        Duration::new(NoteValue::Quarter, 0, None),
                        Lifetime::AutomaticRest
                    ),
                ],
            );
        }

        for bar in &mut [Bar::new(Metre::new(4, 4)), Bar::new(Metre::new(2, 2))] {
            bar.splice(
                Rational::new(3, 4),
                vec![(
                    Duration::new(NoteValue::Quarter, 0, None),
                    Lifetime::Explicit(Entity::new(1)),
                )],
            );
            assert_eq!(
                bar.rhythm(),
                &vec![
                    (
                        Duration::new(NoteValue::Half, 0, None),
                        Lifetime::AutomaticRest
                    ),
                    (
                        Duration::new(NoteValue::Quarter, 0, None),
                        Lifetime::AutomaticRest
                    ),
                    (
                        Duration::new(NoteValue::Quarter, 0, None),
                        Lifetime::Explicit(Entity::new(1))
                    ),
                ],
            );
        }

        for bar in &mut [Bar::new(Metre::new(4, 4)), Bar::new(Metre::new(2, 2))] {
            bar.splice(
                Rational::zero(),
                vec![(
                    Duration::new(NoteValue::Quarter, 0, None),
                    Lifetime::Explicit(Entity::new(1)),
                )],
            );
            bar.splice(
                Rational::new(3, 4),
                vec![(
                    Duration::new(NoteValue::Quarter, 0, None),
                    Lifetime::Explicit(Entity::new(1)),
                )],
            );
            assert_eq!(
                bar.rhythm(),
                &vec![
                    (
                        Duration::new(NoteValue::Quarter, 0, None),
                        Lifetime::Explicit(Entity::new(1))
                    ),
                    (
                        Duration::new(NoteValue::Quarter, 0, None),
                        Lifetime::AutomaticRest
                    ),
                    (
                        Duration::new(NoteValue::Quarter, 0, None),
                        Lifetime::AutomaticRest
                    ),
                    (
                        Duration::new(NoteValue::Quarter, 0, None),
                        Lifetime::Explicit(Entity::new(1))
                    ),
                ],
            );
        }
        for bar in &mut [Bar::new(Metre::new(4, 4)), Bar::new(Metre::new(2, 2))] {
            bar.splice(
                Rational::zero(),
                vec![(
                    Duration::new(NoteValue::Quarter, 0, None),
                    Lifetime::Explicit(Entity::new(1)),
                )],
            );
            assert_eq!(
                bar.rhythm(),
                &vec![
                    (
                        Duration::new(NoteValue::Quarter, 0, None),
                        Lifetime::Explicit(Entity::new(1))
                    ),
                    (
                        Duration::new(NoteValue::Quarter, 0, None),
                        Lifetime::AutomaticRest
                    ),
                    (
                        Duration::new(NoteValue::Half, 0, None),
                        Lifetime::AutomaticRest
                    ),
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
                vec![(
                    Duration::new(NoteValue::Sixteenth, 0, None),
                    Lifetime::Explicit(Entity::new(1)),
                )],
            );
            assert_eq!(
                bar.rhythm(),
                &vec![
                    (
                        Duration::new(NoteValue::Eighth, 1, None),
                        Lifetime::AutomaticRest
                    ),
                    (
                        Duration::new(NoteValue::Eighth, 0, None),
                        Lifetime::AutomaticRest
                    ),
                    (
                        Duration::new(NoteValue::Sixteenth, 0, None),
                        Lifetime::Explicit(Entity::new(1))
                    ),
                ],
            );
        }
        {
            let mut bar = Bar::new(Metre::new(6, 16));
            bar.splice(
                Rational::zero(),
                vec![(
                    Duration::new(NoteValue::Eighth, 0, None),
                    Lifetime::Explicit(Entity::new(1)),
                )],
            );
            bar.splice(
                Rational::new(4, 16),
                vec![(
                    Duration::new(NoteValue::Eighth, 0, None),
                    Lifetime::Explicit(Entity::new(1)),
                )],
            );
            assert_eq!(
                bar.rhythm(),
                &vec![
                    (
                        Duration::new(NoteValue::Eighth, 0, None),
                        Lifetime::Explicit(Entity::new(1))
                    ),
                    (
                        Duration::new(NoteValue::Sixteenth, 0, None),
                        Lifetime::AutomaticRest
                    ),
                    (
                        Duration::new(NoteValue::Sixteenth, 0, None),
                        Lifetime::AutomaticRest
                    ),
                    (
                        Duration::new(NoteValue::Eighth, 0, None),
                        Lifetime::Explicit(Entity::new(1))
                    ),
                ],
            );
        }
        {
            let mut bar = Bar::new(Metre::new(6, 16));
            bar.splice(
                Rational::zero(),
                vec![(
                    Duration::new(NoteValue::Sixteenth, 0, None),
                    Lifetime::Explicit(Entity::new(1)),
                )],
            );
            assert_eq!(
                bar.rhythm(),
                &vec![
                    (
                        Duration::new(NoteValue::Sixteenth, 0, None),
                        Lifetime::Explicit(Entity::new(1))
                    ),
                    (
                        Duration::new(NoteValue::Sixteenth, 0, None),
                        Lifetime::AutomaticRest
                    ),
                    (
                        Duration::new(NoteValue::Sixteenth, 0, None),
                        Lifetime::AutomaticRest
                    ),
                    (
                        Duration::new(NoteValue::Eighth, 1, None),
                        Lifetime::AutomaticRest
                    ),
                ],
            );
        }

        {
            let mut bar = Bar::new(Metre::new(6, 8));
            bar.splice(
                Rational::new(5, 8),
                vec![(
                    Duration::new(NoteValue::Eighth, 0, None),
                    Lifetime::Explicit(Entity::new(1)),
                )],
            );
            assert_eq!(
                bar.rhythm(),
                &vec![
                    (
                        Duration::new(NoteValue::Quarter, 1, None),
                        Lifetime::AutomaticRest
                    ),
                    (
                        Duration::new(NoteValue::Quarter, 0, None),
                        Lifetime::AutomaticRest
                    ),
                    (
                        Duration::new(NoteValue::Eighth, 0, None),
                        Lifetime::Explicit(Entity::new(1))
                    ),
                ],
            );
        }
        {
            let mut bar = Bar::new(Metre::new(6, 8));
            bar.splice(
                Rational::zero(),
                vec![(
                    Duration::new(NoteValue::Quarter, 0, None),
                    Lifetime::Explicit(Entity::new(1)),
                )],
            );
            bar.splice(
                Rational::new(4, 8),
                vec![(
                    Duration::new(NoteValue::Quarter, 0, None),
                    Lifetime::Explicit(Entity::new(1)),
                )],
            );
            assert_eq!(
                bar.rhythm(),
                &vec![
                    (
                        Duration::new(NoteValue::Quarter, 0, None),
                        Lifetime::Explicit(Entity::new(1))
                    ),
                    (
                        Duration::new(NoteValue::Eighth, 0, None),
                        Lifetime::AutomaticRest
                    ),
                    (
                        Duration::new(NoteValue::Eighth, 0, None),
                        Lifetime::AutomaticRest
                    ),
                    (
                        Duration::new(NoteValue::Quarter, 0, None),
                        Lifetime::Explicit(Entity::new(1))
                    ),
                ],
            );
        }
        {
            let mut bar = Bar::new(Metre::new(6, 8));
            bar.splice(
                Rational::zero(),
                vec![(
                    Duration::new(NoteValue::Eighth, 0, None),
                    Lifetime::Explicit(Entity::new(1)),
                )],
            );
            assert_eq!(
                bar.rhythm(),
                &vec![
                    (
                        Duration::new(NoteValue::Eighth, 0, None),
                        Lifetime::Explicit(Entity::new(1))
                    ),
                    (
                        Duration::new(NoteValue::Eighth, 0, None),
                        Lifetime::AutomaticRest
                    ),
                    (
                        Duration::new(NoteValue::Eighth, 0, None),
                        Lifetime::AutomaticRest
                    ),
                    (
                        Duration::new(NoteValue::Quarter, 1, None),
                        Lifetime::AutomaticRest
                    ),
                ],
            );
        }

        {
            let mut bar = Bar::new(Metre::new(6, 4));
            bar.splice(
                Rational::new(5, 4),
                vec![(
                    Duration::new(NoteValue::Quarter, 0, None),
                    Lifetime::Explicit(Entity::new(1)),
                )],
            );
            assert_eq!(
                bar.rhythm(),
                &vec![
                    (
                        Duration::new(NoteValue::Half, 1, None),
                        Lifetime::AutomaticRest
                    ),
                    (
                        Duration::new(NoteValue::Half, 0, None),
                        Lifetime::AutomaticRest
                    ),
                    (
                        Duration::new(NoteValue::Quarter, 0, None),
                        Lifetime::Explicit(Entity::new(1))
                    ),
                ],
            );
        }
        {
            let mut bar = Bar::new(Metre::new(6, 4));
            bar.splice(
                Rational::zero(),
                vec![(
                    Duration::new(NoteValue::Half, 0, None),
                    Lifetime::Explicit(Entity::new(1)),
                )],
            );
            bar.splice(
                Rational::new(4, 4),
                vec![(
                    Duration::new(NoteValue::Half, 0, None),
                    Lifetime::Explicit(Entity::new(1)),
                )],
            );
            assert_eq!(
                bar.rhythm(),
                &vec![
                    (
                        Duration::new(NoteValue::Half, 0, None),
                        Lifetime::Explicit(Entity::new(1))
                    ),
                    (
                        Duration::new(NoteValue::Quarter, 0, None),
                        Lifetime::AutomaticRest
                    ),
                    (
                        Duration::new(NoteValue::Quarter, 0, None),
                        Lifetime::AutomaticRest
                    ),
                    (
                        Duration::new(NoteValue::Half, 0, None),
                        Lifetime::Explicit(Entity::new(1))
                    ),
                ],
            );
        }
        {
            let mut bar = Bar::new(Metre::new(6, 4));
            bar.splice(
                Rational::zero(),
                vec![(
                    Duration::new(NoteValue::Quarter, 0, None),
                    Lifetime::Explicit(Entity::new(1)),
                )],
            );
            assert_eq!(
                bar.rhythm(),
                &vec![
                    (
                        Duration::new(NoteValue::Quarter, 0, None),
                        Lifetime::Explicit(Entity::new(1))
                    ),
                    (
                        Duration::new(NoteValue::Quarter, 0, None),
                        Lifetime::AutomaticRest
                    ),
                    (
                        Duration::new(NoteValue::Quarter, 0, None),
                        Lifetime::AutomaticRest
                    ),
                    (
                        Duration::new(NoteValue::Half, 1, None),
                        Lifetime::AutomaticRest
                    ),
                ],
            );
        }

        {
            let mut bar = Bar::new(Metre::new(12, 8));
            bar.splice(
                Rational::new(11, 8),
                vec![(
                    Duration::new(NoteValue::Eighth, 0, None),
                    Lifetime::Explicit(Entity::new(1)),
                )],
            );
            assert_eq!(
                bar.rhythm(),
                &vec![
                    (
                        Duration::new(NoteValue::Half, 1, None),
                        Lifetime::AutomaticRest
                    ),
                    (
                        Duration::new(NoteValue::Quarter, 1, None),
                        Lifetime::AutomaticRest
                    ),
                    (
                        Duration::new(NoteValue::Quarter, 0, None),
                        Lifetime::AutomaticRest
                    ),
                    (
                        Duration::new(NoteValue::Eighth, 0, None),
                        Lifetime::Explicit(Entity::new(1))
                    ),
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
                vec![(
                    Duration::new(NoteValue::Quarter, 0, None),
                    Lifetime::Explicit(Entity::new(1)),
                )],
            );
            assert_eq!(
                bar.rhythm(),
                &vec![
                    (
                        Duration::new(NoteValue::Quarter, 0, None),
                        Lifetime::AutomaticRest
                    ),
                    (
                        Duration::new(NoteValue::Quarter, 0, None),
                        Lifetime::AutomaticRest
                    ),
                    (
                        Duration::new(NoteValue::Quarter, 0, None),
                        Lifetime::Explicit(Entity::new(1))
                    ),
                ],
            );
        }

        {
            let mut bar = Bar::new(Metre::new(3, 8));
            bar.splice(
                Rational::new(2, 8),
                vec![(
                    Duration::new(NoteValue::Eighth, 0, None),
                    Lifetime::Explicit(Entity::new(1)),
                )],
            );
            assert_eq!(
                bar.rhythm(),
                &vec![
                    (
                        Duration::new(NoteValue::Eighth, 0, None),
                        Lifetime::AutomaticRest
                    ),
                    (
                        Duration::new(NoteValue::Eighth, 0, None),
                        Lifetime::AutomaticRest
                    ),
                    (
                        Duration::new(NoteValue::Eighth, 0, None),
                        Lifetime::Explicit(Entity::new(1))
                    ),
                ],
            );
        }

        {
            let mut bar = Bar::new(Metre::new(9, 8));
            bar.splice(
                Rational::new(6, 8),
                vec![(
                    Duration::new(NoteValue::Quarter, 1, None),
                    Lifetime::Explicit(Entity::new(1)),
                )],
            );
            assert_eq!(
                bar.rhythm(),
                &vec![
                    (
                        Duration::new(NoteValue::Quarter, 1, None),
                        Lifetime::AutomaticRest
                    ),
                    (
                        Duration::new(NoteValue::Quarter, 1, None),
                        Lifetime::AutomaticRest
                    ),
                    (
                        Duration::new(NoteValue::Quarter, 1, None),
                        Lifetime::Explicit(Entity::new(1))
                    ),
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
                vec![(
                    Duration::new(NoteValue::Sixteenth, 0, None),
                    Lifetime::Explicit(Entity::new(1)),
                )],
            );
            bar.splice(
                Rational::new(7, 16),
                vec![(
                    Duration::new(NoteValue::Sixteenth, 0, None),
                    Lifetime::Explicit(Entity::new(1)),
                )],
            );
            assert_eq!(
                bar.rhythm(),
                &vec![
                    (
                        Duration::new(NoteValue::Eighth, 1, None),
                        Lifetime::AutomaticRest
                    ),
                    (
                        Duration::new(NoteValue::Sixteenth, 0, None),
                        Lifetime::Explicit(Entity::new(1))
                    ),
                    (
                        Duration::new(NoteValue::Eighth, 1, None),
                        Lifetime::AutomaticRest
                    ),
                    (
                        Duration::new(NoteValue::Sixteenth, 0, None),
                        Lifetime::Explicit(Entity::new(1))
                    ),
                ],
            );
        }

        {
            let mut bar = Bar::new(Metre::new(9, 8));
            bar.splice(
                Rational::new(8, 8),
                vec![(
                    Duration::new(NoteValue::Eighth, 0, None),
                    Lifetime::Explicit(Entity::new(1)),
                )],
            );
            assert_eq!(
                bar.rhythm(),
                &vec![
                    (
                        Duration::new(NoteValue::Quarter, 1, None),
                        Lifetime::AutomaticRest
                    ),
                    (
                        Duration::new(NoteValue::Quarter, 1, None),
                        Lifetime::AutomaticRest
                    ),
                    (
                        Duration::new(NoteValue::Quarter, 0, None),
                        Lifetime::AutomaticRest
                    ),
                    (
                        Duration::new(NoteValue::Eighth, 0, None),
                        Lifetime::Explicit(Entity::new(1))
                    ),
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
                vec![(
                    Duration::new(NoteValue::Sixteenth, 0, None),
                    Lifetime::Explicit(Entity::new(1)),
                )],
            );
            bar.splice(
                Rational::new(4, 16),
                vec![(
                    Duration::new(NoteValue::Sixteenth, 0, None),
                    Lifetime::Explicit(Entity::new(1)),
                )],
            );
            assert_eq!(
                bar.rhythm(),
                &vec![
                    (
                        Duration::new(NoteValue::Eighth, 1, None),
                        Lifetime::AutomaticRest
                    ),
                    (
                        Duration::new(NoteValue::Sixteenth, 0, None),
                        Lifetime::Explicit(Entity::new(1))
                    ),
                    (
                        Duration::new(NoteValue::Sixteenth, 0, None),
                        Lifetime::Explicit(Entity::new(1))
                    ),
                    (
                        Duration::new(NoteValue::Sixteenth, 0, None),
                        Lifetime::AutomaticRest
                    ),
                    (
                        Duration::new(NoteValue::Eighth, 0, None),
                        Lifetime::AutomaticRest
                    ),
                ],
            );
        }
        {
            let mut bar = Bar::new(Metre::new(2, 2));
            bar.splice(
                Rational::new(3, 8),
                vec![(
                    Duration::new(NoteValue::Eighth, 0, None),
                    Lifetime::Explicit(Entity::new(1)),
                )],
            );
            bar.splice(
                Rational::new(4, 8),
                vec![(
                    Duration::new(NoteValue::Eighth, 0, None),
                    Lifetime::Explicit(Entity::new(1)),
                )],
            );
            assert_eq!(
                bar.rhythm(),
                &vec![
                    (
                        Duration::new(NoteValue::Quarter, 1, None),
                        Lifetime::AutomaticRest
                    ),
                    (
                        Duration::new(NoteValue::Eighth, 0, None),
                        Lifetime::Explicit(Entity::new(1))
                    ),
                    (
                        Duration::new(NoteValue::Eighth, 0, None),
                        Lifetime::Explicit(Entity::new(1))
                    ),
                    (
                        Duration::new(NoteValue::Eighth, 0, None),
                        Lifetime::AutomaticRest
                    ),
                    (
                        Duration::new(NoteValue::Quarter, 0, None),
                        Lifetime::AutomaticRest
                    ),
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
                vec![(
                    Duration::new(NoteValue::Eighth, 0, None),
                    Lifetime::Explicit(Entity::new(1)),
                )],
            );
            assert_eq!(
                bar.rhythm(),
                &vec![
                    (
                        Duration::new(NoteValue::Half, 0, None),
                        Lifetime::AutomaticRest
                    ),
                    (
                        Duration::new(NoteValue::Quarter, 0, None),
                        Lifetime::AutomaticRest
                    ),
                    (
                        Duration::new(NoteValue::Eighth, 0, None),
                        Lifetime::AutomaticRest
                    ),
                    (
                        Duration::new(NoteValue::Eighth, 0, None),
                        Lifetime::Explicit(Entity::new(1))
                    ),
                ],
            );
        }

        {
            let mut bar = Bar::new(Metre::new(4, 4));
            bar.splice(
                Rational::new(3, 16),
                vec![(
                    Duration::new(NoteValue::Sixteenth, 0, None),
                    Lifetime::Explicit(Entity::new(1)),
                )],
            );
            assert_eq!(
                bar.rhythm(),
                &vec![
                    (
                        Duration::new(NoteValue::Eighth, 1, None),
                        Lifetime::AutomaticRest
                    ),
                    (
                        Duration::new(NoteValue::Sixteenth, 0, None),
                        Lifetime::Explicit(Entity::new(1))
                    ),
                    (
                        Duration::new(NoteValue::Quarter, 0, None),
                        Lifetime::AutomaticRest
                    ),
                    (
                        Duration::new(NoteValue::Half, 0, None),
                        Lifetime::AutomaticRest
                    ),
                ],
            );
        }

        {
            let mut bar = Bar::new(Metre::new(2, 2));
            bar.splice(
                Rational::new(3, 8),
                vec![(
                    Duration::new(NoteValue::Eighth, 0, None),
                    Lifetime::Explicit(Entity::new(1)),
                )],
            );
            bar.splice(
                Rational::new(7, 8),
                vec![(
                    Duration::new(NoteValue::Eighth, 0, None),
                    Lifetime::Explicit(Entity::new(1)),
                )],
            );
            assert_eq!(
                bar.rhythm(),
                &vec![
                    (
                        Duration::new(NoteValue::Quarter, 1, None),
                        Lifetime::AutomaticRest
                    ),
                    (
                        Duration::new(NoteValue::Eighth, 0, None),
                        Lifetime::Explicit(Entity::new(1))
                    ),
                    (
                        Duration::new(NoteValue::Quarter, 1, None),
                        Lifetime::AutomaticRest
                    ),
                    (
                        Duration::new(NoteValue::Eighth, 0, None),
                        Lifetime::Explicit(Entity::new(1))
                    ),
                ],
            );
        }

        {
            let mut bar = Bar::new(Metre::new(2, 2));
            bar.splice(
                Rational::zero(),
                vec![(
                    Duration::new(NoteValue::Eighth, 0, None),
                    Lifetime::Explicit(Entity::new(1)),
                )],
            );
            assert_eq!(
                bar.rhythm(),
                &vec![
                    (
                        Duration::new(NoteValue::Eighth, 0, None),
                        Lifetime::Explicit(Entity::new(1))
                    ),
                    (
                        Duration::new(NoteValue::Eighth, 0, None),
                        Lifetime::AutomaticRest
                    ),
                    (
                        Duration::new(NoteValue::Quarter, 0, None),
                        Lifetime::AutomaticRest
                    ),
                    (
                        Duration::new(NoteValue::Half, 0, None),
                        Lifetime::AutomaticRest
                    ),
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
                vec![(
                    Duration::new(NoteValue::ThirtySecond, 0, None),
                    Lifetime::Explicit(Entity::new(1)),
                )],
            );
            assert_eq!(
                bar.rhythm(),
                &vec![
                    (
                        Duration::new(NoteValue::Eighth, 2, None),
                        Lifetime::AutomaticRest
                    ),
                    (
                        Duration::new(NoteValue::ThirtySecond, 0, None),
                        Lifetime::Explicit(Entity::new(1))
                    ),
                    (
                        Duration::new(NoteValue::Quarter, 0, None),
                        Lifetime::AutomaticRest
                    ),
                ],
            );
        }

        {
            let mut bar = Bar::new(Metre::new(2, 4));
            bar.splice(
                Rational::new(7, 32),
                vec![(
                    Duration::new(NoteValue::ThirtySecond, 0, None),
                    Lifetime::Explicit(Entity::new(1)),
                )],
            );
            bar.splice(
                Rational::new(8, 32),
                vec![(
                    Duration::new(NoteValue::ThirtySecond, 0, None),
                    Lifetime::Explicit(Entity::new(1)),
                )],
            );
            assert_eq!(
                bar.rhythm(),
                &vec![
                    (
                        Duration::new(NoteValue::Eighth, 2, None),
                        Lifetime::AutomaticRest
                    ),
                    (
                        Duration::new(NoteValue::ThirtySecond, 0, None),
                        Lifetime::Explicit(Entity::new(1))
                    ),
                    (
                        Duration::new(NoteValue::ThirtySecond, 0, None),
                        Lifetime::Explicit(Entity::new(1))
                    ),
                    (
                        Duration::new(NoteValue::ThirtySecond, 0, None),
                        Lifetime::AutomaticRest
                    ),
                    (
                        Duration::new(NoteValue::Sixteenth, 0, None),
                        Lifetime::AutomaticRest
                    ),
                    (
                        Duration::new(NoteValue::Eighth, 0, None),
                        Lifetime::AutomaticRest
                    ),
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
                vec![(
                    Duration::new(NoteValue::ThirtySecond, 0, None),
                    Lifetime::Explicit(Entity::new(1)),
                )],
            );
            bar.splice(
                Rational::new(7, 32),
                vec![(
                    Duration::new(NoteValue::ThirtySecond, 0, None),
                    Lifetime::Explicit(Entity::new(1)),
                )],
            );
            assert_eq!(
                bar.rhythm(),
                &vec![
                    (
                        Duration::new(NoteValue::ThirtySecond, 0, None),
                        Lifetime::Explicit(Entity::new(1))
                    ),
                    (
                        Duration::new(NoteValue::ThirtySecond, 0, None),
                        Lifetime::AutomaticRest
                    ),
                    (
                        Duration::new(NoteValue::Sixteenth, 0, None),
                        Lifetime::AutomaticRest
                    ),
                    (
                        Duration::new(NoteValue::Sixteenth, 1, None),
                        Lifetime::AutomaticRest
                    ),
                    (
                        Duration::new(NoteValue::ThirtySecond, 0, None),
                        Lifetime::Explicit(Entity::new(1))
                    ),
                    (
                        Duration::new(NoteValue::Quarter, 0, None),
                        Lifetime::AutomaticRest
                    ),
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
                vec![(
                    Duration::new(NoteValue::Quarter, 1, None),
                    Lifetime::Explicit(Entity::new(1)),
                )],
            );
            assert_eq!(
                bar.rhythm(),
                &vec![
                    (
                        Duration::new(NoteValue::Half, 1, None),
                        Lifetime::AutomaticRest
                    ),
                    (
                        Duration::new(NoteValue::Quarter, 1, None),
                        Lifetime::AutomaticRest
                    ),
                    (
                        Duration::new(NoteValue::Quarter, 1, None),
                        Lifetime::Explicit(Entity::new(1))
                    ),
                ],
            );
        }

        {
            let mut bar = Bar::new(Metre::new(12, 8));
            bar.splice(
                Rational::zero(),
                vec![(
                    Duration::new(NoteValue::Quarter, 1, None),
                    Lifetime::Explicit(Entity::new(1)),
                )],
            );
            assert_eq!(
                bar.rhythm(),
                &vec![
                    (
                        Duration::new(NoteValue::Quarter, 1, None),
                        Lifetime::Explicit(Entity::new(1))
                    ),
                    (
                        Duration::new(NoteValue::Quarter, 1, None),
                        Lifetime::AutomaticRest
                    ),
                    (
                        Duration::new(NoteValue::Half, 1, None),
                        Lifetime::AutomaticRest
                    ),
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
                vec![(
                    Duration::new(NoteValue::Eighth, 0, None),
                    Lifetime::Explicit(Entity::new(1)),
                )],
            );
            bar.splice(
                Rational::new(5, 8),
                vec![(
                    Duration::new(NoteValue::Eighth, 0, None),
                    Lifetime::Explicit(Entity::new(1)),
                )],
            );
            assert_eq!(
                bar.rhythm(),
                &vec![
                    (
                        Duration::new(NoteValue::Quarter, 0, None),
                        Lifetime::AutomaticRest
                    ),
                    (
                        Duration::new(NoteValue::Eighth, 0, None),
                        Lifetime::Explicit(Entity::new(1))
                    ),
                    (
                        Duration::new(NoteValue::Quarter, 0, None),
                        Lifetime::AutomaticRest
                    ),
                    (
                        Duration::new(NoteValue::Eighth, 0, None),
                        Lifetime::Explicit(Entity::new(1))
                    ),
                ],
            );
        }

        {
            let mut bar = Bar::new(Metre::new(6, 8));
            bar.splice(
                Rational::new(5, 16),
                vec![(
                    Duration::new(NoteValue::Sixteenth, 0, None),
                    Lifetime::Explicit(Entity::new(1)),
                )],
            );
            assert_eq!(
                bar.rhythm(),
                &vec![
                    (
                        Duration::new(NoteValue::Eighth, 0, None),
                        Lifetime::AutomaticRest
                    ),
                    (
                        Duration::new(NoteValue::Eighth, 1, None),
                        Lifetime::AutomaticRest
                    ),
                    (
                        Duration::new(NoteValue::Sixteenth, 0, None),
                        Lifetime::Explicit(Entity::new(1))
                    ),
                    (
                        Duration::new(NoteValue::Quarter, 1, None),
                        Lifetime::AutomaticRest
                    ),
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
                vec![(
                    Duration::new(NoteValue::Eighth, 0, None),
                    Lifetime::Explicit(Entity::new(1)),
                )],
            );
            bar.splice(
                Rational::new(3, 8),
                vec![(
                    Duration::new(NoteValue::Sixteenth, 0, None),
                    Lifetime::Explicit(Entity::new(1)),
                )],
            );
            assert_eq!(
                bar.rhythm(),
                &vec![
                    (
                        Duration::new(NoteValue::Eighth, 0, None),
                        Lifetime::Explicit(Entity::new(1))
                    ),
                    (
                        Duration::new(NoteValue::Eighth, 0, None),
                        Lifetime::AutomaticRest
                    ),
                    (
                        Duration::new(NoteValue::Eighth, 0, None),
                        Lifetime::AutomaticRest
                    ),
                    (
                        Duration::new(NoteValue::Sixteenth, 0, None),
                        Lifetime::Explicit(Entity::new(1))
                    ),
                    (
                        Duration::new(NoteValue::Sixteenth, 0, None),
                        Lifetime::AutomaticRest
                    ),
                    (
                        Duration::new(NoteValue::Eighth, 0, None),
                        Lifetime::AutomaticRest
                    ),
                    (
                        Duration::new(NoteValue::Eighth, 0, None),
                        Lifetime::AutomaticRest
                    ),
                ],
            );
        }
    }

    #[test]
    fn overfill() {
        let mut bar = Bar::new(Metre::new(4, 4));
        bar.splice(
            Rational::new(3, 4),
            vec![(
                Duration::new(NoteValue::Half, 0, None),
                Lifetime::Explicit(Entity::new(1)),
            )],
        );
        assert_eq!(
            bar.rhythm(),
            &vec![
                (
                    Duration::new(NoteValue::Half, 0, None),
                    Lifetime::AutomaticRest
                ),
                (
                    Duration::new(NoteValue::Quarter, 0, None),
                    Lifetime::AutomaticRest
                ),
                (
                    Duration::new(NoteValue::Quarter, 0, None),
                    Lifetime::Explicit(Entity::new(1))
                ),
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
                vec![(
                    Duration::new(NoteValue::Eighth, 0, None),
                    Lifetime::Explicit(Entity::new(1)),
                )],
            );
            bar.splice(
                Rational::new(11, 16),
                vec![(
                    Duration::new(NoteValue::Sixteenth, 0, None),
                    Lifetime::Explicit(Entity::new(1)),
                )],
            );
            assert_eq!(
                bar.rhythm(),
                &vec![
                    (
                        Duration::new(NoteValue::Quarter, 0, None),
                        Lifetime::AutomaticRest
                    ),
                    (
                        Duration::new(NoteValue::Eighth, 0, None),
                        Lifetime::Explicit(Entity::new(1))
                    ),
                    (
                        Duration::new(NoteValue::Eighth, 0, None),
                        Lifetime::AutomaticRest
                    ),
                    (
                        Duration::new(NoteValue::Eighth, 1, None),
                        Lifetime::AutomaticRest
                    ),
                    (
                        Duration::new(NoteValue::Sixteenth, 0, None),
                        Lifetime::Explicit(Entity::new(1))
                    ),
                ],
            );
        }

        // p164
        {
            let mut bar = Bar::new(Metre::new(9, 8));
            bar.splice(
                Rational::zero(),
                vec![(
                    Duration::new(NoteValue::Eighth, 0, None),
                    Lifetime::Explicit(Entity::new(1)),
                )],
            );
            bar.splice(
                Rational::new(5, 8),
                vec![(
                    Duration::new(NoteValue::Eighth, 0, None),
                    Lifetime::Explicit(Entity::new(1)),
                )],
            );
            bar.splice(
                Rational::new(8, 8),
                vec![(
                    Duration::new(NoteValue::Eighth, 0, None),
                    Lifetime::Explicit(Entity::new(1)),
                )],
            );
            assert_eq!(
                bar.rhythm(),
                &vec![
                    (
                        Duration::new(NoteValue::Eighth, 0, None),
                        Lifetime::Explicit(Entity::new(1))
                    ),
                    (
                        Duration::new(NoteValue::Eighth, 0, None),
                        Lifetime::AutomaticRest
                    ),
                    (
                        Duration::new(NoteValue::Eighth, 0, None),
                        Lifetime::AutomaticRest
                    ),
                    (
                        Duration::new(NoteValue::Quarter, 0, None),
                        Lifetime::AutomaticRest
                    ),
                    (
                        Duration::new(NoteValue::Eighth, 0, None),
                        Lifetime::Explicit(Entity::new(1))
                    ),
                    (
                        Duration::new(NoteValue::Quarter, 0, None),
                        Lifetime::AutomaticRest
                    ),
                    (
                        Duration::new(NoteValue::Eighth, 0, None),
                        Lifetime::Explicit(Entity::new(1))
                    ),
                ],
            );
        }

        {
            let mut bar = Bar::new(Metre::new(6, 8));
            bar.splice(
                Rational::zero(),
                vec![(
                    Duration::new(NoteValue::Sixteenth, 0, None),
                    Lifetime::Explicit(Entity::new(1)),
                )],
            );
            bar.splice(
                Rational::new(5, 16),
                vec![(
                    Duration::new(NoteValue::Sixteenth, 0, None),
                    Lifetime::Explicit(Entity::new(1)),
                )],
            );
            assert_eq!(
                bar.rhythm(),
                &vec![
                    (
                        Duration::new(NoteValue::Sixteenth, 0, None),
                        Lifetime::Explicit(Entity::new(1))
                    ),
                    (
                        Duration::new(NoteValue::Sixteenth, 0, None),
                        Lifetime::AutomaticRest
                    ),
                    // TODO: join?
                    (
                        Duration::new(NoteValue::Eighth, 0, None),
                        Lifetime::AutomaticRest
                    ),
                    (
                        Duration::new(NoteValue::Sixteenth, 0, None),
                        Lifetime::AutomaticRest
                    ),
                    (
                        Duration::new(NoteValue::Sixteenth, 0, None),
                        Lifetime::Explicit(Entity::new(1))
                    ),
                    (
                        Duration::new(NoteValue::Quarter, 1, None),
                        Lifetime::AutomaticRest
                    ),
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
                    Lifetime::Explicit(Entity::new(1)),
                )],
            );
            assert_eq!(
                bar.rhythm(),
                &vec![
                    (
                        Duration::new(NoteValue::Quarter, 0, Some(Rational::new(3, 2))),
                        Lifetime::Explicit(Entity::new(1))
                    ),
                    (
                        Duration::new(NoteValue::Quarter, 0, Some(Rational::new(3, 2))),
                        Lifetime::AutomaticRest
                    ),
                    (
                        Duration::new(NoteValue::Quarter, 0, Some(Rational::new(3, 2))),
                        Lifetime::AutomaticRest
                    ),
                    (
                        Duration::new(NoteValue::Half, 0, None),
                        Lifetime::AutomaticRest
                    ),
                ],
            );
        }
        {
            let mut bar = Bar::new(Metre::new(4, 4));
            bar.splice(
                Rational::new(2, 6),
                vec![(
                    Duration::new(NoteValue::Quarter, 0, Some(Rational::new(3, 2))),
                    Lifetime::Explicit(Entity::new(1)),
                )],
            );
            assert_eq!(
                bar.rhythm(),
                &vec![
                    (
                        Duration::new(NoteValue::Quarter, 0, Some(Rational::new(3, 2))),
                        Lifetime::AutomaticRest
                    ),
                    (
                        Duration::new(NoteValue::Quarter, 0, Some(Rational::new(3, 2))),
                        Lifetime::AutomaticRest
                    ),
                    (
                        Duration::new(NoteValue::Quarter, 0, Some(Rational::new(3, 2))),
                        Lifetime::Explicit(Entity::new(1))
                    ),
                    (
                        Duration::new(NoteValue::Half, 0, None),
                        Lifetime::AutomaticRest
                    ),
                ],
            );
        }
    }

    #[test]
    fn regression_splice_12_8_unprintable() {
        let mut bar = Bar::new(Metre::new(12, 8));
        bar.splice(
            Rational::zero(),
            vec![(
                Duration::new(NoteValue::Eighth, 0, None),
                Lifetime::Explicit(Entity::new(1)),
            )],
        );
        bar.splice(
            Rational::new(11, 8),
            vec![(
                Duration::new(NoteValue::Eighth, 0, None),
                Lifetime::Explicit(Entity::new(2)),
            )],
        );
        assert_eq!(
            bar.rhythm(),
            &vec![
                (
                    Duration::new(NoteValue::Eighth, 0, None),
                    Lifetime::Explicit(Entity::new(1))
                ),
                (
                    Duration::new(NoteValue::Eighth, 0, None),
                    Lifetime::AutomaticRest
                ),
                (
                    Duration::new(NoteValue::Eighth, 0, None),
                    Lifetime::AutomaticRest
                ),
                (
                    Duration::new(NoteValue::Quarter, 1, None),
                    Lifetime::AutomaticRest
                ),
                (
                    Duration::new(NoteValue::Quarter, 1, None),
                    Lifetime::AutomaticRest
                ),
                (
                    Duration::new(NoteValue::Quarter, 0, None),
                    Lifetime::AutomaticRest
                ),
                (
                    Duration::new(NoteValue::Eighth, 0, None),
                    Lifetime::Explicit(Entity::new(2)),
                ),
            ],
        );
    }

    #[test]
    fn split_note_duple() {
        let two_four = Bar::new(Metre::new(2, 4));
        let four_four = Bar::new(Metre::new(4, 4));

        assert_eq!(
            four_four.split_note(
                Rational::new(3, 8),
                Duration::new(NoteValue::Quarter, 0, None)
            ),
            vec![
                Duration::new(NoteValue::Eighth, 0, None),
                Duration::new(NoteValue::Eighth, 0, None),
            ]
        );

        assert_eq!(
            four_four.split_note(Rational::zero(), Duration::new(NoteValue::Half, 1, None)),
            vec![Duration::new(NoteValue::Half, 1, None),]
        );

        assert_eq!(
            two_four.split_note(
                Rational::new(1, 16),
                Duration::new(NoteValue::Quarter, 1, None)
            ),
            vec![
                Duration::new(NoteValue::Eighth, 1, None),
                Duration::new(NoteValue::Eighth, 1, None),
            ]
        );

        assert_eq!(
            four_four.split_note(Rational::new(1, 8), Duration::new(NoteValue::Half, 0, None)),
            vec![
                Duration::new(NoteValue::Quarter, 1, None),
                Duration::new(NoteValue::Eighth, 0, None),
            ]
        );

        assert_eq!(
            four_four.split_note(Rational::new(3, 8), Duration::new(NoteValue::Half, 0, None)),
            vec![
                Duration::new(NoteValue::Eighth, 0, None),
                Duration::new(NoteValue::Quarter, 1, None),
            ]
        );
    }

    #[test]
    fn split_note_triple() {
        let two_four = Bar::new(Metre::new(2, 4));
        let nine_eight = Bar::new(Metre::new(9, 8));

        assert_eq!(
            two_four.split_note(Rational::new(1, 8), Duration::new(NoteValue::Half, 0, None)),
            vec![
                Duration::new(NoteValue::Eighth, 0, None),
                Duration::new(NoteValue::Quarter, 1, None),
            ]
        );

        assert_eq!(
            nine_eight.split_note(Rational::zero(), Duration::new(NoteValue::Half, 1, None)),
            vec![Duration::new(NoteValue::Half, 1, None),]
        );

        assert_eq!(
            nine_eight.split_note(Rational::zero(), Duration::new(NoteValue::Whole, 0, None)),
            vec![
                Duration::new(NoteValue::Half, 1, None),
                Duration::new(NoteValue::Quarter, 0, None),
            ]
        );
    }

    #[test]
    fn split_note_existing_space() {
        let mut four_four = Bar::new(Metre::new(4, 4));
        four_four.splice(
            Rational::new(1, 8),
            vec![(
                Duration::new(NoteValue::Quarter, 1, None),
                Lifetime::Explicit(Entity::new(1)),
            )],
        );

        assert_eq!(
            four_four.split_note(Rational::new(0, 1), Duration::new(NoteValue::Half, 0, None)),
            vec![Duration::new(NoteValue::Eighth, 0, None),]
        );
    }

    #[test]
    fn split_syncopation_exceptions() {
        // Behind Bars p.166 and p.171
        let two_four = Bar::new(Metre::new(2, 4));
        let three_four = Bar::new(Metre::new(3, 4));
        let four_four = Bar::new(Metre::new(4, 4));

        assert_eq!(
            four_four.split_note(
                Rational::new(1, 8),
                Duration::new(NoteValue::Quarter, 0, None)
            ),
            vec![Duration::new(NoteValue::Quarter, 0, None),]
        );

        assert_eq!(
            two_four.split_note(
                Rational::new(1, 8),
                Duration::new(NoteValue::Quarter, 0, None)
            ),
            vec![Duration::new(NoteValue::Quarter, 0, None),]
        );

        assert_eq!(
            four_four.split_note(Rational::new(1, 4), Duration::new(NoteValue::Half, 1, None)),
            vec![Duration::new(NoteValue::Half, 1, None),]
        );

        assert_eq!(
            two_four.split_note(
                Rational::new(1, 8),
                Duration::new(NoteValue::Quarter, 1, None)
            ),
            vec![Duration::new(NoteValue::Quarter, 1, None),]
        );

        assert_eq!(
            three_four.split_note(
                Rational::new(1, 8),
                Duration::new(NoteValue::Quarter, 0, None)
            ),
            vec![Duration::new(NoteValue::Quarter, 0, None),]
        );

        assert_eq!(
            three_four.split_note(
                Rational::new(3, 8),
                Duration::new(NoteValue::Quarter, 0, None)
            ),
            vec![
                Duration::new(NoteValue::Eighth, 0, None),
                Duration::new(NoteValue::Eighth, 0, None)
            ]
        );

        assert_eq!(
            three_four.split_note(
                Rational::new(1, 16),
                Duration::new(NoteValue::Eighth, 0, None)
            ),
            vec![Duration::new(NoteValue::Eighth, 0, None),]
        );

        assert_eq!(
            four_four.split_note(Rational::new(1, 4), Duration::new(NoteValue::Half, 0, None)),
            vec![Duration::new(NoteValue::Half, 0, None),]
        );

        assert_eq!(
            four_four.split_note(Rational::new(1, 4), Duration::new(NoteValue::Half, 1, None)),
            vec![Duration::new(NoteValue::Half, 1, None),]
        );
    }

    #[test]
    fn split_syncopation() {
        // Behind Bars p.166 and p.171
        let three_four = Bar::new(Metre::new(3, 4));

        assert_eq!(
            three_four.split_note(
                Rational::new(1, 16),
                Duration::new(NoteValue::Quarter, 0, None)
            ),
            vec![
                Duration::new(NoteValue::Eighth, 1, None),
                Duration::new(NoteValue::Sixteenth, 0, None)
            ]
        );

        // TODO
        // let two_four = Bar::new(Metre::new(2, 4));
        // assert_eq!(
        //     two_four.split_note(
        //         Rational::new(1, 32),
        //         Duration::new(NoteValue::Eighth, 1, None)
        //     ),
        //     vec![
        //         Duration::new(NoteValue::Sixteenth, 1, None),
        //         Duration::new(NoteValue::Sixteenth, 1, None)
        //     ]
        // );
    }

    #[test]
    fn split_long_simple() {
        let two_four = Bar::new(Metre::new(2, 4));
        let four_eight = Bar::new(Metre::new(4, 8));
        let four_four = Bar::new(Metre::new(4, 4));

        assert_eq!(
            two_four.split_note(Rational::zero(), Duration::new(NoteValue::Quarter, 3, None)),
            vec![Duration::new(NoteValue::Quarter, 3, None),]
        );

        assert_eq!(
            four_eight.split_note(Rational::zero(), Duration::new(NoteValue::Quarter, 3, None)),
            vec![Duration::new(NoteValue::Quarter, 3, None),]
        );

        assert_eq!(
            four_four.split_note(Rational::zero(), Duration::new(NoteValue::Half, 4, None)),
            vec![Duration::new(NoteValue::Half, 4, None),]
        );

        assert_eq!(
            two_four.split_note(
                Rational::new(1, 16),
                Duration::new(NoteValue::Quarter, 2, None)
            ),
            vec![
                Duration::new(NoteValue::Eighth, 1, None),
                Duration::new(NoteValue::Quarter, 0, None)
            ]
        );

        assert_eq!(
            four_eight.split_note(
                Rational::new(1, 16),
                Duration::new(NoteValue::Quarter, 2, None)
            ),
            vec![
                Duration::new(NoteValue::Eighth, 1, None),
                Duration::new(NoteValue::Quarter, 0, None)
            ]
        );

        assert_eq!(
            four_four.split_note(
                Rational::new(1, 16),
                Duration::new(NoteValue::Half, 3, None)
            ),
            vec![
                Duration::new(NoteValue::Quarter, 2, None),
                Duration::new(NoteValue::Half, 0, None)
            ]
        );
    }

    #[test]
    fn beaming_basic_two_four() {
        let two_four = Bar::new(Metre::new(2, 4));
        assert_eq!(
            two_four.beaming(
                Rational::zero(),
                vec![
                    Duration::new(NoteValue::Eighth, 0, None),
                    Duration::new(NoteValue::Eighth, 0, None),
                    Duration::new(NoteValue::Eighth, 0, None),
                    Duration::new(NoteValue::Eighth, 0, None),
                ]
            ),
            vec![
                Some(RhythmicBeaming {
                    entering: 0,
                    leaving: 1,
                }),
                Some(RhythmicBeaming {
                    entering: 1,
                    leaving: 1,
                }),
                Some(RhythmicBeaming {
                    entering: 1,
                    leaving: 1,
                }),
                Some(RhythmicBeaming {
                    entering: 1,
                    leaving: 0,
                }),
            ]
        );

        assert_eq!(
            two_four.beaming(
                Rational::new(1, 8),
                vec![
                    Duration::new(NoteValue::Eighth, 0, None),
                    Duration::new(NoteValue::Eighth, 0, None),
                    Duration::new(NoteValue::Eighth, 0, None),
                ]
            ),
            vec![
                Some(RhythmicBeaming {
                    entering: 0,
                    leaving: 1,
                }),
                Some(RhythmicBeaming {
                    entering: 1,
                    leaving: 1,
                }),
                Some(RhythmicBeaming {
                    entering: 1,
                    leaving: 0,
                }),
            ]
        );
    }

    #[test]
    fn beaming_basic_three_four() {
        let three_four = Bar::new(Metre::new(3, 4));
        assert_eq!(
            three_four.beaming(
                Rational::zero(),
                vec![
                    Duration::new(NoteValue::Eighth, 0, None),
                    Duration::new(NoteValue::Eighth, 0, None),
                    Duration::new(NoteValue::Eighth, 0, None),
                    Duration::new(NoteValue::Eighth, 0, None),
                    Duration::new(NoteValue::Eighth, 0, None),
                ]
            ),
            vec![
                Some(RhythmicBeaming {
                    entering: 0,
                    leaving: 1,
                }),
                Some(RhythmicBeaming {
                    entering: 1,
                    leaving: 1,
                }),
                Some(RhythmicBeaming {
                    entering: 1,
                    leaving: 1,
                }),
                Some(RhythmicBeaming {
                    entering: 1,
                    leaving: 1,
                }),
                Some(RhythmicBeaming {
                    entering: 1,
                    leaving: 0,
                }),
            ]
        );
    }

    #[test]
    fn beaming_three_four_is_not_six_eight() {
        let three_four = Bar::new(Metre::new(3, 4));
        assert_eq!(
            three_four.beaming(
                Rational::new(3, 8),
                vec![
                    Duration::new(NoteValue::Eighth, 0, None),
                    Duration::new(NoteValue::Eighth, 0, None),
                    Duration::new(NoteValue::Eighth, 0, None),
                ]
            ),
            vec![
                None,
                Some(RhythmicBeaming {
                    entering: 0,
                    leaving: 1,
                }),
                Some(RhythmicBeaming {
                    entering: 1,
                    leaving: 0,
                }),
            ]
        );

        assert_eq!(
            three_four.beaming(
                Rational::new(1, 4),
                vec![
                    Duration::new(NoteValue::Eighth, 0, None),
                    Duration::new(NoteValue::Eighth, 0, None),
                    Duration::new(NoteValue::Quarter, 0, None),
                ]
            ),
            vec![
                Some(RhythmicBeaming {
                    entering: 0,
                    leaving: 1,
                }),
                Some(RhythmicBeaming {
                    entering: 1,
                    leaving: 0,
                }),
                None,
            ]
        );
    }

    #[test]
    fn beaming_basic_four_eight() {
        let four_eight = Bar::new(Metre::new(4, 8));
        assert_eq!(
            four_eight.beaming(
                Rational::zero(),
                vec![
                    Duration::new(NoteValue::Sixteenth, 0, None),
                    Duration::new(NoteValue::Sixteenth, 0, None),
                    Duration::new(NoteValue::Sixteenth, 0, None),
                    Duration::new(NoteValue::Sixteenth, 0, None),
                    Duration::new(NoteValue::Sixteenth, 0, None),
                    Duration::new(NoteValue::Sixteenth, 0, None),
                    Duration::new(NoteValue::Sixteenth, 0, None),
                    Duration::new(NoteValue::Sixteenth, 0, None),
                ]
            ),
            vec![
                Some(RhythmicBeaming {
                    entering: 0,
                    leaving: 2,
                }),
                Some(RhythmicBeaming {
                    entering: 2,
                    leaving: 2,
                }),
                Some(RhythmicBeaming {
                    entering: 2,
                    leaving: 2,
                }),
                Some(RhythmicBeaming {
                    entering: 2,
                    leaving: 0,
                }),
                Some(RhythmicBeaming {
                    entering: 0,
                    leaving: 2,
                }),
                Some(RhythmicBeaming {
                    entering: 2,
                    leaving: 2,
                }),
                Some(RhythmicBeaming {
                    entering: 2,
                    leaving: 2,
                }),
                Some(RhythmicBeaming {
                    entering: 2,
                    leaving: 0,
                }),
            ]
        );
    }

    #[test]
    fn beaming_basic_four_four() {
        let four_four = Bar::new(Metre::new(4, 4));
        assert_eq!(
            four_four.beaming(
                Rational::zero(),
                vec![
                    Duration::new(NoteValue::Eighth, 0, None),
                    Duration::new(NoteValue::Eighth, 0, None),
                    Duration::new(NoteValue::Eighth, 0, None),
                    Duration::new(NoteValue::Eighth, 0, None),
                    Duration::new(NoteValue::Eighth, 0, None),
                    Duration::new(NoteValue::Eighth, 0, None),
                    Duration::new(NoteValue::Eighth, 0, None),
                    Duration::new(NoteValue::Eighth, 0, None),
                ]
            ),
            vec![
                Some(RhythmicBeaming {
                    entering: 0,
                    leaving: 1,
                }),
                Some(RhythmicBeaming {
                    entering: 1,
                    leaving: 1,
                }),
                Some(RhythmicBeaming {
                    entering: 1,
                    leaving: 1,
                }),
                Some(RhythmicBeaming {
                    entering: 1,
                    leaving: 0,
                }),
                Some(RhythmicBeaming {
                    entering: 0,
                    leaving: 1,
                }),
                Some(RhythmicBeaming {
                    entering: 1,
                    leaving: 1,
                }),
                Some(RhythmicBeaming {
                    entering: 1,
                    leaving: 1,
                }),
                Some(RhythmicBeaming {
                    entering: 1,
                    leaving: 0,
                }),
            ]
        );
        assert_eq!(
            four_four.beaming(
                Rational::new(1, 8),
                vec![
                    Duration::new(NoteValue::Eighth, 0, None),
                    Duration::new(NoteValue::Eighth, 0, None),
                    Duration::new(NoteValue::Eighth, 0, None),
                    Duration::new(NoteValue::Eighth, 0, None),
                    Duration::new(NoteValue::Eighth, 0, None),
                    Duration::new(NoteValue::Eighth, 0, None),
                ]
            ),
            vec![
                Some(RhythmicBeaming {
                    entering: 0,
                    leaving: 1,
                }),
                Some(RhythmicBeaming {
                    entering: 1,
                    leaving: 1,
                }),
                Some(RhythmicBeaming {
                    entering: 1,
                    leaving: 0,
                }),
                Some(RhythmicBeaming {
                    entering: 0,
                    leaving: 1,
                }),
                Some(RhythmicBeaming {
                    entering: 1,
                    leaving: 1,
                }),
                Some(RhythmicBeaming {
                    entering: 1,
                    leaving: 0,
                }),
            ]
        );
        assert_eq!(
            four_four.beaming(
                Rational::new(0, 1),
                vec![
                    Duration::new(NoteValue::Eighth, 0, None),
                    Duration::new(NoteValue::Eighth, 0, None),
                ]
            ),
            vec![
                Some(RhythmicBeaming {
                    entering: 0,
                    leaving: 1,
                }),
                Some(RhythmicBeaming {
                    entering: 1,
                    leaving: 0,
                }),
            ]
        );
        assert_eq!(
            four_four.beaming(
                Rational::new(1, 4),
                vec![
                    Duration::new(NoteValue::Eighth, 0, None),
                    Duration::new(NoteValue::Eighth, 0, None),
                    Duration::new(NoteValue::Eighth, 0, None),
                    Duration::new(NoteValue::Eighth, 0, None),
                ]
            ),
            vec![
                Some(RhythmicBeaming {
                    entering: 0,
                    leaving: 1,
                }),
                Some(RhythmicBeaming {
                    entering: 1,
                    leaving: 0,
                }),
                Some(RhythmicBeaming {
                    entering: 0,
                    leaving: 1,
                }),
                Some(RhythmicBeaming {
                    entering: 1,
                    leaving: 0,
                }),
            ]
        );
    }
}
