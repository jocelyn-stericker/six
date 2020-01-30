use crate::duration::{Duration, RationalToF64};
use num_rational::Rational;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct RelativeRhythmicSpacing {
    pub t: Rational,
    pub relative: f64,
    pub start_x: f64,
    pub end_x: f64,
}

impl Default for RelativeRhythmicSpacing {
    fn default() -> Self {
        RelativeRhythmicSpacing {
            t: Rational::new(0, 1),
            relative: 1.0,
            start_x: 0.0,
            end_x: 0.0,
        }
    }
}

impl RelativeRhythmicSpacing {
    pub fn new(shortest: Rational, duration: &Duration) -> RelativeRhythmicSpacing {
        let duration = duration.duration();
        RelativeRhythmicSpacing {
            t: Rational::new(0, 1),
            relative: 1.0 + (duration.to_f64() / shortest.to_f64()).log2(),
            start_x: 0.0,
            end_x: 0.0,
        }
    }

    pub fn relative(self) -> f64 {
        self.relative
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
            RelativeRhythmicSpacing {
                t: Rational::new(0, 1),
                relative: 4.0,
                start_x: 0.0,
                end_x: 0.0
            }
        );
        assert_eq!(
            RelativeRhythmicSpacing::new(sixteenth, &Duration::exact(Rational::new(1, 4), None)),
            RelativeRhythmicSpacing {
                t: Rational::new(0, 1),
                relative: 3.0,
                start_x: 0.0,
                end_x: 0.0
            }
        );
        assert_eq!(
            RelativeRhythmicSpacing::new(sixteenth, &Duration::exact(Rational::new(1, 8), None)),
            RelativeRhythmicSpacing {
                t: Rational::new(0, 1),
                relative: 2.0,
                start_x: 0.0,
                end_x: 0.0
            }
        );
        assert_eq!(
            RelativeRhythmicSpacing::new(sixteenth, &Duration::exact(Rational::new(1, 16), None)),
            RelativeRhythmicSpacing {
                t: Rational::new(0, 1),
                relative: 1.0,
                start_x: 0.0,
                end_x: 0.0
            }
        );

        assert_eq!(
            RelativeRhythmicSpacing::new(
                Rational::new(1, 32),
                &Duration::exact(Rational::new(1, 16), None)
            ),
            RelativeRhythmicSpacing {
                t: Rational::new(0, 1),
                relative: 2.0,
                start_x: 0.0,
                end_x: 0.0
            }
        );
    }
}
