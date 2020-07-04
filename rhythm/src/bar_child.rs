use crate::duration::Duration;
use crate::lifetime::Lifetime;
use num_rational::Rational;
use specs::Entity;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct BarChild {
    pub duration: Duration,
    pub start: Rational,
    pub lifetime: Lifetime,
    pub stencil: Entity,
}

