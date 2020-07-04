use num_rational::Rational;
use specs::{storage::BTreeStorage, Component};

/// A map from time to position.
///
/// Used to position items when hovering (see root.freeze_spacing)
#[derive(Debug)]
pub struct SpaceTimeWarp(pub Vec<(Rational, f64)>);

impl Component for SpaceTimeWarp {
    type Storage = BTreeStorage<Self>;
}

impl SpaceTimeWarp {
    pub fn t_to_x(&self, t: Rational) -> f64 {
        for (i, (t1, x1)) in self.0.iter().enumerate() {
            if let Some((t2, x2)) = self.0.get(i + 1) {
                if t >= *t1 && t <= *t2 {
                    let pct = (t - t1) / (t2 - t1);
                    let pct = (*pct.numer() as f64) / (*pct.denom() as f64);

                    return x1 + pct * (x2 - x1);
                }
            }
        }

        0.0
    }
}

