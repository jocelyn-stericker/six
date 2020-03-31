use num_rational::Rational;
use num_traits::One;

pub(crate) trait RationalToF64 {
    fn to_f64(&self) -> f64;
}

impl RationalToF64 for Rational {
    fn to_f64(&self) -> f64 {
        (*self.numer() as f64) / (*self.denom() as f64)
    }
}

#[repr(i8)]
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
/// The unmodified relative duration of a rest, note, or chord.
///
/// Determines the note head glyph, whether the note has a stem, and whether the note has a flag.
///
/// See https://en.wikipedia.org/wiki/Note_value
pub enum NoteValue {
    Maxima = 3,
    Longa = 2,
    DoubleWhole = 1,
    Whole = 0,
    Half = -1,
    Quarter = -2,
    Eighth = -3,
    Sixteenth = -4,
    ThirtySecond = -5,
    SixtyFourth = -6,
    HundredTwentyEighth = -7,
    TwoHundredFiftySixth = -8,
}

impl NoteValue {
    pub fn new(log2: isize) -> Option<NoteValue> {
        match log2 {
            3 => Some(NoteValue::Maxima),
            2 => Some(NoteValue::Longa),
            1 => Some(NoteValue::DoubleWhole),
            0 => Some(NoteValue::Whole),
            -1 => Some(NoteValue::Half),
            -2 => Some(NoteValue::Quarter),
            -3 => Some(NoteValue::Eighth),
            -4 => Some(NoteValue::Sixteenth),
            -5 => Some(NoteValue::ThirtySecond),
            -6 => Some(NoteValue::SixtyFourth),
            -7 => Some(NoteValue::HundredTwentyEighth),
            -8 => Some(NoteValue::TwoHundredFiftySixth),
            _ => None,
        }
    }

    /// The base-2 log of the duration, compared to a whole note, ignoring dots and tuplets.
    ///
    /// ```
    /// use rhythm::*;
    ///
    /// assert_eq!(NoteValue::DoubleWhole.log2(), 1);
    /// assert_eq!(NoteValue::Whole.log2(), 0);
    /// assert_eq!(NoteValue::Quarter.log2(), -2);
    /// assert_eq!(NoteValue::HundredTwentyEighth.log2(), -7);
    /// ```
    pub fn log2(self) -> i8 {
        self as i8
    }

    /// The number of whole notes in the duration, ignoring dots and tuplets.
    ///
    /// ```
    /// use rhythm::*;
    /// use num_rational::Rational;
    ///
    /// assert_eq!(NoteValue::DoubleWhole.count(), 2.into());
    /// assert_eq!(NoteValue::Whole.count(), 1.into());
    /// assert_eq!(NoteValue::Quarter.count(), Rational::new(1, 4));
    /// assert_eq!(NoteValue::HundredTwentyEighth.count(), Rational::new(1, 128));
    /// ```
    pub fn count(self) -> Rational {
        Rational::from_integer(2).pow(self.log2() as i32)
    }

    pub fn has_stem(self) -> bool {
        self <= NoteValue::Quarter
    }

    pub fn has_flag(self) -> bool {
        self <= NoteValue::Eighth
    }

    pub fn beam_count(self) -> u8 {
        (-self.log2() - 2).max(0) as u8
    }
}

/// The maximum number of dots a note or rest can have.
///
/// "Quadruple dots appear in Liszt: Piano Concerto #2 (1839, rev. 1848; Kalmus ed.), Allegro
/// deciso, mm. 327 and 331; Schumann: String Quartet no. 1, Op. 41 no. 1 (1842), III, mm. 16, 17,
/// 33, 34, etc. (contributed by Cuthbert) (in Byrd, 2012a); Verdi: Requiem (1874; Dover ed.), Rex
/// Tremandae, mm. 356 and 358; Franck: Prelude, Chorale, and Fugue (1884; Schirmer ed.), mm. 2 and
/// 4 (in B & M, 1948); Hindemith: Mathis der Maler Symphony (1934; Schott ed.), III, introduction;
/// Bartok: Music for Strings, Percussion, and Celesta (1936; Boosey & Hawkes ed.), III, m. 80.
/// In every case, the dots are on a half note."
///
/// https://web.archive.org/web/20141031082831/http://www.informatics.indiana.edu/donbyrd/CMNExtremesBody.htm
const MAX_DOTS: u8 = 4;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
/// The duration of a note, rest, or chord.
///
/// This contains whether the duration is a tuplet, because some the duration alone is insuffient
/// to differentiate some tuplets from non-tuplets. For example, 4:3 ("2") quarter note duplet is
/// the same as a dotted eighth note. We do want to differentiate the two.
///
/// Every Duration can be displayed as a notehead or rest between a 256th and maxima note,
/// no more than `MAX_DOTS` dots, and a tuplet modifier.
///
/// Cannot be changed after creation.
pub struct Duration {
    display: Rational,
    tuplet: Rational,

