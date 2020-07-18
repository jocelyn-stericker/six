use specs::{Read, ReadStorage, System, Write};
use staff::{
    components::Song,
    resources::{KeepSpacing, Root},
};

#[derive(Debug, Default)]
pub struct UpdateKeepSpacing;

impl<'a> System<'a> for UpdateKeepSpacing {
    type SystemData = (
        Read<'a, Root>,
        ReadStorage<'a, Song>,
        Write<'a, KeepSpacing>,
    );

    fn run(&mut self, (root, songs, mut keep_spacing): Self::SystemData) {
        keep_spacing.0 = root
            .0
            .and_then(|root| songs.get(root))
            .map(|root| {
                root.freeze_spacing.is_some()
                    && (root.freeze_spacing == root.prev_freeze_spacing
                        || root.prev_freeze_spacing.is_none())
            })
            .unwrap_or(false);
    }
}
