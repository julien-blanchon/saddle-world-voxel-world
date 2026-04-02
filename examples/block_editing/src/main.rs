use saddle_world_voxel_world_example_support as support;

use bevy::prelude::*;
use saddle_world_voxel_world::{BlockEdit, BlockId, VoxelCommand, VoxelWorldPlugin};

#[derive(Resource)]
struct EditTimer(Timer);

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::srgb(0.64, 0.77, 0.94)))
        .insert_resource(support::default_config())
        .insert_resource(EditTimer(Timer::from_seconds(0.35, TimerMode::Repeating)))
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Voxel World Block Editing".into(),
                resolution: (1280, 840).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(VoxelWorldPlugin::default())
        .add_systems(Startup, support::spawn_scene)
        .add_systems(Update, (support::spin_viewer, pulse_edits))
        .run();
}

fn pulse_edits(
    time: Res<Time>,
    mut timer: ResMut<EditTimer>,
    mut writer: MessageWriter<VoxelCommand>,
) {
    if !timer.0.tick(time.delta()).just_finished() {
        return;
    }
    let t = time.elapsed_secs() * 2.0;
    let world_pos = IVec3::new(t.cos().round() as i32 * 4, 14, t.sin().round() as i32 * 4);
    writer.write(VoxelCommand::SetBlock(BlockEdit {
        world_pos,
        block: if (time.elapsed_secs() as i32) % 2 == 0 {
            BlockId::AIR
        } else {
            BlockId::LAMP
        },
    }));
}
