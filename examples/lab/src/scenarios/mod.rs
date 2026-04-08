use std::collections::HashSet;

use bevy::{
    ecs::system::SystemState,
    prelude::*,
    render::view::screenshot::{Screenshot, save_to_disk},
    window::PrimaryWindow,
};
use saddle_bevy_e2e::{action::Action, actions::assertions, scenario::Scenario};
use saddle_camera_orbit_camera::OrbitCamera;
use saddle_world_voxel_world_example_support as support;

use crate::{LabOverlay, LabPrimaryViewer, LabSecondaryViewer, LabUiMode, SecondaryViewerEnabled};

pub fn list_scenarios() -> Vec<&'static str> {
    vec![
        "voxel_smoke_launch",
        "voxel_example_basic",
        "voxel_example_debug_gizmos",
        "voxel_example_block_editing",
        "voxel_example_multi_viewer",
        "voxel_example_persistence",
        "voxel_streaming_motion",
        // Backward-compatible aliases for the previous scenario names.
        "voxel_terrain_generation",
        "voxel_block_editing",
        "voxel_multi_viewer",
    ]
}

pub fn scenario_by_name(name: &str) -> Option<Scenario> {
    match name {
        "voxel_smoke_launch" => Some(smoke_launch("voxel_smoke_launch")),
        "voxel_example_basic" => Some(basic_example("voxel_example_basic")),
        "voxel_terrain_generation" => Some(basic_example("voxel_terrain_generation")),
        "voxel_example_debug_gizmos" => Some(debug_gizmos_example("voxel_example_debug_gizmos")),
        "voxel_example_block_editing" => Some(block_editing_example("voxel_example_block_editing")),
        "voxel_block_editing" => Some(block_editing_example("voxel_block_editing")),
        "voxel_example_multi_viewer" => Some(multi_viewer_example("voxel_example_multi_viewer")),
        "voxel_multi_viewer" => Some(multi_viewer_example("voxel_multi_viewer")),
        "voxel_example_persistence" => Some(persistence_example("voxel_example_persistence")),
        "voxel_streaming_motion" => Some(streaming_motion("voxel_streaming_motion")),
        _ => None,
    }
}

fn smoke_launch(name: &'static str) -> Scenario {
    Scenario::builder(name)
        .description(
            "Launch the voxel-world lab, wait for chunk streaming, and capture the baseline showcase composition.",
        )
        .then(Action::WaitFrames(80))
        .then(assertions::resource_exists::<
            saddle_world_voxel_world::VoxelWorldStats,
        >("stats resource present"))
        .then(assertions::entity_count_range::<
            saddle_world_voxel_world::ChunkPos,
        >("reasonable startup chunk count", 1, 900))
        .then(assertions::resource_satisfies::<
            saddle_world_voxel_world::VoxelWorldStats,
        >("startup generated or loaded chunks", |stats| {
            stats.loaded_chunks > 0 || stats.generated_chunks > 0
        }))
        .then(capture_window_screenshot("voxel_smoke_launch"))
        .then(Action::WaitFrames(20))
        .then(assertions::log_summary(name))
        .build()
}

fn basic_example(name: &'static str) -> Scenario {
    Scenario::builder(name)
        .description(
            "Exercise the basic streaming showcase by waiting for terrain, framing two viewpoints, and verifying the overlay and chunk stats stay healthy.",
        )
        .then(set_lab_mode("basic showcase"))
        .then(Action::WaitFrames(100))
        .then(assertions::resource_satisfies::<
            saddle_world_voxel_world::VoxelWorldStats,
        >("basic showcase produced generated and meshed chunks", |stats| {
            stats.generated_chunks > 0 && (stats.meshed_chunks > 0 || stats.remeshed_chunks > 0)
        }))
        .then(Action::Custom(Box::new(|world| {
            let Ok(overlay) = world.query_filtered::<&Text, With<LabOverlay>>().single(world) else {
                panic!("lab overlay should exist for the basic showcase");
            };
            assert!(overlay.0.contains("chunks:"));
        })))
        .then(capture_window_screenshot("basic_near"))
        .then(Action::WaitFrames(20))
        .then(Action::Custom(Box::new(|world| {
            retarget_primary_camera(world, Vec3::new(36.0, 24.0, -12.0), Vec3::new(0.0, 6.0, 0.0));
        })))
        .then(Action::WaitFrames(50))
        .then(capture_window_screenshot("basic_far"))
        .then(Action::WaitFrames(20))
        .then(assertions::log_summary(name))
        .build()
}

