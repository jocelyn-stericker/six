mod corefont;

use kurbo::{BezPath, Rect, TranslateScale, Vec2};

/// A path with precomputed bounds.
pub struct Path {
    pub(crate) outline: BezPath,
    pub(crate) bounds: Rect,
    pub(crate) advance: f64,
}

pub struct CombineStencil(pub Vec<Stencil>);

pub enum Stencil {
    Path(Path),
    Combine(CombineStencil),
    TranslateScale(TranslateScale, Box<Stencil>),
}

impl Stencil {
    /// Initialize a stencil, in staff cordinates.
    fn from_corefont(corefont: &(i32, [i32; 4], &str)) -> Stencil {
        Stencil::Path(Path {
            outline: TranslateScale::scale(1.0 / (corefont::UNITS_PER_EM as f64))
                * BezPath::from_svg(corefont.2).expect("Invalid corefont"),
            bounds: Rect::new(
                corefont.1[0] as f64 / (corefont::UNITS_PER_EM as f64),
                corefont.1[1] as f64 / (corefont::UNITS_PER_EM as f64),
                corefont.1[2] as f64 / (corefont::UNITS_PER_EM as f64),
                corefont.1[3] as f64 / (corefont::UNITS_PER_EM as f64),
            ),
            advance: corefont.0 as f64 / (corefont::UNITS_PER_EM as f64),
        })
    }

    pub fn time_sig_number(mut number: u8) -> Stencil {
        let mut digits = Vec::with_capacity(3);
        while number > 0 {
            digits.push(match number % 10 {
                0 => Self::from_corefont(&corefont::TIME_SIG0),
                1 => Self::from_corefont(&corefont::TIME_SIG1),
                2 => Self::from_corefont(&corefont::TIME_SIG2),
                3 => Self::from_corefont(&corefont::TIME_SIG3),
                4 => Self::from_corefont(&corefont::TIME_SIG4),
                5 => Self::from_corefont(&corefont::TIME_SIG5),
                6 => Self::from_corefont(&corefont::TIME_SIG6),
                7 => Self::from_corefont(&corefont::TIME_SIG7),
                8 => Self::from_corefont(&corefont::TIME_SIG8),
                9 => Self::from_corefont(&corefont::TIME_SIG9),
                _ => unreachable!(),
            });
            number /= 10;
        }

        digits.reverse();

        let mut advance = 0.0;
        let mut stencils = Vec::with_capacity(digits.len());
        for digit in digits {
            let digit_advance = digit.advance();
            stencils.push(digit.with_translation(Vec2::new(advance, 0.0)));
            advance += digit_advance;
        }

        Self::combine(stencils)
    }

    pub fn time_sig_fraction(num: u8, den: u8) -> Stencil {
        let mut num = Self::time_sig_number(num);
        let mut den = Self::time_sig_number(den);

        let num_adv = num.advance();
        let den_adv = den.advance();

        if num_adv > den_adv {
            num = num.with_translation(Vec2::new(0.0, 0.247));
            den = den.with_translation(Vec2::new((num_adv - den_adv) / 2.0, -0.247));
        } else {
            num = num.with_translation(Vec2::new((den_adv - num_adv) / 2.0, 0.247));
            den = den.with_translation(Vec2::new(0.0, -0.247));
        }

        Stencil::combine(vec![num, den])
    }

    pub fn time_sig_common() -> Stencil {
        Self::from_corefont(&corefont::TIME_SIG_COMMON)
    }

    pub fn time_sig_cut() -> Stencil {
        Self::from_corefont(&corefont::TIME_SIG_CUT_COMMON)
    }

    pub fn time_sig_cancel() -> Stencil {
        Self::from_corefont(&corefont::TIME_SIG_X)
    }

    pub fn combine(stencils: Vec<Stencil>) -> Stencil {
        Stencil::Combine(CombineStencil(stencils))
    }

    pub fn with_translation(self, offset: Vec2) -> Stencil {
        Stencil::TranslateScale(TranslateScale::translate(offset), Box::new(self))
    }

    pub fn with_scale(self, scale: f64) -> Stencil {
        Stencil::TranslateScale(TranslateScale::scale(scale), Box::new(self))
    }

    pub fn and(self, other: Stencil) -> Stencil {
        match (self, other) {
            (
                Stencil::Combine(CombineStencil(mut mine)),
                Stencil::Combine(CombineStencil(mut theirs)),
            ) => {
                mine.append(&mut theirs);
                Stencil::Combine(CombineStencil(mine))
            }
            (Stencil::Combine(CombineStencil(mut stencils)), other) => {
                stencils.push(other);
                Stencil::Combine(CombineStencil(stencils))
            }
            (me, Stencil::Combine(CombineStencil(mut stencils))) => {
                stencils.push(me);
                Stencil::Combine(CombineStencil(stencils))
            }
            (inner, other) => Stencil::Combine(CombineStencil(vec![inner, other])),
        }
    }

    pub fn and_right(self, other: Stencil) -> Stencil {
        let advance = self.advance();
        self.and(other.with_translation(Vec2::new(advance, 0.0)))
    }

