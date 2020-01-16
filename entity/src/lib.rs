//! A minimal Entity Component System (ECS) inspired by specs.
//!
//! Create a single `EntitiesRes` and use it to allocate Entity objects. You can pass it into
//! systems.
//!
//! Component storage is defined as `HashMap<Entity, C>` where C is any type.
//!
//! World management and running systems is up to the application.
//!
//! There is a `Join` trait which can be used to join HashSets, HashMap refs, and HashMap mut refs,
//! based on SPECS `Join` trait. This is used to filter entities to those which are in a set or
//! have all required components.
#![allow(clippy::implicit_hasher)]

use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicUsize, Ordering};

#[derive(Clone, Copy, Debug, Hash, Eq, Ord, PartialEq, PartialOrd)]
pub struct Entity(usize);

impl Entity {
    pub fn new(id: usize) -> Entity {
        Entity(id)
    }
    pub fn id(self) -> usize {
        self.0
    }
}

#[derive(Default, Debug)]
pub struct EntitiesRes(AtomicUsize);

/// Use this to create new Entities.
impl EntitiesRes {
    pub fn create(&self) -> Entity {
        let mut prev = self.0.load(Ordering::Relaxed);
        while prev != std::usize::MAX {
            match self
                .0
                .compare_exchange_weak(prev, prev + 1, Ordering::Relaxed, Ordering::Relaxed)
            {
                Ok(x) => return Entity(x),
                Err(next_prev) => prev = next_prev,
            }
        }

        // When we get here, we'll need to make a better entity struct.
        panic!("Out of numbers.");
    }
}

pub trait Join {
    type Value;

    fn join(self) -> HashMap<Entity, Self::Value>;
}

impl<'a, T> Join for &'a HashMap<Entity, T> {
    type Value = &'a T;

    fn join(self) -> HashMap<Entity, &'a T> {
        self.iter().map(|(a, b)| (*a, b)).collect()
    }
}

impl<'a, T> Join for &'a mut HashMap<Entity, T> {
    type Value = &'a mut T;

    fn join(self) -> HashMap<Entity, &'a mut T> {
        self.iter_mut().map(|(a, b)| (*a, b)).collect()
    }
}

impl Join for &HashSet<Entity> {
    type Value = ();

    fn join(self) -> HashMap<Entity, ()> {
        self.iter().map(|a| (*a, ())).collect()
    }
}

// Trivial.
impl<A: Join<Value = AT>, AT> Join for (A,) {
    type Value = (AT,);

    fn join(self) -> HashMap<Entity, (AT,)> {
        self.0.join().into_iter().map(|(k, v)| (k, (v,))).collect()
    }
}

impl<A: Join<Value = AT>, B: Join<Value = BT>, AT, BT> Join for (A, B) {
    type Value = (AT, BT);

    fn join(self) -> HashMap<Entity, (AT, BT)> {
        let mut a = self.0.join();
        let mut b = self.1.join();
        let keys: Vec<Entity> = a
            .keys()
            .copied()
            .collect::<HashSet<_>>()
            .intersection(&b.keys().copied().collect::<HashSet<_>>())
            .copied()
            .collect();

        let mut pairs: HashMap<Entity, (AT, BT)> = HashMap::default();
        for key in keys {
            pairs.insert(key, (a.remove(&key).unwrap(), b.remove(&key).unwrap()));
        }

        pairs
    }
}

impl<'a, A: Join<Value = AT>, B: Join<Value = BT>, C: Join<Value = CT>, AT, BT, CT> Join
    for (A, B, C)
{
    type Value = (AT, BT, CT);

    fn join(self) -> HashMap<Entity, (AT, BT, CT)> {
        let mut a = self.0.join();
        let mut b = self.1.join();
        let mut c = self.2.join();

        let keys: Vec<Entity> = a
            .keys()
            .copied()
            .collect::<HashSet<_>>()
            .intersection(&b.keys().copied().collect::<HashSet<_>>())
            .copied()
            .collect::<HashSet<_>>()
            .intersection(&c.keys().copied().collect::<HashSet<_>>())
            .copied()
            .collect();

        let mut pairs: HashMap<Entity, (AT, BT, CT)> = HashMap::default();
        for key in keys {
            pairs.insert(
                key,
                (
                    a.remove(&key).unwrap(),
                    b.remove(&key).unwrap(),
                    c.remove(&key).unwrap(),
                ),
            );
        }

        pairs
    }
}

