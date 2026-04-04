//! Debug gizmos example — chunk bounds and viewer radii visualization.
//!
//! Same setup as basic, but with `VoxelDebugConfig` fields enabled to draw
//! chunk bounding boxes and viewer streaming radii as gizmos. Toggle these
//! via the saddle-pane UI.

use saddle_world_voxel_world_example_support as support;

use bevy::prelude::*;
use saddle_pane::prelude::*;
use saddle_world_voxel_world::{
    ChunkViewer, ChunkViewerSettings, SaveMode, SavePolicy, VoxelDebugConfig, VoxelWorldConfig,
    VoxelWorldPlugin,
};

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::srgb(0.58, 0.74, 0.90)))
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

    // Camera with ChunkViewer
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
