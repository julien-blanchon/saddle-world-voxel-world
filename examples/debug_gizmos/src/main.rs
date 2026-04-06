//! Debug gizmos example — chunk bounds and viewer radii visualization.
//!
//! Same setup as basic, but with `VoxelDebugConfig` fields enabled to draw
//! chunk bounding boxes and viewer streaming radii as gizmos. Toggle these
//! via the saddle-pane UI.

use saddle_world_voxel_world_example_support as support;

use bevy::prelude::*;
use saddle_pane::prelude::*;
use saddle_world_voxel_world::{
    SaveMode, SavePolicy, VoxelDebugConfig, VoxelWorldConfig, VoxelWorldPlugin,
};

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::srgb(0.58, 0.74, 0.90)))
        .insert_resource(support::showcase_registry())
        .insert_resource(support::showcase_generator())
        // --------------- Voxel world configuration ---------------
        .insert_resource(VoxelWorldConfig {
            request_radius: 4,
            keep_radius: 6,
            max_chunk_requests_per_frame: 16,
            save_policy: SavePolicy {
                mode: SaveMode::Disabled,
                ..SavePolicy::default()
            },
            ..VoxelWorldConfig::default()
        })
        // --------------- Debug config: show gizmos from the start ---------------
        .insert_resource(VoxelDebugConfig {
            show_chunk_bounds: true,
            show_viewer_radii: true,
            ..default()
        })
        // --------------- Window ---------------
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Voxel World — Debug Gizmos".into(),
                resolution: (1280, 840).into(),
                ..default()
            }),
            ..default()
        }))
        // --------------- Pane (debug UI) — pre-enable gizmo toggles ---------------
        .add_plugins(support::pane_plugins())
        .insert_resource(support::VoxelExamplePane {
            show_chunk_bounds: true,
            show_viewer_radii: true,
            ..default()
        })
        .register_pane::<support::VoxelExamplePane>()
        // --------------- Plugin ---------------
        .add_plugins(VoxelWorldPlugin::default())
        // --------------- Systems ---------------
        .add_systems(Startup, (support::spawn_scene, setup_overlay))
        .add_systems(Update, (support::sync_example_pane, support::spin_viewer))
        .run();
}

fn setup_overlay(mut commands: Commands) {
    support::spawn_overlay(
        &mut commands,
        "Debug Gizmos",
        "This example uses the same optional showcase preset, but starts with chunk bounds and viewer radii enabled.\nPane: toggle gizmos live and adjust request/keep radii while the camera auto-orbits.",
    );
}