impl<
        'a,
        A: Join<Value = AT>,
        B: Join<Value = BT>,
        C: Join<Value = CT>,
        D: Join<Value = DT>,
        AT,
        BT,
        CT,
        DT,
    > Join for (A, B, C, D)
{
    type Value = (AT, BT, CT, DT);

    fn join(self) -> HashMap<Entity, (AT, BT, CT, DT)> {
        let mut a = self.0.join();
        let mut b = self.1.join();
        let mut c = self.2.join();
        let mut d = self.3.join();

        let keys: Vec<Entity> = a
            .keys()
            .copied()
            .collect::<HashSet<_>>()
            .intersection(&b.keys().copied().collect::<HashSet<_>>())
            .copied()
            .collect::<HashSet<_>>()
            .intersection(&c.keys().copied().collect::<HashSet<_>>())
            .copied()
            .collect::<HashSet<_>>()
            .intersection(&d.keys().copied().collect::<HashSet<_>>())
            .copied()
            .collect();

        let mut pairs: HashMap<Entity, (AT, BT, CT, DT)> = HashMap::default();
        for key in keys {
            pairs.insert(
                key,
                (
                    a.remove(&key).unwrap(),
                    b.remove(&key).unwrap(),
                    c.remove(&key).unwrap(),
                    d.remove(&key).unwrap(),
                ),
            );
        }

        pairs
    }
}

// TODO(joshuan): Continue this up to 16 or so. Allow structs, maybe?

#[cfg(test)]
mod tests {
    use super::Join;
    use std::collections::{HashMap, HashSet};

    #[test]
    fn it_counts() {
        let res = super::EntitiesRes::default();
        assert_eq!(res.create(), super::Entity(0));
        assert_eq!(res.create(), super::Entity(1));
        assert_eq!(res.create(), super::Entity(2));
    }

    #[test]
    fn it_joins() {
        let mut w: HashSet<super::Entity> = Default::default();
        w.insert(super::Entity(2));

        let mut x: HashMap<super::Entity, i64> = Default::default();
        x.insert(super::Entity(1), 1);
        x.insert(super::Entity(2), 2);
        x.insert(super::Entity(4), 4);

        let mut y: HashMap<super::Entity, f32> = Default::default();
        y.insert(super::Entity(1), 1.0);
        y.insert(super::Entity(2), 2.0);
        y.insert(super::Entity(3), 3.0);

        let _: HashMap<super::Entity, (&i64,)> = (&x,).join();
        let _: HashMap<super::Entity, &mut f32> = (&mut y).join();

        let xy: HashMap<super::Entity, (&i64, &f32)> = (&x, &y).join();
        assert_eq!(xy.get(&super::Entity(1)), Some(&(&1, &1.0)));
        assert_eq!(xy.get(&super::Entity(2)), Some(&(&2, &2.0)));
        assert_eq!(xy.get(&super::Entity(3)), None);
        assert_eq!(xy.get(&super::Entity(4)), None);

        let mut yx: HashMap<super::Entity, (&mut f32, &i64)> = (&mut y, &x).join();
        assert_eq!(yx.get(&super::Entity(1)), Some(&(&mut 1.0, &1)));
        assert_eq!(yx.get(&super::Entity(2)), Some(&(&mut 2.0, &2)));
        assert_eq!(yx.get(&super::Entity(3)), None);
        assert_eq!(yx.get(&super::Entity(4)), None);

        *yx.get_mut(&super::Entity(2)).unwrap().0 = 23.0;
        assert_eq!(y.get(&super::Entity(2)), Some(&23.0));

        let wy: HashMap<super::Entity, (&f32, ())> = (&y, &w).join();
        // w does not have 1.
        assert_eq!(wy.get(&super::Entity(1)), None);
        // We modified it.
        assert_eq!(wy.get(&super::Entity(2)), Some(&(&23.0, ())));
    }
}