    whole_rest: bool,
}

impl Duration {
    /// Create the duration of a note, chord, or rest in a bar, given a duration base, the number
    /// of dots, and what kind of tuplet it is in, if applicable.
    pub fn new(base: NoteValue, dots: u8, tuplet: Option<Rational>) -> Duration {
        if dots > MAX_DOTS {
            panic!("Too many dots");
        }

        let mut base = Rational::from_integer(2).pow(base as i8 as i32);
        let mut display = base;
        for _ in 0..dots {
            base /= 2;
            display += base;
        }

        Duration {
            display,
            tuplet: tuplet.unwrap_or_else(Rational::one),
            whole_rest: false,
        }
    }

    /// Create a duration with a displayed duration, in whole notes, and a tuplet.
    pub fn exact(display: Rational, tuplet: Option<Rational>) -> Duration {
        Duration {
            display,
            tuplet: tuplet.unwrap_or_else(Rational::one),
            whole_rest: false,
        }
    }

    /// Create a whole rest for a specified bar duration.
    pub fn new_whole_rest(display: Rational) -> Duration {
        Duration {
            display,
            tuplet: Rational::one(),
            whole_rest: true,
        }
    }

    /// Return whether this is a valid duration for a single note.
    ///
    /// A valid duration has a base type between a 1/256th note and a maxima note and at most 3
    /// dots.
    pub fn printable(&self) -> bool {
        self.whole_rest || self.duration_display_base().is_some() && self.display_dots().is_some()
    }

    /// A multiplier which converts the true duration into how it is displayed.
    ///
    /// For example, for a triplet where 3 beats are shown where 2 beats are played,
    /// this field would be 3:2.
    pub fn tuplet(&self) -> Rational {
        self.tuplet
    }

    /// The number of whole notes this event is played for.
    pub fn duration(&self) -> Rational {
        self.display / self.tuplet
    }

    /// The number of whole notes it looks like this note is played for.
    ///
    /// This is not the same as the real duration if the note is a tuplet.
    pub fn display_duration(&self) -> Rational {
        self.display
    }

    /// The kind of note this will be rendered as.
    pub fn duration_display_base(&self) -> Option<NoteValue> {
        if self.whole_rest {
            return Some(NoteValue::Whole);
        }

        NoteValue::new(self.display_duration().to_f64().log2().floor() as isize)
    }

    /// The number of dots that will be rendered.
    pub fn display_dots(&self) -> Option<usize> {
        if self.whole_rest {
            return Some(0);
        }

        let len = self.display_duration();
        let base = self.duration_display_base()?.count();
        let mut dots: usize = 0;
        let mut len_with_dots = base;
        while len_with_dots < len && dots <= MAX_DOTS.into() {
            dots += 1;
            len_with_dots += base * Rational::new(1, 2).pow(dots as i32);
        }

        if len_with_dots != len {
            return None;
        }

        Some(dots)
    }
}

#[cfg(test)]
mod event_rhythm_tests {
    use super::{Duration, NoteValue};
    use num_rational::Rational;

    #[test]
    fn half_note() {
        let er = Duration::new(NoteValue::Half, 0, None);
        assert_eq!(er.duration_display_base(), Some(NoteValue::Half));
        assert_eq!(er.display_dots(), Some(0));
        assert_eq!(er.tuplet(), 1.into());
        assert_eq!(er.duration(), Rational::new(1, 2));
        assert_eq!(er.display_duration(), Rational::new(1, 2));
    }

    #[test]
    fn dotted_double_whole_note() {
        let er = Duration::new(NoteValue::DoubleWhole, 1, None);
        assert_eq!(er.duration_display_base(), Some(NoteValue::DoubleWhole));
        assert_eq!(er.display_dots(), Some(1));
        assert_eq!(er.tuplet(), 1.into());
        assert_eq!(er.duration(), Rational::from_integer(3));
        assert_eq!(er.display_duration(), Rational::from_integer(3));
    }

    #[test]
    fn double_dotted_sixteenth_note() {
        let er = Duration::new(NoteValue::Sixteenth, 2, None);
        assert_eq!(er.duration_display_base(), Some(NoteValue::Sixteenth));
        assert_eq!(er.display_dots(), Some(2));
        assert_eq!(er.tuplet(), 1.into());
        assert_eq!(er.duration(), Rational::new(7, 64));
        assert_eq!(er.display_duration(), Rational::new(7, 64));
    }

    #[test]
    fn triplet_quarter_note() {
        let er = Duration::new(NoteValue::Quarter, 0, Some(Rational::new(3, 2)));
        assert_eq!(er.duration_display_base(), Some(NoteValue::Quarter));
        assert_eq!(er.display_dots(), Some(0));
        assert_eq!(er.tuplet(), Rational::new(3, 2));
        assert_eq!(er.duration(), Rational::new(1, 6));
        assert_eq!(er.display_duration(), Rational::new(1, 4));
    }
}
