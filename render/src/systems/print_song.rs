use kurbo::Vec2;
use specs::{Join, ReadStorage, System, WriteStorage};
use staff::components::{Children, Song, Staff};
use stencil::components::StencilMap;

#[derive(Debug, Default)]
pub struct PrintSong;

impl<'a> System<'a> for PrintSong {
    type SystemData = (
        ReadStorage<'a, Song>,
        ReadStorage<'a, Staff>,
        ReadStorage<'a, Children>,
        WriteStorage<'a, StencilMap>,
    );

    fn run(&mut self, (songs, staffs, children, mut stencil_maps): Self::SystemData) {
        for (song, children, render) in (&songs, &children, &mut stencil_maps).join() {
            let mut map = StencilMap::new();
            let mut h = 5500.0;
            for &child in &children.0 {
                if let Some(staff) = staffs.get(child) {
                    for line in &staff.lines {
                        map = map.and(
                            *line,
                            if h > 0.0 {
                                Some(Vec2::new(0.0, h))
                            } else {
                                None
                            },
                        );
                        h += 3000.0;
                    }
                }
            }
            if let Some(title_stencil) = song.title_stencil {
                map = map.and(title_stencil, None);
            }
            if let Some(author_stencil) = song.author_stencil {
                map = map.and(author_stencil, None);
            }
            *render = map;
        }
    }
}