    pub fn rect(&self) -> Rect {
        match self {
            Stencil::Path(path) => path.bounds,
            Stencil::TranslateScale(ts, child) => *ts * child.rect(),
            Stencil::Combine(combine) => {
                let mut rect = combine.0.first().map(|f| f.rect()).unwrap_or_default();

                for child_rect in combine.0.iter().skip(1).map(|c| c.rect()) {
                    if child_rect.x0 < rect.x0 {
                        rect.x0 = child_rect.x0;
                    }
                    if child_rect.y0 < rect.y0 {
                        rect.x0 = child_rect.y0;
                    }
                    if child_rect.x1 > rect.x1 {
                        rect.x1 = child_rect.x1;
                    }
                    if child_rect.y1 > rect.y1 {
                        rect.x1 = child_rect.y1;
                    }
                }

                rect
            }
        }
    }

    pub fn advance(&self) -> f64 {
        match self {
            Stencil::Path(path) => path.advance,
            Stencil::TranslateScale(ts, child) => {
                let (translation, scale) = ts.as_tuple();
                translation.x + scale * child.advance()
            }
            Stencil::Combine(combine) => {
                combine
                    .0
                    .iter()
                    .map(|c| c.advance())
                    .fold(0.0, |max_so_far, child_adv| {
                        if child_adv > max_so_far {
                            child_adv
                        } else {
                            max_so_far
                        }
                    })
            }
        }
    }

    /// Convert from staff-size (1 unit is 1 staff) to paper-size (1 unit is 1 mm)
    ///
    /// Behind Bars, p483.
    ///
    /// Rastal sizes vary from 0 to 8, where 0 is large and 8 is small.
    ///  - 0 and 1 are used for educational music.
    ///  - 2 is not generally used, but is sometimes used for piano music/songs.
    ///  - 3-4 are commonly used for single-staff-parts, piano music, and songs.
    ///  - 5 is less commonly used for single-staff-parts, piano music, and songs.
    ///  - 6-7 are used for choral music, cue saves, or ossia.
    ///  - 8 is used for full scores.
    pub fn with_paper_size(self, rastal: u8) -> Stencil {
        match rastal {
            0 => self.with_scale(9.2),
            1 => self.with_scale(7.9),
            2 => self.with_scale(7.4),
            3 => self.with_scale(7.0),
            4 => self.with_scale(6.5),
            5 => self.with_scale(6.0),
            6 => self.with_scale(5.5),
            7 => self.with_scale(4.8),
            8 => self.with_scale(3.7),
            _ => panic!("Expected rastal size <= 8"),
        }
    }

    pub fn to_svg(&self) -> String {
        match self {
            Stencil::Path(path) => ["<path d=\"", &path.outline.to_svg(), "\" />"].concat(),
            Stencil::TranslateScale(ts, child) => {
                let (translation, scale) = ts.as_tuple();

                [
                    "<g transform=\"translate(",
                    &translation.x.to_string(),
                    ",",
                    &translation.y.to_string(),
                    ") ",
                    "scale(",
                    &scale.to_string(),
                    ",",
                    &scale.to_string(),
                    ")\">",
                    &child.to_svg(),
                    "</g>",
                ]
                .concat()
            }
            Stencil::Combine(combine) => {
                let mut parts = Vec::with_capacity(combine.0.len() + 2);
                parts.push("<g>".to_owned());
                for part in &combine.0 {
                    parts.push(part.to_svg());
                }
                parts.push("</g>".to_owned());
                parts.concat()
            }
        }
    }

    pub fn to_svg_doc_for_testing(&self) -> String {
        [
            "<svg viewBox=\"0 0 215.9 279.4\" width=\"215.9mm\" height=\"279.4mm\" xmlns=\"http://www.w3.org/2000/svg\" xmlns:xlink=\"http://www.w3.org/1999/xlink\"><g transform=\"scale(1, -1)\">",
            &self.to_svg(),
            "</g></svg>\n"
        ].concat()
    }
}

impl Default for Stencil {
    fn default() -> Stencil {
        Stencil::Combine(CombineStencil(vec![]))
    }
}

#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn time_signatures() {
        assert_eq!(
            Stencil::time_sig_fraction(4, 4)
                .and_right(Stencil::time_sig_fraction(3, 4))
                .and_right(Stencil::time_sig_fraction(5, 4))
                .and_right(Stencil::time_sig_fraction(7, 4))
                .and_right(Stencil::time_sig_fraction(12, 8))
                .and_right(Stencil::time_sig_fraction(6, 16))
                .and_right(Stencil::time_sig_fraction(9, 8))
                .and_right(Stencil::time_sig_fraction(6, 8))
                .and_right(Stencil::time_sig_common())
                .and_right(Stencil::time_sig_cut())
                .and_right(Stencil::time_sig_cancel())
                .with_translation(Vec2::new(0.0, -0.5))
                .with_paper_size(3)
                .to_svg_doc_for_testing(),
            include_str!("./test_time_signature_stencils.svg")
        );
    }
}
