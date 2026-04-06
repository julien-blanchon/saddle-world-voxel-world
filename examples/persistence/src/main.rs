use saddle_world_voxel_world_example_support as support;

use bevy::prelude::*;
use saddle_pane::prelude::*;
use saddle_world_voxel_world::{BlockEdit, SaveMode, SavePolicy, VoxelCommand, VoxelWorldPlugin};

#[derive(Resource)]
struct SaveStampState {
    next_slot: i32,
}

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
        .insert_resource(support::showcase_registry())
        .insert_resource(support::showcase_generator())
        .insert_resource(config)
        .insert_resource(support::VoxelExamplePane::default())
        .insert_resource(SaveStampState { next_slot: 0 })
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
        .add_systems(Startup, (support::spawn_scene, setup_overlay))
        .add_systems(
            Update,
            (support::sync_example_pane, support::spin_viewer, stamp_edit),
        )
        .run();
}

fn setup_overlay(mut commands: Commands) {
    support::spawn_overlay(
        &mut commands,
        "Persistence",
        "Press E to stamp one more manual edit into the save line near the origin.\nThe example writes delta-region files under target/voxel_world_example_save.\nPane: tweak streaming/debug settings while persistence keeps using the same optional showcase preset.",
    );
}

fn stamp_edit(
    keys: Res<ButtonInput<KeyCode>>,
    mut state: ResMut<SaveStampState>,
    mut writer: MessageWriter<VoxelCommand>,
) {
    if keys.just_pressed(KeyCode::KeyE) {
        let world_pos = IVec3::new(state.next_slot - 3, 15, 0);
        let block = if state.next_slot % 2 == 0 {
            support::SHOWCASE_LAMP
        } else {
            support::SHOWCASE_STONE
        };
        writer.write(VoxelCommand::SetBlock(BlockEdit { world_pos, block }));
        state.next_slot = (state.next_slot + 1) % 6;
    }
}
