use std::collections::HashMap;

use crate::Stencil;
use entity::Entity;
use kurbo::{TranslateScale, Vec2};

#[derive(Debug, Clone, Default)]
pub struct StencilMap {
    transform: Option<TranslateScale>,
    children: HashMap<Entity, (isize, Option<TranslateScale>)>,
    top_zindex: isize,
}

impl StencilMap {
    pub fn and(mut self, child: Entity, transform: Option<TranslateScale>) -> StencilMap {
        self.top_zindex += 1;
        let z = self.top_zindex;
        self.children.insert(child, (z, transform));

        self
    }

    pub fn without(mut self, child: Entity) -> StencilMap {
        self.children.remove(&child);

        self
    }

    pub fn with_transform(mut self, transform: Option<TranslateScale>) -> StencilMap {
        self.transform = transform;

        self
    }

    pub fn with_translation(mut self, offset: Vec2) -> StencilMap {
        self.transform =
            Some(TranslateScale::translate(offset) * self.transform.unwrap_or_default());
        self
    }

    pub fn with_scale(mut self, scale: f64) -> StencilMap {
        self.transform = Some(TranslateScale::scale(scale) * self.transform.unwrap_or_default());
        self
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
    pub fn with_paper_size(self, rastal: u8) -> StencilMap {
        match rastal {
            0 => self.with_scale(9.2 / 1000.0),
            1 => self.with_scale(7.9 / 1000.0),
            2 => self.with_scale(7.4 / 1000.0),
            3 => self.with_scale(7.0 / 1000.0),
            4 => self.with_scale(6.5 / 1000.0),
            5 => self.with_scale(6.0 / 1000.0),
            6 => self.with_scale(5.5 / 1000.0),
            7 => self.with_scale(4.8 / 1000.0),
            8 => self.with_scale(3.7 / 1000.0),
            _ => panic!("Expected rastal size <= 8"),
        }
    }

    pub fn get_sorted_children(&self) -> Vec<(Entity, Option<TranslateScale>)> {
        let mut sorted = self.children.iter().collect::<Vec<_>>();
        sorted.sort_by_key(|k| (k.1).0);

        sorted
            .into_iter()
            .map(|(entity, value)| (*entity, value.1))
            .collect()
    }

    pub fn to_json(&self) -> String {
        let sorted = self.get_sorted_children();
        let len = sorted.len();

        [
            "[",
            &sorted
                .into_iter()
                .enumerate()
                .map(|(i, (entity, transform))| {
                    if let Some(transform) = transform {
                        let (translate, scale) = transform.as_tuple();
                        [
                            "[",
                            &entity.id().to_string(),
                            ",",
                            &translate.x.to_string(),
                            ",",
                            &translate.y.to_string(),
                            ",",
                            &scale.to_string(),
                            if i == len - 1 { "]" } else { "]," },
                        ]
                        .concat()
                    } else {
                        [
                            "[",
                            &entity.id().to_string(),
                            if i == len - 1 { "]" } else { "]," },
                        ]
                        .concat()
                    }
                })
                .collect::<String>(),
            "]",
        ]
        .concat()
    }

    pub fn to_svg(
        &self,
        stencil_maps: &HashMap<Entity, StencilMap>,
        stencils: &HashMap<Entity, Stencil>,
    ) -> String {
        let children: String = self
            .get_sorted_children()
            .into_iter()
            .map(|(entity, transform)| {
                let child_svg;
                if let Some(map) = stencil_maps.get(&entity) {
                    child_svg = map.to_svg(stencil_maps, stencils);
                } else if let Some(stencil) = stencils.get(&entity) {
                    child_svg = stencil.to_svg();
                } else {
                    child_svg = String::new();
                }

                if let Some(transform) = transform {
                    let (translation, scale) = transform.as_tuple();
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
                        &child_svg,
                        "</g>",
                    ]
                    .concat()
                } else {
                    child_svg
                }
            })
            .collect();

        if let Some(transform) = self.transform {
            let (translation, scale) = transform.as_tuple();

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
                &children,
                "</g>",
            ]
            .concat()
        } else {
            ["<g>", &children, "</g>"].concat()
        }
    }

    pub fn to_svg_doc_for_testing(
        &self,
        stencil_maps: &HashMap<Entity, StencilMap>,
        stencils: &HashMap<Entity, Stencil>,
    ) -> String {
        [
            "<svg viewBox=\"0 0 215.9 279.4\" width=\"215.9mm\" height=\"279.4mm\" xmlns=\"http://www.w3.org/2000/svg\" xmlns:xlink=\"http://www.w3.org/1999/xlink\"><g transform=\"scale(1, -1)\">",
            &self.to_svg(stencil_maps, stencils),
            "</g></svg>\n"
        ].concat()
    }
}
