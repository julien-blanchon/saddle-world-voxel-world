use saddle_world_voxel_world_example_support as support;

use bevy::prelude::*;
use saddle_pane::prelude::*;
use saddle_world_voxel_world::{ChunkViewer, ChunkViewerSettings, VoxelWorldPlugin};

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::srgb(0.58, 0.74, 0.90)))
        .insert_resource(support::default_config())
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Voxel World Multi Viewer".into(),
                resolution: (1280, 840).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(support::pane_plugins())
        .add_plugins(VoxelWorldPlugin::default())
        .register_pane::<support::VoxelExamplePane>()
        .add_systems(Startup, (support::spawn_scene, spawn_secondary_viewer))
        .add_systems(
            Update,
            (
                support::sync_example_pane,
                support::spin_viewer,
                move_secondary_viewer,
            ),
        )
        .run();
}

fn spawn_secondary_viewer(mut commands: Commands) {
    commands.spawn((
        Name::new("Secondary Viewer"),
        ChunkViewer,
        ChunkViewerSettings {
            request_radius: 3,
            keep_radius: 5,
            priority: 3,
        },
        support::SecondaryViewer,
        Transform::from_xyz(-24.0, 18.0, -24.0),
        GlobalTransform::default(),
    ));
}

fn move_secondary_viewer(
    time: Res<Time>,
    mut viewers: Query<&mut Transform, With<support::SecondaryViewer>>,
) {
    for mut transform in &mut viewers {
        transform.translation = Vec3::new(
            26.0 * (time.elapsed_secs() * 0.23).cos(),
            16.0,
            26.0 * (time.elapsed_secs() * 0.31).sin(),
        );
    }
}
