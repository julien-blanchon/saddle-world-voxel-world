//! Basic voxel world example — streaming and meshing with the optional showcase preset.
//!
//! Demonstrates the minimal setup for `VoxelWorldPlugin`: insert a
//! `VoxelWorldConfig`, opt into a preset `BlockRegistry` + `VoxelWorldGenerator`,
//! spawn a camera with `ChunkViewer` + `ChunkViewerSettings`, and let the plugin
//! stream/generate/mesh chunks around the viewer.

use saddle_world_voxel_world_example_support as support;

use bevy::prelude::*;
use saddle_pane::prelude::*;
use saddle_world_voxel_world::{SaveMode, SavePolicy, VoxelWorldConfig, VoxelWorldPlugin};

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::srgb(0.60, 0.76, 0.92)))
        .insert_resource(support::showcase_registry())
        .insert_resource(support::showcase_generator())
        // --------------- Voxel world configuration ---------------
        .insert_resource(VoxelWorldConfig {
            request_radius: 4, // request chunks within 4-chunk radius
            keep_radius: 6,    // keep loaded chunks within 6-chunk radius
            max_chunk_requests_per_frame: 16,
            save_policy: SavePolicy {
                mode: SaveMode::Disabled,
                ..SavePolicy::default()
            },
            ..VoxelWorldConfig::default()
        })
        // --------------- Window ---------------
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Voxel World — Basic".into(),
                resolution: (1280, 840).into(),
                ..default()
            }),
            ..default()
        }))
        // --------------- Pane (debug UI) ---------------
        .add_plugins(support::pane_plugins())
        .insert_resource(support::VoxelExamplePane::default())
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
        "Basic Streaming Showcase",
        "This example opts into the showcase terrain preset from example support.\nPane: tune request/keep radii and orbit motion.\nCamera: auto-orbits the streamed chunk field for quick visual inspection.",
    );
}
