//! Basic voxel world example — a procedurally generated voxel terrain.
//!
//! Demonstrates the minimal setup for `VoxelWorldPlugin`: insert a
//! `VoxelWorldConfig` resource, spawn a camera with `ChunkViewer` +
//! `ChunkViewerSettings`, and let the plugin generate/mesh chunks around
//! the viewer.

use saddle_world_voxel_world_example_support as support;

use bevy::prelude::*;
use saddle_pane::prelude::*;
use saddle_world_voxel_world::{
    ChunkViewer, ChunkViewerSettings, SaveMode, SavePolicy, VoxelWorldConfig, VoxelWorldPlugin,
};

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::srgb(0.60, 0.76, 0.92)))
        // --------------- Voxel world configuration ---------------
        .insert_resource(VoxelWorldConfig {
            request_radius: 4,              // request chunks within 4-chunk radius
            keep_radius: 6,                 // keep loaded chunks within 6-chunk radius
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
        .add_systems(Startup, setup_scene)
        .add_systems(Update, (support::sync_example_pane, orbit_viewer))
        .run();
}

fn setup_scene(mut commands: Commands) {
    // Directional light
    commands.spawn((
        Name::new("Sun"),
        DirectionalLight {
            illuminance: light_consts::lux::OVERCAST_DAY,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.9, 0.8, 0.0)),
    ));
    commands.insert_resource(GlobalAmbientLight {
        brightness: 120.0,
        ..default()
    });

    // Camera with ChunkViewer — the plugin streams chunks around this entity.
    commands.spawn((
        Name::new("Main Camera"),
        Camera3d::default(),
        ChunkViewer,
        ChunkViewerSettings {
            request_radius: 4,
            keep_radius: 6,
            priority: 10,
        },
        Transform::from_xyz(24.0, 26.0, 24.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

/// Orbit the camera around the world origin.
fn orbit_viewer(
    time: Res<Time>,
    pane: Res<support::VoxelExamplePane>,
    mut viewers: Query<&mut Transform, With<ChunkViewer>>,
) {
    let angle = time.elapsed_secs() * pane.orbit_speed;
    for mut transform in &mut viewers {
        transform.translation = Vec3::new(
            angle.cos() * pane.orbit_radius,
            pane.orbit_height,
            angle.sin() * pane.orbit_radius,
        );
        transform.look_at(Vec3::new(0.0, 6.0, 0.0), Vec3::Y);
    }
}
