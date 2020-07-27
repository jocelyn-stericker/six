use crate::{
    components::Css,
    systems::{DeleteOrphans, PrintMeta, PrintSong, UpdateKeepSpacing, UpdateWorldBbox},
};
use rhythm::components::{Bar, Spacing};
use specs::{RunNow, World, WorldExt};
use staff::{
    components::{
        Beam, BeamForChord, Children, Chord, Context, Cursor, FlagAttachment, LineOfStaff,
        Signature, Song, SpaceTimeWarp, Staff,
    },
    resources::{KeepSpacing, Root},
    systems::{
        ApplySpaceTimeWarp, BreakIntoLines, DraftBeam, MaintainAutorests, PrintBeam, PrintChord,
        PrintCursor, PrintSignature, PrintStaff, PrintStaffLines, RecordSpaceTimeWarp, SpaceBeam,
        UpdateContext,
    },
};
use stencil::components::{Parent, Stencil, StencilMap, WorldBbox};

#[derive(Default)]
struct Systems {
    apply_space_time_warp: ApplySpaceTimeWarp,
    draft_beam: DraftBeam,
    print_beam: PrintBeam,
    print_chord: PrintChord,
    record_space_time_warp: RecordSpaceTimeWarp,
    space_beam: SpaceBeam,
    maintain_autorests: MaintainAutorests,
    break_into_lines: BreakIntoLines,
    print_signature: PrintSignature,
    print_staff: PrintStaff,
    print_staff_lines: PrintStaffLines,
    print_cursor: PrintCursor,
    delete_orphans: DeleteOrphans,
    print_meta: PrintMeta,
    print_song: PrintSong,
    update_keep_spacing: UpdateKeepSpacing,
    update_world_bbox: UpdateWorldBbox,
    update_context: UpdateContext,
}

pub struct State {
    pub(crate) world: World,
    systems: Systems,
}

impl Default for State {
    fn default() -> State {
        let mut world = World::new();
        world.insert(KeepSpacing(false));
        world.insert(Root(None));

        world.register::<Bar>();
        world.register::<Beam>();
        world.register::<BeamForChord>();
        world.register::<Children>();
        world.register::<Chord>();
        world.register::<Context>();
        world.register::<Css>();
        world.register::<Cursor>();
        world.register::<FlagAttachment>();
        world.register::<LineOfStaff>();
        world.register::<Parent>();
        world.register::<Signature>();
        world.register::<Song>();
        world.register::<SpaceTimeWarp>();
        world.register::<Spacing>();
        world.register::<Staff>();
        world.register::<Stencil>();
        world.register::<StencilMap>();
        world.register::<WorldBbox>();

        Self {
            world,
            systems: Default::default(),
        }
    }
}

impl State {
    pub fn exec(&mut self) {
        self.systems.delete_orphans.run_now(&self.world);
        self.systems.update_keep_spacing.run_now(&self.world);
        self.systems.maintain_autorests.run_now(&self.world);
        self.world.maintain();

        self.systems.draft_beam.run_now(&self.world);
        self.systems.update_context.run_now(&self.world);

        self.systems.print_chord.run_now(&self.world);
        self.systems.print_signature.run_now(&self.world);

        self.systems.apply_space_time_warp.run_now(&self.world);
        self.systems.break_into_lines.run_now(&self.world);
        self.systems.record_space_time_warp.run_now(&self.world);

        self.systems.space_beam.run_now(&self.world);
        self.systems.print_beam.run_now(&self.world);
        self.systems.print_cursor.run_now(&self.world);

        self.systems.print_staff.run_now(&self.world);
        self.systems.print_staff_lines.run_now(&self.world);
        self.systems.print_meta.run_now(&self.world);
        self.systems.print_song.run_now(&self.world);

        self.systems.update_world_bbox.run_now(&self.world);
        self.world.maintain();

        if let Some(root) = self.world.read_resource::<Root>().0 {
            if let Some(root) = self.world.write_component::<Song>().get_mut(root) {
                root.prev_freeze_spacing = root.freeze_spacing;
            }
        }
    }
}
