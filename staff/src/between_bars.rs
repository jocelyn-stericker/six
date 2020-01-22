use stencil::Stencil;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[derive(Debug)]
pub enum Barline {
    Normal,
    Final,
}

#[derive(Default, Debug)]
pub struct BetweenBars {
    pub clef: bool,
    pub time: Option<(u8, u8)>,
    pub barline: Option<Barline>,
}

impl BetweenBars {
    pub fn render(&self) -> Stencil {
        let mut stencil = Stencil::default();

        match self.barline {
            Some(Barline::Normal) => {
                stencil = stencil
                    .and_right(Stencil::padding(100.0))
                    .and_right(Stencil::barline_thin(0.0, -500.0, 500.0))
            }
            Some(Barline::Final) => {
                stencil = stencil
                    .and_right(Stencil::padding(100.0))
                    .and_right(Stencil::barline_thin(0.0, -500.0, 500.0))
                    .and_right(Stencil::padding(125.0))
                    .and_right(Stencil::barline_thick(0.0, -500.0, 500.0));
            }
            None => {}
        }

        if self.clef {
            stencil = stencil
                .and_right(Stencil::padding(100.0))
                .and_right(Stencil::clef_unpitched())
                .and_right(Stencil::padding(100.0));
        }

        if let Some((num, den)) = self.time {
            stencil = stencil.and_right(Stencil::time_sig_fraction(num, den));
        }

        stencil
    }
}