fn debug_gizmos_example(name: &'static str) -> Scenario {
    Scenario::builder(name)
        .description(
            "Verify that the lab starts with chunk bounds and viewer radii gizmos enabled, then capture the debug overlay view used to validate the debug_gizmos example.",
        )
        .then(set_lab_mode("debug gizmos"))
        .then(Action::WaitFrames(70))
        .then(assertions::resource_satisfies::<
            saddle_world_voxel_world::VoxelDebugConfig,
        >("debug gizmos are enabled", |debug| {
            debug.show_chunk_bounds && debug.show_viewer_radii
        }))
        .then(assertions::entity_count_range::<
            saddle_world_voxel_world::ChunkPos,
        >("gizmo scene has resident chunks", 1, 900))
        .then(capture_window_screenshot("debug_gizmos"))
        .then(Action::WaitFrames(20))
        .then(assertions::log_summary(name))
        .build()
}

fn streaming_motion(name: &'static str) -> Scenario {
    Scenario::builder(name)
        .description(
            "Move the primary viewer across the world and verify the streamed chunk set changes without exceeding the keep-radius budget.",
        )
        .then(set_lab_mode("streaming motion"))
        .then(Action::WaitFrames(60))
        .then(Action::Custom(Box::new(|world| {
            let snapshot = chunk_set(world);
            world.insert_resource(ChunkSetSnapshot(snapshot));
        })))
        .then(capture_window_screenshot("streaming_start"))
        .then(Action::WaitFrames(20))
        .then(Action::Custom(Box::new(|world| {
            retarget_primary_camera(
                world,
                Vec3::new(72.0, 30.0, 64.0),
                Vec3::new(52.0, 8.0, 44.0),
            );
        })))
        .then(Action::WaitFrames(80))
        .then(assertions::resource_satisfies::<
            saddle_world_voxel_world::VoxelWorldStats,
        >("streaming workload stayed bounded", |stats| {
            stats.loaded_chunks > 0 && stats.pending_generation_jobs <= 4 && stats.pending_meshing_jobs <= 4
        }))
        .then(Action::Custom(Box::new(|world| {
            let before = world.resource::<ChunkSetSnapshot>().0.clone();
            let after = chunk_set(world);
            let new_chunks = after.difference(&before).count();
            let keep_radius =
                world.resource::<saddle_world_voxel_world::VoxelWorldConfig>().keep_radius as usize;
            let max_loaded = (keep_radius * 2 + 1).pow(3) + 128;
            assert!(!after.is_empty());
            assert!(new_chunks > 0);
            assert!(after.len() <= max_loaded);
        })))
        .then(capture_window_screenshot("streaming_end"))
        .then(Action::WaitFrames(20))
        .then(assertions::log_summary(name))
        .build()
}

fn block_editing_example(name: &'static str) -> Scenario {
    Scenario::builder(name)
        .description(
            "Issue edits across a chunk boundary, verify block-modification and remesh counters increase, and capture before/after screenshots for the block_editing example.",
        )
        .then(set_lab_mode("block editing"))
        .then(Action::WaitFrames(60))
        .then(Action::Custom(Box::new(|world| {
            let stats = world.resource::<saddle_world_voxel_world::VoxelWorldStats>();
            let block_modifications = stats.block_modifications;
            let remeshed_chunks = stats.remeshed_chunks;
            let _ = stats;
            world.insert_resource(StatSnapshot {
                block_modifications,
                remeshed_chunks,
            });
        })))
        .then(capture_window_screenshot("editing_before"))
        .then(Action::WaitFrames(20))
        .then(Action::Custom(Box::new(|world| {
            let config = world.resource::<saddle_world_voxel_world::VoxelWorldConfig>().clone();
            let generator = world
                .resource::<saddle_world_voxel_world::VoxelWorldGenerator>()
                .clone();
            let targets = [
                IVec3::new(15, 10, 0),
                IVec3::new(16, 10, 0),
                IVec3::new(15, 11, 0),
                IVec3::new(16, 11, 0),
            ];
            let edits = targets
                .into_iter()
                .map(|world_pos| {
                    let current = saddle_world_voxel_world::sample_generated_block(
                        world_pos,
                        &config,
                        &generator,
                    );
                    let block = if current == saddle_world_voxel_world::BlockId::AIR {
                        support::SHOWCASE_LAMP
                    } else {
                        saddle_world_voxel_world::BlockId::AIR
                    };
                    saddle_world_voxel_world::BlockEdit { world_pos, block }
                })
                .collect();
            world
                .resource_mut::<Messages<saddle_world_voxel_world::VoxelCommand>>()
                .write(saddle_world_voxel_world::VoxelCommand::Batch(edits));
        })))
        .then(Action::WaitFrames(40))
        .then(assertions::custom("editing increments block and remesh counters", |world| {
            let before = *world.resource::<StatSnapshot>();
            let stats = world.resource::<saddle_world_voxel_world::VoxelWorldStats>();
            stats.block_modifications >= before.block_modifications + 4
                && stats.remeshed_chunks > before.remeshed_chunks
        }))
        .then(capture_window_screenshot("editing_after"))
        .then(Action::WaitFrames(20))
        .then(assertions::log_summary(name))
        .build()
}

