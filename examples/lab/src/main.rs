#[cfg(feature = "e2e")]
mod e2e;
#[cfg(feature = "e2e")]
mod scenarios;

use saddle_world_voxel_world_example_support as support;

use bevy::{
    input::ButtonInput,
    prelude::*,
    remote::{RemotePlugin, http::RemoteHttpPlugin},
};
use saddle_pane::prelude::*;
use saddle_camera_orbit_camera::{OrbitCamera, OrbitCameraInputTarget, OrbitCameraPlugin};
use saddle_world_voxel_world::{
    ChunkViewer, ChunkViewerSettings, VoxelDebugConfig, VoxelWorldConfig, VoxelWorldPlugin,
};

#[derive(Component)]
pub(crate) struct LabPrimaryViewer;

#[derive(Component)]
pub(crate) struct LabSecondaryViewer;

#[derive(Component)]
pub(crate) struct LabOverlay;

#[derive(Resource, Default)]
pub(crate) struct SecondaryViewerEnabled(pub bool);

#[derive(Resource, Default)]
pub(crate) struct LabUiMode(pub String);

fn main() {
    let mut config = VoxelWorldConfig::default();
    config.request_radius = 4;
    config.keep_radius = 6;
    config.max_chunk_requests_per_frame = 18;

    let mut app = App::new();
    app.insert_resource(ClearColor(Color::srgb(0.60, 0.76, 0.92)));
    app.insert_resource(config);
    app.insert_resource(support::VoxelExamplePane {
        show_chunk_bounds: true,
        show_viewer_radii: true,
        ..default()
    });
    app.insert_resource(SecondaryViewerEnabled::default());
    app.insert_resource(LabUiMode("idle".into()));
    app.insert_resource(VoxelDebugConfig {
        show_chunk_bounds: true,
        show_viewer_radii: true,
        ..default()
    });
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "Voxel World Lab".into(),
            resolution: (1400, 900).into(),
            ..default()
        }),
        ..default()
    }));
    app.add_plugins(support::pane_plugins());
    app.add_plugins((
        RemotePlugin::default(),
        OrbitCameraPlugin::default(),
        VoxelWorldPlugin::default(),
    ));
    app.register_pane::<support::VoxelExamplePane>();
    #[cfg(all(feature = "dev", not(target_arch = "wasm32")))]
    app.add_plugins(bevy_brp_extras::BrpExtrasPlugin::with_http_plugin(
        RemoteHttpPlugin::default(),
    ));
    #[cfg(feature = "e2e")]
    app.add_plugins(e2e::E2EPlugin);
    app.add_systems(Startup, setup);
    app.add_systems(
        Update,
        (
            support::sync_example_pane,
            handle_debug_keys,
            animate_secondary_viewer,
            update_overlay,
        ),
    );
    app.run();
}

fn setup(mut commands: Commands) {
    commands.spawn((
        Name::new("Sun"),
        DirectionalLight {
            illuminance: 10_000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.9, 0.72, 0.0)),
    ));

    commands.insert_resource(GlobalAmbientLight {
        brightness: 140.0,
        ..default()
    });

    commands.spawn((
        Name::new("Primary Viewer Camera"),
        OrbitCamera::looking_at(Vec3::new(0.0, 8.0, 0.0), Vec3::new(28.0, 26.0, 28.0)),
        OrbitCameraInputTarget,
        ChunkViewer,
        ChunkViewerSettings {
            request_radius: 4,
            keep_radius: 6,
            priority: 8,
        },
        LabPrimaryViewer,
    ));

    commands.spawn((
        Name::new("Secondary Viewer"),
        ChunkViewer,
        ChunkViewerSettings {
            request_radius: 3,
            keep_radius: 5,
            priority: 2,
        },
        LabSecondaryViewer,
        Transform::from_xyz(-20.0, 16.0, -16.0),
        GlobalTransform::default(),
    ));

    commands.spawn((
        Name::new("Overlay"),
        LabOverlay,
        Node {
            position_type: PositionType::Absolute,
            top: px(16.0),
            left: px(16.0),
            width: px(420.0),
            padding: UiRect::all(px(12.0)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.04, 0.06, 0.10, 0.78)),
        Text::default(),
        TextFont {
            font_size: 15.0,
            ..default()
        },
        TextColor(Color::WHITE),
    ));
}

fn handle_debug_keys(
    keys: Res<ButtonInput<KeyCode>>,
    mut mode: ResMut<LabUiMode>,
    mut secondary_enabled: ResMut<SecondaryViewerEnabled>,
    mut cameras: Query<&mut OrbitCamera, With<LabPrimaryViewer>>,
    mut commands: MessageWriter<saddle_world_voxel_world::VoxelCommand>,
) {
    if keys.just_pressed(KeyCode::Space) {
        secondary_enabled.0 = !secondary_enabled.0;
        mode.0 = if secondary_enabled.0 {
            "secondary viewer enabled".into()
        } else {
            "secondary viewer disabled".into()
        };
    }

    if keys.just_pressed(KeyCode::KeyR) {
        for mut camera in &mut cameras {
            camera.reset_to_home();
        }
        mode.0 = "camera reset".into();
    }

    if keys.just_pressed(KeyCode::KeyE) {
        commands.write(saddle_world_voxel_world::VoxelCommand::Batch(vec![
            saddle_world_voxel_world::BlockEdit {
                world_pos: IVec3::new(15, 10, 0),
                block: saddle_world_voxel_world::BlockId::AIR,
            },
            saddle_world_voxel_world::BlockEdit {
                world_pos: IVec3::new(16, 10, 0),
                block: saddle_world_voxel_world::BlockId::LAMP,
            },
        ]));
        mode.0 = "manual edit burst".into();
    }
}

fn animate_secondary_viewer(
    time: Res<Time>,
    enabled: Res<SecondaryViewerEnabled>,
    mut viewers: Query<&mut Transform, With<LabSecondaryViewer>>,
) {
    if !enabled.0 {
        return;
    }
    for mut transform in &mut viewers {
        transform.translation = Vec3::new(
            -48.0 + 18.0 * (time.elapsed_secs() * 0.31).cos(),
            18.0,
            -52.0 + 18.0 * (time.elapsed_secs() * 0.21).sin(),
        );
    }
}

fn update_overlay(
    mode: Res<LabUiMode>,
    stats: Res<saddle_world_voxel_world::VoxelWorldStats>,
    chunks: Query<&saddle_world_voxel_world::ChunkPos>,
    mut overlay: Query<&mut Text, With<LabOverlay>>,
) {
    let Ok(mut overlay) = overlay.single_mut() else {
        return;
    };
    let count = chunks.iter().count();
    overlay.0 = format!(
        "Voxel World Lab\nmode: {}\ncontrols: LMB orbit | MMB pan | wheel zoom | Space secondary viewer | E edit burst | R reset\nchunks: {}\nloaded={} meshed={} dirty={} unloaded={}\ngen_jobs={} mesh_jobs={} remeshes={}\nblock_modifications={}",
        mode.0,
        count,
        stats.loaded_chunks,
        stats.meshed_chunks,
        stats.dirty_chunks,
        stats.unloaded_chunks,
        stats.pending_generation_jobs,
        stats.pending_meshing_jobs,
        stats.remeshed_chunks,
        stats.block_modifications,
    );
}
