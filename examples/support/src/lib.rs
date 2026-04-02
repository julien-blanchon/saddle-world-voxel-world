use bevy::prelude::*;
use saddle_world_voxel_world::{ChunkViewer, ChunkViewerSettings, SaveMode, SavePolicy, VoxelDebugConfig, VoxelWorldConfig};

#[derive(Component)]
pub struct ExampleViewer;

#[derive(Component)]
pub struct SecondaryViewer;

pub fn default_config() -> VoxelWorldConfig {
    VoxelWorldConfig {
        request_radius: 4,
        keep_radius: 6,
        max_chunk_requests_per_frame: 16,
        save_policy: SavePolicy {
            mode: SaveMode::Disabled,
            ..SavePolicy::default()
        },
        ..VoxelWorldConfig::default()
    }
}

pub fn spawn_scene(mut commands: Commands) {
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

    commands.spawn((
        Name::new("Main Camera"),
        Camera3d::default(),
        ChunkViewer,
        ChunkViewerSettings {
            request_radius: 4,
            keep_radius: 6,
            priority: 10,
        },
        ExampleViewer,
        Transform::from_xyz(24.0, 26.0, 24.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

#[allow(dead_code)]
pub fn enable_debug(mut debug: ResMut<VoxelDebugConfig>) {
    debug.show_chunk_bounds = true;
    debug.show_viewer_radii = true;
}

pub fn spin_viewer(
    time: Res<Time>,
    mut viewers: Query<&mut Transform, (With<ExampleViewer>, Without<SecondaryViewer>)>,
) {
    let angle = time.elapsed_secs() * 0.18;
    for mut transform in &mut viewers {
        transform.translation = Vec3::new(angle.cos() * 40.0, 28.0, angle.sin() * 40.0);
        transform.look_at(Vec3::new(0.0, 6.0, 0.0), Vec3::Y);
    }
}
