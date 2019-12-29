use num_integer::Integer;
use num_rational::Rational;
use num_traits::Zero;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
/// Whether there are an even or odd number of time signature denominator notes in a bar segment.
///
/// A bar segment is a part of a bar, starting on a stress, and ending before the next stress.
///
/// In classical music, segments are organized to ensure each segment has only 2, 3, or 4 equal
/// subdivisions.
///
/// See https://en.wikipedia.org/wiki/Metre_(music)#Metres_classified_by_the_subdivisions_of_a_beat
pub enum Subdivision {
    /// There are an even number of equal subdivisions.
    ///
    /// Simple time is implied when there is a single subdivision.
    Simple,

    /// There are an odd number of equal subdivisions.
    Compound,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
/// There can be an even or odd number of segments in a bar, which determines a segment's role.
///
/// A bar segment is a part of a bar, starting on a stress, and ending before the next stress.
///
/// See
/// https://en.wikipedia.org/wiki/Metre_(music)#Metres_classified_by_the_number_of_beats_per_measure
pub enum Superdivision {
    /// An even number of such parts from a group.
    Duple,
    /// An odd (usually three) number of such parts form a group.
    Triple,
    /// The emphasis on the 6th eighth note of 12/8.
    Quadruple,
}

#[derive(Debug, Clone, Copy)]
/// A part of bar, starting on a stress and ending before the next one.
pub struct MetreSegment {
    duration: Rational,
    subdivisions: u8,
    superdivision: Superdivision,
}

impl MetreSegment {
    /// The number of whole notes in this part of the bar.
    pub fn duration(&self) -> Rational {
        self.duration
    }

    /// The number of base notes, as defined by the time signature denominator, this part is
    /// divided into.
    pub fn subdivisions(&self) -> u8 {
        self.subdivisions
    }

    /// The number of whole notes each base notes, as defined by the time signature denominator,
    /// takes.
    pub fn subdivision_duration(&self) -> Rational {
        self.duration / Rational::from_integer(self.subdivisions.into())
    }

    /// The classification of how this segment is divided.
    pub fn subdivision(&self) -> Subdivision {
        if self.subdivisions % 2 == 0 || self.subdivisions == 1 {
            Subdivision::Simple
        } else {
            Subdivision::Compound
        }
    }

    /// The role this segment plays in the bar.
    pub fn superdivision(&self) -> Superdivision {
        self.superdivision
    }
}

#[derive(Debug, Clone)]
pub struct Metre(Vec<MetreSegment>);

/// The way beats are organized in a bar of music.
impl Metre {
    pub fn new(num: u8, den: u8) -> Metre {
        fn duple(duration: Rational, subdivisions: u8) -> Vec<MetreSegment> {
            let seg = MetreSegment {
                duration,
                subdivisions,
                superdivision: Superdivision::Duple,
            };
            vec![seg, seg]
        }
        fn triple(duration: Rational, subdivisions: u8) -> Vec<MetreSegment> {
            let seg = MetreSegment {
                duration,
                subdivisions,
                superdivision: Superdivision::Triple,
            };
            vec![seg, seg, seg]
        }
        fn quad(duration: Rational, subdivisions: u8) -> Vec<MetreSegment> {
            let seg = MetreSegment {
                duration,
                subdivisions,
                superdivision: Superdivision::Duple,
            };
            vec![
                seg,
                seg,
                MetreSegment {
                    duration,
                    subdivisions,
                    superdivision: Superdivision::Quadruple,
                },
                seg,
            ]
        }

        Metre(match (num, den) {
            (4, 4) => duple(Rational::new(1, 2), 2),
            (2, 2) => duple(Rational::new(1, 2), 1),
            (4, 8) => duple(Rational::new(1, 4), 2),
            (2, 4) => duple(Rational::new(1, 4), 1),
            (6, 16) => duple(Rational::new(3, 16), 3),
            (6, 8) => duple(Rational::new(3, 8), 3),
            (6, 4) => duple(Rational::new(3, 4), 3),
            (12, 8) => quad(Rational::new(3, 8), 3),
            (3, 4) => triple(Rational::new(1, 4), 1),
            (3, 8) => triple(Rational::new(1, 8), 1),
            (9, 8) => triple(Rational::new(3, 8), 3),
            (num, den)
                if den == 1 || den == 2 || den == 4 || den == 8 || den == 16 || den == 32 =>
            {
                // Segments should have a printable duration.  There is no accepted convention for
                // how to organize these segments, so take a guess.
                let mut segments = Vec::new();
                let mut t = num;
                while t > 0 {
                    let subdivisions = if t == 4 {
                        4
                    } else if t >= 3 {
                        3
                    } else if t >= 2 {
                        2
                    } else {
                        1
                    };
                    assert!(t >= subdivisions);
                    segments.push(MetreSegment {
                        duration: Rational::new(subdivisions.into(), den.into()),
                        subdivisions,
                        // TODO: guess this too?
                        superdivision: Superdivision::Duple,
                    });
                    t -= subdivisions;
                }

                segments
            }
            _ => panic!("Invalid denominator."),
        })
    }

