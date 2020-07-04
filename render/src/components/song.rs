#[derive(Debug)]
pub struct Song {
    freeze_spacing: Option<isize>,
    prev_freeze_spacing: Option<isize>,

    /// In mm
    width: f64,

    /// In mm
    height: f64,

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
    rastal_size: u8,

    title: String,
    title_width: f64,
    title_stencil: Option<Entity>,

    author: String,
    author_width: f64,
    author_stencil: Option<Entity>,
}

impl Component for Song {
    type Storage = HashMapStorage<Self>;
}

impl Default for Song {
    fn default() -> Song {
        Song {
            freeze_spacing: None,
            prev_freeze_spacing: None,

            width: 0f64,
            height: 0f64,
            rastal_size: 3,
            title: String::default(),
            title_width: 0f64,
            title_stencil: None,
            author: String::default(),
            author_width: 0f64,
            author_stencil: None,
        }
    }
}

impl Song {
    pub fn scale(&self) -> f64 {
        match self.rastal_size {
            0 => 9.2 / 1000.0,
            1 => 7.9 / 1000.0,
            2 => 7.4 / 1000.0,
            3 => 7.0 / 1000.0,
            4 => 6.5 / 1000.0,
            5 => 6.0 / 1000.0,
            6 => 5.5 / 1000.0,
            7 => 4.8 / 1000.0,
            8 => 3.7 / 1000.0,
            _ => panic!("Expected rastal size <= 8"),
        }
    }
}

