use kurbo::Point;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BeamAttachment {
    pub stem_start: Point,
    pub extreme_y: f64,
    pub entering: u8,
    pub leaving: u8,
}