    /// The duration of the bar, in whole notes.
    ///
    /// ```
    /// use rhythm::*;
    /// use num_rational::Rational;
    ///
    /// assert_eq!(Metre::new(4, 4).duration(), 1.into());
    ///
    /// for num in 1..12 {
    ///   for den in &[1,2,4,8,16,32] {
    ///     assert_eq!(Metre::new(num, *den).duration(), Rational::new(num as isize, *den as isize));
    ///   }
    /// }
    /// ```
    pub fn duration(&self) -> Rational {
        self.0
            .iter()
            .fold(Rational::zero(), |len, division| len + division.duration)
    }

    /// The 0-indexed times (in whole notes) of divisions in this bar, and the start of the next
    /// bar.
    ///
    /// ```
    /// use rhythm::*;
    /// use num_traits::Zero;
    /// use num_rational::Rational;
    ///
    /// assert_eq!(Metre::new(4, 4).division_starts(), vec![Rational::zero(), Rational::new(1, 2), Rational::new(2, 2)] );
    /// assert_eq!(Metre::new(3, 4).division_starts(), vec![Rational::zero(), Rational::new(1, 4), Rational::new(2, 4), Rational::new(3, 4)] );
    /// assert_eq!(Metre::new(5, 8).division_starts(), vec![Rational::zero(), Rational::new(3, 8), Rational::new(5, 8)] );
    /// ```
    pub fn division_starts(&self) -> Vec<Rational> {
        let mut divisions = vec![];
        let mut start = Rational::zero();
        for division in &self.0 {
            divisions.push(start);
            start += division.duration;
        }
        // Include the start of the next bar.
        divisions.push(start);

        divisions
    }

    /// Given a time, the division that it is in.
    pub fn on_division(&self, t: Rational) -> Option<&MetreSegment> {
        let mut t2 = Rational::zero();
        for division in &self.0 {
            if t == t2 {
                return Some(division);
            }
            t2 += division.duration();
        }

        None
    }

    /// Given a time, the division that it is in.
    pub fn division(&self, t: Rational) -> (Rational, MetreSegment) {
        let mut t2 = Rational::zero();
        for division in &self.0 {
            let next = t2 + division.duration();
            if t < next {
                return (t2, *division);
            }
            t2 = next;
        }

        (t2, *self.0.last().unwrap())
    }

    /// Given a time, the start of the division that follows, or the end of the bar.
    pub fn next_division(&self, t: Rational) -> Rational {
        let starts = self.division_starts();
        for division in &starts {
            if t < *division {
                return *division;
            }
        }

        *starts.last().unwrap()
    }

    /// The 0-indexed times (in whole notes) of beats in this bar, and the first beat of the next
    /// bar.
    pub fn beats(&self) -> Vec<Rational> {
        let mut beats = vec![];
        let mut start = Rational::zero();
        for division in &self.0 {
            let increment =
                division.duration / Rational::from_integer(division.subdivisions.into());

            for _ in 0..division.subdivisions {
                beats.push(start);
                start += increment;
            }
        }
        beats.push(start);

        beats
    }

    /// The duration (in whole notes) of a beat at the specified time in the bar.
    pub fn beat_duration(&self, time: Rational) -> Rational {
        let mut start = Rational::zero();
        for division in &self.0 {
            if time < start + division.duration {
                return division.subdivision_duration();
            }
            start += division.duration;
        }

        Rational::zero()
    }

    pub fn lcm(&self) -> isize {
        self.0.iter().fold(1isize, |acc, sub| {
            acc.lcm((sub.duration / Rational::from_integer(sub.subdivisions.into())).denom())
        })
    }
}
