use crate::duration::{Duration, RationalToF64};
use num_rational::Rational;

#[derive(Debug, Default, Copy, Clone, PartialEq)]
pub struct RelativeRhythmicSpacing(f64);

impl RelativeRhythmicSpacing {
    pub fn new(shortest: Rational, duration: &Duration) -> RelativeRhythmicSpacing {
        let duration = duration.duration();
        RelativeRhythmicSpacing(1.0 + (duration.to_f64() / shortest.to_f64()).log2())
    }

    pub fn relative(&self) -> f64 {
        self.0
    }
}

#[cfg(test)]
mod spacing_tests {
    use super::*;

    #[test]
    fn basic_rythmic_spacing() {
        let sixteenth = Rational::new(1, 16);
        assert_eq!(
            RelativeRhythmicSpacing::new(sixteenth, &Duration::exact(Rational::new(1, 2), None)),
            RelativeRhythmicSpacing(4.0)
        );
        assert_eq!(
            RelativeRhythmicSpacing::new(sixteenth, &Duration::exact(Rational::new(1, 4), None)),
            RelativeRhythmicSpacing(3.0)
        );
        assert_eq!(
            RelativeRhythmicSpacing::new(sixteenth, &Duration::exact(Rational::new(1, 8), None)),
            RelativeRhythmicSpacing(2.0)
        );
        assert_eq!(
            RelativeRhythmicSpacing::new(sixteenth, &Duration::exact(Rational::new(1, 16), None)),
            RelativeRhythmicSpacing(1.0)
        );

        assert_eq!(
            RelativeRhythmicSpacing::new(
                Rational::new(1, 32),
                &Duration::exact(Rational::new(1, 16), None)
            ),
            RelativeRhythmicSpacing(2.0)
        );
    }
}