fn multi_viewer_example(name: &'static str) -> Scenario {
    Scenario::builder(name)
        .description(
            "Enable the secondary viewer, verify that the chunk union actually grows, and capture the expanded streamed field for the multi_viewer example.",
        )
        .then(set_lab_mode("multi viewer"))
        .then(Action::WaitFrames(60))
        .then(Action::Custom(Box::new(|world| {
            let snapshot = chunk_set(world);
            world.insert_resource(ChunkSetSnapshot(snapshot));
        })))
        .then(capture_window_screenshot("multi_viewer_before"))
        .then(Action::WaitFrames(20))
        .then(Action::Custom(Box::new(|world| {
            world.resource_mut::<SecondaryViewerEnabled>().0 = true;
            let Ok(secondary) = world
                .query_filtered::<Entity, With<LabSecondaryViewer>>()
                .single(world)
            else {
                return;
            };
            world
                .entity_mut(secondary)
                .insert(Transform::from_xyz(-72.0, 18.0, -56.0));
        })))
        .then(Action::WaitFrames(80))
        .then(assertions::resource_satisfies::<SecondaryViewerEnabled>(
            "secondary viewer enabled",
            |enabled| enabled.0,
        ))
        .then(Action::Custom(Box::new(|world| {
            let before = world.resource::<ChunkSetSnapshot>().0.clone();
            let after = chunk_set(world);
            assert!(after.len() > before.len());
            assert!(after.difference(&before).next().is_some());
        })))
        .then(capture_window_screenshot("multi_viewer_after"))
        .then(Action::WaitFrames(20))
        .then(assertions::log_summary(name))
        .build()
}

fn persistence_example(name: &'static str) -> Scenario {
    Scenario::builder(name)
        .description(
            "Enable delta-region persistence, stamp an edit, unload the edited chunk, and verify that the edit survives a reload from disk.",
        )
        .then(set_lab_mode("persistence"))
        .then(Action::WaitFrames(60))
        .then(Action::Custom(Box::new(|world| {
            let output_dir = world
                .resource::<saddle_bevy_e2e::capture::CaptureState>()
                .output_dir
                .join("persistence-save");
            let _ = std::fs::remove_dir_all(&output_dir);
            std::fs::create_dir_all(&output_dir)
                .expect("persistence scenario should be able to create its save directory");

            let mut config = world.resource_mut::<saddle_world_voxel_world::VoxelWorldConfig>();
            config.save_policy = saddle_world_voxel_world::SavePolicy {
                mode: saddle_world_voxel_world::SaveMode::DeltaRegions,
                root: output_dir.to_string_lossy().to_string(),
                autosave_interval_seconds: 0.0,
                max_chunks_per_frame: 16,
                ..Default::default()
            };
            let chunk_dims = config.chunk_dims;
            let _ = config;

            let world_pos = IVec3::new(15, 10, 0);
            world.insert_resource(PersistenceScenarioState {
                root: output_dir.to_string_lossy().to_string(),
                world_pos,
                expected_block: support::SHOWCASE_LAMP,
                chunk: saddle_world_voxel_world::world_to_chunk(world_pos, chunk_dims),
            });
        })))
        .then(capture_window_screenshot("persistence_before"))
        .then(Action::WaitFrames(10))
        .then(Action::Custom(Box::new(|world| {
            let state = world.resource::<PersistenceScenarioState>().clone();
            world
                .resource_mut::<Messages<saddle_world_voxel_world::VoxelCommand>>()
                .write(saddle_world_voxel_world::VoxelCommand::SetBlock(
                    saddle_world_voxel_world::BlockEdit {
                        world_pos: state.world_pos,
                        block: state.expected_block,
                    },
                ));
        })))
        .then(Action::WaitFrames(40))
        .then(assertions::custom("delta-region file was written", |world| {
            let state = world.resource::<PersistenceScenarioState>();
            std::fs::read_dir(&state.root)
                .ok()
                .is_some_and(|entries| entries.flatten().any(|entry| {
                    entry
                        .path()
                        .extension()
                        .and_then(|ext| ext.to_str())
                        .is_some_and(|ext| ext == "vwr")
                }))
        }))
        .then(Action::Custom(Box::new(|world| {
            let state = world.resource::<PersistenceScenarioState>().clone();
            assert_eq!(
                sample_loaded_block(world, state.world_pos),
                Some(state.expected_block)
            );
        })))
        .then(capture_window_screenshot("persistence_saved"))
        .then(Action::WaitFrames(10))
        .then(Action::Custom(Box::new(|world| {
            retarget_primary_camera(
                world,
                Vec3::new(192.0, 30.0, 192.0),
                Vec3::new(176.0, 12.0, 176.0),
            );
        })))
        .then(Action::WaitFrames(140))
        .then(Action::Custom(Box::new(|world| {
            let state = world.resource::<PersistenceScenarioState>().clone();
            assert!(
                !world
                .query::<&saddle_world_voxel_world::ChunkPos>()
                .iter(world)
                .any(|chunk| chunk.0 == state.chunk)
            );
        })))
        .then(Action::Custom(Box::new(|world| {
            retarget_primary_camera(world, Vec3::new(28.0, 26.0, 28.0), Vec3::new(0.0, 8.0, 0.0));
        })))
        .then(Action::WaitFrames(140))
        .then(Action::Custom(Box::new(|world| {
            let state = world.resource::<PersistenceScenarioState>().clone();
            assert_eq!(
                sample_loaded_block(world, state.world_pos),
                Some(state.expected_block)
            );
        })))
        .then(capture_window_screenshot("persistence_reloaded"))
        .then(Action::WaitFrames(20))
        .then(assertions::log_summary(name))
        .build()
}

