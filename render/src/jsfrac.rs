use num_rational::Rational;
use rhythm::{Duration, NoteValue};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
struct JsFrac;

trait RationalToVecForJs {
    fn to_js(&self) -> Vec<isize>;
}

impl RationalToVecForJs for Rational {
    fn to_js(&self) -> Vec<isize> {
        vec![*self.numer(), *self.denom()]
    }
}

#[wasm_bindgen]
impl JsFrac {
    pub fn reduce(numer: isize, denom: isize) -> Vec<isize> {
        Rational::new(numer, denom).to_js()
    }

    pub fn from_duration(note_value: isize, dots: u8) -> Vec<isize> {
        let note_value = NoteValue::new(note_value).unwrap();
        let d = Duration::new(note_value, dots, None).duration();
        vec![*d.numer(), *d.denom()]
    }

    pub fn add(a_numer: isize, a_denom: isize, b_numer: isize, b_denom: isize) -> Vec<isize> {
        (Rational::new(a_numer, a_denom) + Rational::new(b_numer, b_denom)).to_js()
    }

    pub fn gt(a_numer: isize, a_denom: isize, b_numer: isize, b_denom: isize) -> bool {
        Rational::new(a_numer, a_denom) > Rational::new(b_numer, b_denom)
    }

    pub fn lt(a_numer: isize, a_denom: isize, b_numer: isize, b_denom: isize) -> bool {
        Rational::new(a_numer, a_denom) < Rational::new(b_numer, b_denom)
    }

    pub fn eq(a_numer: isize, a_denom: isize, b_numer: isize, b_denom: isize) -> bool {
        Rational::new(a_numer, a_denom) == Rational::new(b_numer, b_denom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        assert_eq!(JsFrac::reduce(4, 2), vec![2, 1]);
        assert_eq!(JsFrac::add(1, 2, 1, 4), vec![3, 4]);
        assert_eq!(JsFrac::gt(1, 2, 1, 3), true);
        assert_eq!(JsFrac::lt(1, 2, 1, 3), false);
        assert_eq!(JsFrac::eq(1, 2, 1, 3), false);
    }
}
