//! Mini-Minecraft — a playable voxel sandbox demo.
//!
//! Demonstrates the full voxel-world pipeline in an interactive first-person
//! game: FPS camera, block placement/breaking, the optional showcase terrain
//! preset from example support, dynamic chunk streaming, and a simple HUD.
//!
//! # Controls
//! - **WASD** — Move
//! - **Space** — Jump / fly up
//! - **Shift** — Fly down
//! - **Mouse** — Look around
//! - **Left click** — Break block
//! - **Right click** — Place block
//! - **1-6** — Select block type (Grass, Dirt, Stone, Sand, Wood, Leaves)
//! - **Escape** — Toggle mouse capture
//! - **F3** — Toggle debug overlay

use bevy::{
    input::mouse::AccumulatedMouseMotion,
    prelude::*,
    window::{CursorGrabMode, CursorOptions},
};
use saddle_pane::prelude::*;
use saddle_world_voxel_world::{
    BlockEdit, BlockId, BlockRegistry, ChunkViewer, ChunkViewerSettings, SaveMode, SavePolicy,
    VoxelCommand, VoxelWorldConfig, VoxelWorldPlugin, VoxelWorldStats, raycast_blocks,
};
use saddle_world_voxel_world_example_support as support;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::srgb(0.53, 0.74, 0.93)))
        .insert_resource(support::showcase_registry())
        .insert_resource(support::showcase_generator())
        .insert_resource(VoxelWorldConfig {
            request_radius: 8,
            keep_radius: 10,
            max_chunk_requests_per_frame: 24,
            max_generation_jobs_in_flight: 8,
            max_mesh_jobs_in_flight: 8,
            seed: 42,
            save_policy: SavePolicy {
                mode: SaveMode::Disabled,
                ..SavePolicy::default()
            },
            ..VoxelWorldConfig::default()
        })
        .insert_resource(support::VoxelExamplePane::default())
        .insert_resource(PlayerState::default())
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Mini Minecraft — Voxel World Demo".into(),
                resolution: (1280, 840).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(support::pane_plugins())
        .register_pane::<support::VoxelExamplePane>()
        .add_plugins(VoxelWorldPlugin::default())
        .add_systems(Startup, setup_scene)
        .add_systems(
            Update,
            (
                support::sync_example_pane,
                toggle_cursor_grab,
                fps_camera_look,
                fps_camera_move,
                block_select,
                block_interact,
            ),
        )
        .add_systems(
            Update,
            (update_crosshair, update_hud_text, toggle_debug_overlay),
        )
        .run();
}

#[derive(Component)]
struct FpsPlayer;

#[derive(Component)]
struct CrosshairUi;

#[derive(Component)]
struct HudText;

#[derive(Component)]
struct DebugOverlay;

#[derive(Resource)]
struct PlayerState {
    yaw: f32,
    pitch: f32,
    selected_block: BlockId,
    selected_index: usize,
    cursor_grabbed: bool,
    show_debug: bool,
}

impl Default for PlayerState {
    fn default() -> Self {
        Self {
            yaw: 0.0,
            pitch: 0.0,
            selected_block: support::SHOWCASE_STONE,
            selected_index: 2,
            cursor_grabbed: true,
            show_debug: false,
        }
    }
}