#[derive(Resource, Clone, Copy)]
struct StatSnapshot {
    block_modifications: u64,
    remeshed_chunks: u64,
}

#[derive(Resource, Clone)]
struct ChunkSetSnapshot(HashSet<IVec3>);

#[derive(Resource, Clone)]
struct PersistenceScenarioState {
    root: String,
    world_pos: IVec3,
    expected_block: saddle_world_voxel_world::BlockId,
    chunk: IVec3,
}

fn set_lab_mode(label: &'static str) -> Action {
    Action::Custom(Box::new(move |world| {
        world.resource_mut::<LabUiMode>().0 = label.into();
    }))
}

fn chunk_set(world: &mut World) -> HashSet<IVec3> {
    world
        .query::<&saddle_world_voxel_world::ChunkPos>()
        .iter(world)
        .map(|pos| pos.0)
        .collect()
}

fn sample_loaded_block(
    world: &mut World,
    world_pos: IVec3,
) -> Option<saddle_world_voxel_world::BlockId> {
    let mut system_state = SystemState::<saddle_world_voxel_world::VoxelWorldView>::new(world);
    let view = system_state.get(world);
    view.sample_loaded_block(world_pos)
}

fn retarget_primary_camera(world: &mut World, eye: Vec3, focus: Vec3) {
    let Ok(viewer) = world
        .query_filtered::<Entity, With<LabPrimaryViewer>>()
        .single(world)
    else {
        return;
    };
    let (yaw, pitch, distance) = saddle_camera_orbit_camera::orbit_state_from_eye(focus, eye);
    let mut entity = world.entity_mut(viewer);
    let Some(mut camera) = entity.get_mut::<OrbitCamera>() else {
        return;
    };
    camera.focus_on(focus);
    camera.set_target_angles(yaw, pitch);
    camera.set_target_distance(distance);
}

fn capture_window_screenshot(name: &str) -> Action {
    let name = name.to_string();
    Action::Custom(Box::new(move |world| {
        let frame = world
            .resource::<saddle_bevy_e2e::runner::ScenarioRunner>()
            .total_frames;
        let output_dir = world
            .resource::<saddle_bevy_e2e::capture::CaptureState>()
            .output_dir
            .clone();
        std::fs::create_dir_all(&output_dir)
            .expect("lab E2E should be able to create the screenshot output directory");
        let window = world
            .query_filtered::<Entity, With<PrimaryWindow>>()
            .single(world)
            .expect("primary window should exist during lab E2E");
        world
            .resource_mut::<saddle_bevy_e2e::capture::CaptureState>()
            .log(format!("[frame {frame}] Screenshot({name:?})"));

        let mut commands = world.commands();
        commands
            .spawn(Screenshot::window(window))
            .observe(save_to_disk(output_dir.join(format!("{name}.png"))));
    }))
}
