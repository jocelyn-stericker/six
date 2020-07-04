use std::collections::HashMap;

use super::stencil::Stencil;
use kurbo::{Rect, Vec2};
use specs::{Component, Entity, VecStorage};

#[derive(Debug, Clone, Default)]
pub struct StencilMap {
    pub(crate) translate: Option<Vec2>,
    explicit_rect: Option<Rect>,
    children: HashMap<Entity, (isize, Option<Vec2>)>,
    top_zindex: isize,
}

impl Component for StencilMap {
    type Storage = VecStorage<StencilMap>;
}

impl StencilMap {
    pub fn new() -> StencilMap {
        StencilMap::default()
    }

    pub fn and(mut self, child: Entity, transform: Option<Vec2>) -> StencilMap {
        self.top_zindex += 1;
        let z = self.top_zindex;
        self.children.insert(child, (z, transform));

        self
    }

    pub fn without(mut self, child: Entity) -> StencilMap {
        self.children.remove(&child);

        self
    }

    pub fn with_translation(mut self, offset: Vec2) -> StencilMap {
        self.translate = Some(offset);
        self
    }

    pub fn explicit_rect(&self) -> Option<Rect> {
        self.explicit_rect
    }

    pub fn set_explicit_rect(&mut self, rect: Rect) {
        self.explicit_rect = Some(rect);
    }

    pub fn get_sorted_children(&self) -> Vec<(Entity, Option<Vec2>)> {
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
                .map(|(i, (entity, translate))| {
                    let translate = match (self.translate, translate) {
                        (Some(a), Some(b)) => Some(a + b),
                        (Some(a), None) | (None, Some(a)) => Some(a),
                        (None, None) => None,
                    };

                    if let Some(translate) = translate {
                        [
                            "[",
                            &entity.id().to_string(),
                            ",",
                            &translate.x.round().to_string(),
                            ",",
                            &translate.y.round().to_string(),
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
            .map(|(entity, translate)| {
                let child_svg;
                if let Some(map) = stencil_maps.get(&entity) {
                    child_svg = map.to_svg(stencil_maps, stencils);
                } else if let Some(stencil) = stencils.get(&entity) {
                    child_svg = stencil.to_svg();
                } else {
                    child_svg = String::new();
                }

                if let Some(translate) = translate {
                    [
                        "<g transform=\"translate(",
                        &translate.x.to_string(),
                        ",",
                        &translate.y.to_string(),
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

        if let Some(translate) = self.translate {
            [
                "<g transform=\"translate(",
                &translate.x.to_string(),
                ",",
                &translate.y.to_string(),
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
        scale: f64,
        stencil_maps: &HashMap<Entity, StencilMap>,
        stencils: &HashMap<Entity, Stencil>,
    ) -> String {
        [
            "<svg viewBox=\"0 0 ",
            &(215.9 / scale).round().to_string(),
            " ",
            &(279.4 / scale).round().to_string(),
            "\" width=\"215.9mm\" height=\"279.4mm\" xmlns=\"http://www.w3.org/2000/svg\" xmlns:xlink=\"http://www.w3.org/1999/xlink\">",
            &self.to_svg(stencil_maps, stencils),
            "</svg>\n"
        ].concat()
    }
}
