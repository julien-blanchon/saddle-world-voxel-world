use saddle_world_voxel_world_example_support as support;

use bevy::prelude::*;
use saddle_pane::prelude::*;
use saddle_world_voxel_world::{BlockEdit, BlockId, VoxelCommand, VoxelWorldPlugin};

#[derive(Resource)]
struct EditBurstState {
    filled: bool,
}

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::srgb(0.64, 0.77, 0.94)))
        .insert_resource(support::showcase_registry())
        .insert_resource(support::showcase_generator())
        .insert_resource(support::default_config())
        .insert_resource(support::VoxelExamplePane::default())
        .insert_resource(EditBurstState { filled: false })
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Voxel World Block Editing".into(),
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
            (
                support::sync_example_pane,
                support::spin_viewer,
                trigger_boundary_edits,
            ),
        )
        .run();
}

fn setup_overlay(mut commands: Commands) {
    support::spawn_overlay(
        &mut commands,
        "Manual Block Editing",
        "Press E to toggle a small edit burst across a chunk boundary.\nThe viewer keeps orbiting so you can watch dirty chunks remesh without the example mutating itself continuously.\nPane: adjust radii and debug toggles live.",
    );
}

fn trigger_boundary_edits(
    keys: Res<ButtonInput<KeyCode>>,
    mut state: ResMut<EditBurstState>,
    mut writer: MessageWriter<VoxelCommand>,
) {
    if !keys.just_pressed(KeyCode::KeyE) {
        return;
    }

    let block = if state.filled {
        BlockId::AIR
    } else {
        support::SHOWCASE_LAMP
    };
    state.filled = !state.filled;
    writer.write(VoxelCommand::Batch(
        [
            IVec3::new(15, 10, 0),
            IVec3::new(16, 10, 0),
            IVec3::new(15, 11, 0),
            IVec3::new(16, 11, 0),
        ]
        .into_iter()
        .map(|world_pos| BlockEdit { world_pos, block })
        .collect(),
    ));
}
