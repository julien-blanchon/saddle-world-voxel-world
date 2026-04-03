use saddle_world_voxel_world_example_support as support;

use bevy::prelude::*;
use saddle_pane::prelude::*;
use saddle_world_voxel_world::VoxelWorldPlugin;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::srgb(0.58, 0.74, 0.90)))
        .insert_resource(support::default_config())
        .insert_resource(support::VoxelExamplePane {
            show_chunk_bounds: true,
            show_viewer_radii: true,
            ..default()
        })
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Voxel World Debug Gizmos".into(),
                resolution: (1280, 840).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(support::pane_plugins())
        .add_plugins(VoxelWorldPlugin::default())
        .register_pane::<support::VoxelExamplePane>()
        .add_systems(Startup, support::spawn_scene)
        .add_systems(Update, (support::sync_example_pane, support::spin_viewer))
        .run();
}