fn setup_scene(mut commands: Commands, mut cursor_opts: Query<&mut CursorOptions, With<Window>>) {
    // Grab cursor on startup
    if let Ok(mut opts) = cursor_opts.single_mut() {
        opts.grab_mode = CursorGrabMode::Locked;
        opts.visible = false;
    }

    // Sun
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
        brightness: 200.0,
        ..default()
    });

    // FPS Player (camera + chunk viewer)
    commands.spawn((
        Name::new("FPS Player"),
        Camera3d::default(),
        FpsPlayer,
        ChunkViewer,
        ChunkViewerSettings {
            request_radius: 8,
            keep_radius: 10,
            priority: 10,
        },
        Transform::from_xyz(8.0, 40.0, 8.0).looking_to(Vec3::NEG_Z, Vec3::Y),
    ));

    // Crosshair
    commands
        .spawn((
            Name::new("Crosshair"),
            CrosshairUi,
            Node {
                position_type: PositionType::Absolute,
                left: Val::Percent(50.0),
                top: Val::Percent(50.0),
                width: Val::Px(2.0),
                height: Val::Px(2.0),
                margin: UiRect {
                    left: Val::Px(-1.0),
                    top: Val::Px(-1.0),
                    ..default()
                },
                ..default()
            },
            BackgroundColor(Color::WHITE),
        ))
        .with_children(|parent| {
            // Horizontal bar
            parent.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    width: Val::Px(20.0),
                    height: Val::Px(2.0),
                    left: Val::Px(-9.0),
                    top: Val::Px(0.0),
                    ..default()
                },
                BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.8)),
            ));
            // Vertical bar
            parent.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    width: Val::Px(2.0),
                    height: Val::Px(20.0),
                    left: Val::Px(0.0),
                    top: Val::Px(-9.0),
                    ..default()
                },
                BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.8)),
            ));
        });

    // HUD text (controls + selected block)
    commands.spawn((
        Name::new("HUD"),
        HudText,
        Text::new(""),
        TextFont {
            font_size: 16.0,
            ..default()
        },
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(12.0),
            bottom: Val::Px(12.0),
            ..default()
        },
    ));

    // Debug overlay (hidden by default)
    commands.spawn((
        Name::new("Debug Overlay"),
        DebugOverlay,
        Text::new(""),
        TextFont {
            font_size: 14.0,
            ..default()
        },
        TextColor(Color::srgba(1.0, 1.0, 0.6, 0.9)),
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(12.0),
            top: Val::Px(12.0),
            ..default()
        },
        Visibility::Hidden,
    ));
}

fn toggle_cursor_grab(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut cursor_opts: Query<&mut CursorOptions, With<Window>>,
    mut state: ResMut<PlayerState>,
) {
    if keyboard.just_pressed(KeyCode::Escape) {
        state.cursor_grabbed = !state.cursor_grabbed;
        if let Ok(mut opts) = cursor_opts.single_mut() {
            if state.cursor_grabbed {
                opts.grab_mode = CursorGrabMode::Locked;
                opts.visible = false;
            } else {
                opts.grab_mode = CursorGrabMode::None;
                opts.visible = true;
            }
        }
    }
}

fn fps_camera_look(
    mouse: Res<AccumulatedMouseMotion>,
    mut state: ResMut<PlayerState>,
    mut cameras: Query<&mut Transform, With<FpsPlayer>>,
) {
    if !state.cursor_grabbed {
        return;
    }
    let sensitivity = 0.003;
    let delta = mouse.delta;
    if delta == Vec2::ZERO {
        return;
    }

    state.yaw -= delta.x * sensitivity;
    state.pitch = (state.pitch - delta.y * sensitivity).clamp(-1.5, 1.5);

    for mut transform in &mut cameras {
        transform.rotation = Quat::from_euler(EulerRot::YXZ, state.yaw, state.pitch, 0.0);
    }
}

fn fps_camera_move(
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    state: Res<PlayerState>,
    mut cameras: Query<&mut Transform, With<FpsPlayer>>,
) {
    let speed = 16.0;
    let mut direction = Vec3::ZERO;

    if keyboard.pressed(KeyCode::KeyW) {
        direction += Vec3::NEG_Z;
    }
    if keyboard.pressed(KeyCode::KeyS) {
        direction += Vec3::Z;
    }
    if keyboard.pressed(KeyCode::KeyA) {
        direction += Vec3::NEG_X;
    }
    if keyboard.pressed(KeyCode::KeyD) {
        direction += Vec3::X;
    }
    if keyboard.pressed(KeyCode::Space) {
        direction += Vec3::Y;
    }
    if keyboard.pressed(KeyCode::ShiftLeft) {
        direction += Vec3::NEG_Y;
    }

    if direction == Vec3::ZERO {
        return;
    }

    let yaw_rot = Quat::from_rotation_y(state.yaw);
    let forward = yaw_rot * Vec3::NEG_Z;
    let right = yaw_rot * Vec3::X;

    let movement = (forward * direction.z + right * direction.x + Vec3::Y * direction.y)
        .normalize_or_zero()
        * speed
        * time.delta_secs();

    for mut transform in &mut cameras {
        transform.translation += movement;
    }
}

fn block_select(keyboard: Res<ButtonInput<KeyCode>>, mut state: ResMut<PlayerState>) {
    let keys = [
        KeyCode::Digit1,
        KeyCode::Digit2,
        KeyCode::Digit3,
        KeyCode::Digit4,
        KeyCode::Digit5,
        KeyCode::Digit6,
    ];
    for (i, key) in keys.iter().enumerate() {
        if keyboard.just_pressed(*key) {
            state.selected_index = i;
            state.selected_block = support::SHOWCASE_PLACEABLE_BLOCKS[i].0;
        }
    }
}

