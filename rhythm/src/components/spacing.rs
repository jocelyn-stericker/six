use crate::duration::{Duration, RationalToF64};
use num_rational::Rational;
use specs::{Component, VecStorage};

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Spacing {
    pub t: Rational,
    pub relative: f64,
    pub start_x: f64,
    pub end_x: f64,
}

impl Component for Spacing {
    type Storage = VecStorage<Self>;
}

impl Default for Spacing {
    fn default() -> Self {
        Spacing {
            t: Rational::new(0, 1),
            relative: 1.0,
            start_x: 0.0,
            end_x: 0.0,
        }
    }
}

impl Spacing {
    pub fn new(shortest: Rational, duration: &Duration) -> Spacing {
        let duration = duration.duration();
        Spacing {
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
            Spacing::new(sixteenth, &Duration::exact(Rational::new(1, 2), None)),
            Spacing {
                t: Rational::new(0, 1),
                relative: 4.0,
                start_x: 0.0,
                end_x: 0.0
            }
        );
        assert_eq!(
            Spacing::new(sixteenth, &Duration::exact(Rational::new(1, 4), None)),
            Spacing {
                t: Rational::new(0, 1),
                relative: 3.0,
                start_x: 0.0,
                end_x: 0.0
            }
        );
        assert_eq!(
            Spacing::new(sixteenth, &Duration::exact(Rational::new(1, 8), None)),
            Spacing {
                t: Rational::new(0, 1),
                relative: 2.0,
                start_x: 0.0,
                end_x: 0.0
            }
        );
        assert_eq!(
            Spacing::new(sixteenth, &Duration::exact(Rational::new(1, 16), None)),
            Spacing {
                t: Rational::new(0, 1),
                relative: 1.0,
                start_x: 0.0,
                end_x: 0.0
            }
        );

        assert_eq!(
            Spacing::new(
                Rational::new(1, 32),
                &Duration::exact(Rational::new(1, 16), None)
            ),
            Spacing {
                t: Rational::new(0, 1),
                relative: 2.0,
                start_x: 0.0,
                end_x: 0.0
            }
        );
    }
}
