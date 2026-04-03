use saddle_world_voxel_world_example_support as support;

use bevy::prelude::*;
use saddle_pane::prelude::*;
use saddle_world_voxel_world::{
    BlockEdit, BlockId, SaveMode, SavePolicy, VoxelCommand, VoxelWorldPlugin,
};

#[derive(Resource)]
struct SavePulse(Timer);

fn main() {
    let mut config = support::default_config();
    config.save_policy = SavePolicy {
        mode: SaveMode::DeltaRegions,
        root: "target/voxel_world_example_save".into(),
        autosave_interval_seconds: 2.0,
        ..SavePolicy::default()
    };

    App::new()
        .insert_resource(ClearColor(Color::srgb(0.62, 0.78, 0.95)))
        .insert_resource(config)
        .insert_resource(SavePulse(Timer::from_seconds(0.75, TimerMode::Repeating)))
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Voxel World Persistence".into(),
                resolution: (1280, 840).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(support::pane_plugins())
        .add_plugins(VoxelWorldPlugin::default())
        .register_pane::<support::VoxelExamplePane>()
        .add_systems(Startup, support::spawn_scene)
        .add_systems(Update, (support::sync_example_pane, support::spin_viewer, stamp_edits))
        .run();
}

fn stamp_edits(
    time: Res<Time>,
    mut pulse: ResMut<SavePulse>,
    mut writer: MessageWriter<VoxelCommand>,
) {
    if pulse.0.tick(time.delta()).just_finished() {
        let index = (time.elapsed_secs() as i32) % 6;
        writer.write(VoxelCommand::SetBlock(BlockEdit {
            world_pos: IVec3::new(index - 3, 15, 0),
            block: if index % 2 == 0 {
                BlockId::LAMP
            } else {
                BlockId::STONE
            },
        }));
    }
}