/// Block sampler that reads from loaded chunk data via VoxelWorldView.
struct ViewSampler<'a> {
    view: &'a saddle_world_voxel_world::VoxelWorldView<'a>,
}

impl saddle_world_voxel_world::BlockSampler for ViewSampler<'_> {
    fn sample_block(&self, world_pos: IVec3) -> Option<BlockId> {
        self.view.sample_loaded_block(world_pos)
    }
}

fn block_interact(
    mouse: Res<ButtonInput<MouseButton>>,
    state: Res<PlayerState>,
    registry: Res<BlockRegistry>,
    view: saddle_world_voxel_world::VoxelWorldView,
    cameras: Query<&Transform, With<FpsPlayer>>,
    mut writer: MessageWriter<VoxelCommand>,
) {
    if !state.cursor_grabbed {
        return;
    }
    let left = mouse.just_pressed(MouseButton::Left);
    let right = mouse.just_pressed(MouseButton::Right);
    if !left && !right {
        return;
    }

    let Ok(transform) = cameras.single() else {
        return;
    };

    let origin = transform.translation;
    let direction = transform.forward().as_vec3();
    let sampler = ViewSampler { view: &view };
    let hit = raycast_blocks(&sampler, &registry, origin, direction, 8.0);

    if let Some(hit) = hit {
        if left {
            // Break block
            writer.write(VoxelCommand::SetBlock(BlockEdit {
                world_pos: hit.world_pos,
                block: BlockId::AIR,
            }));
        } else if right {
            // Place block adjacent to hit face
            let place_pos = hit.world_pos + hit.normal;
            writer.write(VoxelCommand::SetBlock(BlockEdit {
                world_pos: place_pos,
                block: state.selected_block,
            }));
        }
    }
}

fn update_crosshair(
    state: Res<PlayerState>,
    mut crosshairs: Query<&mut BackgroundColor, With<CrosshairUi>>,
) {
    for mut bg in &mut crosshairs {
        bg.0 = if state.cursor_grabbed {
            Color::WHITE
        } else {
            Color::srgba(1.0, 1.0, 1.0, 0.3)
        };
    }
}

fn update_hud_text(state: Res<PlayerState>, mut texts: Query<&mut Text, With<HudText>>) {
    let block_name = support::SHOWCASE_PLACEABLE_BLOCKS[state.selected_index].1;
    let mut bar = String::new();
    for (i, (_, name)) in support::SHOWCASE_PLACEABLE_BLOCKS.iter().enumerate() {
        if i == state.selected_index {
            bar.push_str(&format!("[{}: {}] ", i + 1, name));
        } else {
            bar.push_str(&format!(" {}: {}  ", i + 1, name));
        }
    }
    for mut text in &mut texts {
        **text = format!(
            "{bar}\n\
             WASD: Move | Mouse: Look | LMB: Break | RMB: Place {block_name}\n\
             Space/Shift: Up/Down | Esc: Release cursor | F3: Debug"
        );
    }
}

fn toggle_debug_overlay(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut state: ResMut<PlayerState>,
    stats: Res<VoxelWorldStats>,
    cameras: Query<&Transform, With<FpsPlayer>>,
    mut overlays: Query<(&mut Text, &mut Visibility), With<DebugOverlay>>,
) {
    if keyboard.just_pressed(KeyCode::F3) {
        state.show_debug = !state.show_debug;
    }

    for (mut text, mut vis) in &mut overlays {
        *vis = if state.show_debug {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };

        if state.show_debug {
            let pos = cameras
                .single()
                .map(|t| t.translation)
                .unwrap_or(Vec3::ZERO);
            **text = format!(
                "Pos: ({:.1}, {:.1}, {:.1})\n\
                 Chunks: {} loaded, {} meshed, {} dirty\n\
                 Gen jobs: {} | Mesh jobs: {}\n\
                 Last gen: {:.1}ms | Last mesh: {:.1}ms",
                pos.x,
                pos.y,
                pos.z,
                stats.loaded_chunks,
                stats.meshed_chunks,
                stats.dirty_chunks,
                stats.pending_generation_jobs,
                stats.pending_meshing_jobs,
                stats.last_generation_time_ms,
                stats.last_meshing_time_ms,
            );
        }
    }
}
