use bevy::prelude::*;
use saddle_pane::prelude::*;
use saddle_world_voxel_world::{
    ChunkViewer, ChunkViewerSettings, SaveMode, SavePolicy, VoxelDebugConfig, VoxelWorldConfig,
};

#[derive(Component)]
pub struct ExampleViewer;

#[derive(Component)]
pub struct SecondaryViewer;

#[derive(Resource, Pane)]
#[pane(title = "Voxel World")]
pub struct VoxelExamplePane {
    #[pane(slider, min = 2.0, max = 10.0, step = 1.0)]
    pub request_radius: f32,
    #[pane(slider, min = 3.0, max = 12.0, step = 1.0)]
    pub keep_radius: f32,
    #[pane(toggle)]
    pub show_chunk_bounds: bool,
    #[pane(toggle)]
    pub show_viewer_radii: bool,
    #[pane(slider, min = 18.0, max = 72.0, step = 1.0)]
    pub orbit_radius: f32,
    #[pane(slider, min = 12.0, max = 42.0, step = 0.5)]
    pub orbit_height: f32,
    #[pane(slider, min = 0.04, max = 0.45, step = 0.01)]
    pub orbit_speed: f32,
}

impl Default for VoxelExamplePane {
    fn default() -> Self {
        Self {
            request_radius: 4.0,
            keep_radius: 6.0,
            show_chunk_bounds: false,
            show_viewer_radii: false,
            orbit_radius: 40.0,
            orbit_height: 28.0,
            orbit_speed: 0.18,
        }
    }
}

pub fn pane_plugins() -> (
    bevy_flair::FlairPlugin,
    bevy_input_focus::InputDispatchPlugin,
    bevy_ui_widgets::UiWidgetsPlugins,
    bevy_input_focus::tab_navigation::TabNavigationPlugin,
    saddle_pane::PanePlugin,
) {
    (
        bevy_flair::FlairPlugin,
        bevy_input_focus::InputDispatchPlugin,
        bevy_ui_widgets::UiWidgetsPlugins,
        bevy_input_focus::tab_navigation::TabNavigationPlugin,
        saddle_pane::PanePlugin,
    )
}

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

pub fn sync_example_pane(
    pane: Res<VoxelExamplePane>,
    mut config: ResMut<VoxelWorldConfig>,
    mut debug: ResMut<VoxelDebugConfig>,
) {
    config.request_radius = pane.request_radius.round().max(1.0) as u32;
    config.keep_radius = pane.keep_radius.round().max(config.request_radius as f32) as u32;
    debug.show_chunk_bounds = pane.show_chunk_bounds;
    debug.show_viewer_radii = pane.show_viewer_radii;
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
    pane: Res<VoxelExamplePane>,
    mut viewers: Query<&mut Transform, (With<ExampleViewer>, Without<SecondaryViewer>)>,
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
