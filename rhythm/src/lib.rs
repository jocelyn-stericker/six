mod bar;
mod duration;
mod metre;
mod spacing;

use num_rational::Rational;

#[derive(Debug, Clone, Copy)]
pub struct Start {
    pub bar: usize,
    pub beat: Rational,
    pub natural_beat: Rational,
}

impl Default for Start {
    fn default() -> Start {
        Start {
            bar: 0,
            beat: Rational::new(0, 1),
            natural_beat: Rational::new(0, 1),
        }
    }
}

pub use bar::Bar;
pub use duration::{Duration, NoteValue};
pub use metre::{Metre, MetreSegment, Subdivision, Superdivision};
pub use spacing::RelativeRhythmicSpacing;
