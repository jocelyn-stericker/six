use kurbo::{Line, Vec2};

/// Normalized normal vector of line.
pub fn normal(line: Line) -> Vec2 {
    let n = tangent(line);
    Vec2::new(n.y, -n.x)
}

/// Normalized tangent vector of line.
pub fn tangent(line: Line) -> Vec2 {
    (line.p1 - line.p0).normalize()
}

pub const BEZIER_CIRCLE_FACTOR: f64 = 0.552_284_8;

pub fn escape(s: &str) -> String {
    // From https://doc.rust-lang.org/1.1.0/src/rustdoc/html/escape.rs.html#20
    let mut parts = vec![];

    // Because the internet is always right, turns out there's not that many
    // characters to escape: http://stackoverflow.com/questions/7381974
    let pile_o_bits = s;
    let mut last = 0;
    for (i, ch) in s.bytes().enumerate() {
        match ch as char {
            '<' | '>' | '&' | '\'' | '"' => {
                parts.push(&pile_o_bits[last..i]);
                let replace = match ch as char {
                    '>' => "&gt;",
                    '<' => "&lt;",
                    '&' => "&amp;",
                    '\'' => "&#39;",
                    '"' => "&quot;",
                    _ => unreachable!(),
                };
                parts.push(replace);
                last = i + 1;
            }
            _ => {}
        }
    }

    if last < s.len() {
        parts.push(&pile_o_bits[last..]);
    }

    parts.concat()
}
