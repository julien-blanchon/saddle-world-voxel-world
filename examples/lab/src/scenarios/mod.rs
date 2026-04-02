use std::collections::HashSet;

use bevy::prelude::*;
use saddle_bevy_e2e::{action::Action, actions::assertions, scenario::Scenario};
use saddle_camera_orbit_camera::OrbitCamera;

use crate::{LabPrimaryViewer, LabSecondaryViewer, LabUiMode, SecondaryViewerEnabled};

pub fn list_scenarios() -> Vec<&'static str> {
    vec![
        "voxel_smoke_launch",
        "voxel_terrain_generation",
        "voxel_streaming_motion",
        "voxel_block_editing",
        "voxel_multi_viewer",
    ]
}

pub fn scenario_by_name(name: &str) -> Option<Scenario> {
    match name {
        "voxel_smoke_launch" => Some(voxel_smoke_launch()),
        "voxel_terrain_generation" => Some(voxel_terrain_generation()),
        "voxel_streaming_motion" => Some(voxel_streaming_motion()),
        "voxel_block_editing" => Some(voxel_block_editing()),
        "voxel_multi_viewer" => Some(voxel_multi_viewer()),
        _ => None,
    }
}

fn voxel_smoke_launch() -> Scenario {
    Scenario::builder("voxel_smoke_launch")
        .description(
            "Launch the voxel-world lab, wait for chunk streaming, and capture the baseline view.",
        )
        .then(Action::WaitFrames(80))
        .then(assertions::resource_exists::<saddle_world_voxel_world::VoxelWorldStats>(
            "stats resource present",
        ))
        .then(assertions::entity_count_range::<saddle_world_voxel_world::ChunkPos>(
            "reasonable startup chunk count",
            1,
            900,
        ))
        .then(Action::Custom(Box::new(|world| {
            let chunk_count = world.query::<&saddle_world_voxel_world::ChunkPos>().iter(world).count();
            let stats = world.resource::<saddle_world_voxel_world::VoxelWorldStats>();
            assert!(chunk_count > 0);
            assert!(stats.loaded_chunks > 0 || stats.generated_chunks > 0);
        })))
        .then(Action::Screenshot("voxel_smoke_launch".into()))
        .then(assertions::log_summary("voxel_smoke_launch"))
        .build()
}

fn voxel_terrain_generation() -> Scenario {
    Scenario::builder("voxel_terrain_generation")
        .description("Let the terrain settle, capture two visual checkpoints, and assert the streamed set is healthy.")
        .then(Action::WaitFrames(100))
        .then(assertions::resource_satisfies::<saddle_world_voxel_world::VoxelWorldStats>(
            "terrain generation produced chunks",
            |stats| stats.generated_chunks > 0 && (stats.meshed_chunks > 0 || stats.remeshed_chunks > 0),
        ))
        .then(Action::Custom(Box::new(|world| {
            let stats = world.resource::<saddle_world_voxel_world::VoxelWorldStats>();
            assert!(stats.generated_chunks > 0);
            assert!(stats.meshed_chunks > 0 || stats.remeshed_chunks > 0);
            world.resource_mut::<LabUiMode>().0 = "terrain".into();
        })))
        .then(Action::Screenshot("terrain_near".into()))
        .then(Action::Custom(Box::new(|world| {
            retarget_primary_camera(world, Vec3::new(36.0, 24.0, -12.0), Vec3::new(0.0, 6.0, 0.0));
        })))
        .then(Action::WaitFrames(50))
        .then(Action::Custom(Box::new(|world| {
            let overlay = world.query::<&Text>().iter(world).next().unwrap();
            assert!(overlay.0.contains("chunks"));
        })))
        .then(Action::Screenshot("terrain_far".into()))
        .then(assertions::log_summary("voxel_terrain_generation"))
        .build()
}

fn voxel_streaming_motion() -> Scenario {
    Scenario::builder("voxel_streaming_motion")
        .description("Move the primary viewer across the world and verify the loaded set changes without exploding in size.")
        .then(Action::WaitFrames(60))
        .then(Action::Custom(Box::new(|world| {
            let snapshot = world
                .query::<&saddle_world_voxel_world::ChunkPos>()
                .iter(world)
                .map(|pos| pos.0)
                .collect::<HashSet<_>>();
            world.insert_resource(ChunkSetSnapshot(snapshot));
        })))
        .then(Action::Screenshot("streaming_start".into()))
        .then(Action::Custom(Box::new(|world| {
            retarget_primary_camera(world, Vec3::new(72.0, 30.0, 64.0), Vec3::new(52.0, 8.0, 44.0));
        })))
        .then(Action::WaitFrames(80))
        .then(assertions::resource_satisfies::<saddle_world_voxel_world::VoxelWorldStats>(
            "streaming workload stayed bounded",
            |stats| stats.loaded_chunks > 0 && stats.pending_generation_jobs <= 4 && stats.pending_meshing_jobs <= 4,
        ))
        .then(Action::Custom(Box::new(|world| {
            let before = world.resource::<ChunkSetSnapshot>().0.clone();
            let after = world
                .query::<&saddle_world_voxel_world::ChunkPos>()
                .iter(world)
                .map(|pos| pos.0)
                .collect::<HashSet<_>>();
            let new_chunks = after.difference(&before).count();
            let keep_radius = world.resource::<saddle_world_voxel_world::VoxelWorldConfig>().keep_radius as usize;
            let max_loaded = (keep_radius * 2 + 1).pow(3) + 128;
            assert!(!after.is_empty());
            assert!(new_chunks > 0);
            assert!(after.len() <= max_loaded);
        })))
        .then(Action::Screenshot("streaming_end".into()))
        .then(assertions::log_summary("voxel_streaming_motion"))
        .build()
}

fn voxel_block_editing() -> Scenario {
    Scenario::builder("voxel_block_editing")
        .description("Issue edits near a chunk border, wait for remeshing, and verify the stats reflect the edit workload.")
        .then(Action::WaitFrames(60))
        .then(Action::Screenshot("editing_before".into()))
        .then(Action::Custom(Box::new(|world| {
            let config = world.resource::<saddle_world_voxel_world::VoxelWorldConfig>().clone();
            let targets = [IVec3::new(15, 10, 0), IVec3::new(16, 10, 0)];
            let edits = targets
                .into_iter()
                .map(|world_pos| {
                    let current = saddle_world_voxel_world::sample_generated_block(world_pos, &config);
                    let block = if current == saddle_world_voxel_world::BlockId::AIR {
                        saddle_world_voxel_world::BlockId::LAMP
                    } else {
                        saddle_world_voxel_world::BlockId::AIR
                    };
                    saddle_world_voxel_world::BlockEdit { world_pos, block }
                })
                .collect();
            world
                .resource_mut::<Messages<saddle_world_voxel_world::VoxelCommand>>()
                .write(saddle_world_voxel_world::VoxelCommand::Batch(edits));
            world.resource_mut::<LabUiMode>().0 = "editing".into();
        })))
        .then(Action::WaitFrames(40))
        .then(assertions::resource_satisfies::<saddle_world_voxel_world::VoxelWorldStats>(
            "editing updated stats",
            |stats| stats.block_modifications >= 2 && stats.remeshed_chunks > 0,
        ))
        .then(Action::Custom(Box::new(|world| {
            let stats = world.resource::<saddle_world_voxel_world::VoxelWorldStats>();
            assert!(stats.block_modifications >= 2);
            assert!(stats.remeshed_chunks > 0);
        })))
        .then(Action::Screenshot("editing_after".into()))
        .then(assertions::log_summary("voxel_block_editing"))
        .build()
}

fn voxel_multi_viewer() -> Scenario {
    Scenario::builder("voxel_multi_viewer")
        .description("Enable the secondary viewer, verify the chunk union grows, and capture the expanded streamed field.")
        .then(Action::WaitFrames(60))
        .then(Action::Custom(Box::new(|world| {
            let count = world.query::<&saddle_world_voxel_world::ChunkPos>().iter(world).count();
            world.insert_resource(ChunkCountSnapshot(count));
            world.resource_mut::<LabUiMode>().0 = "single viewer".into();
        })))
        .then(Action::Screenshot("multi_viewer_before".into()))
        .then(Action::Custom(Box::new(|world| {
            world.resource_mut::<SecondaryViewerEnabled>().0 = true;
            let secondary = world
                .query_filtered::<Entity, With<LabSecondaryViewer>>()
                .single(world)
                .unwrap();
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
            let before = world.resource::<ChunkCountSnapshot>().0;
            let after = world.query::<&saddle_world_voxel_world::ChunkPos>().iter(world).count();
            assert!(after >= before);
        })))
        .then(Action::Screenshot("multi_viewer_after".into()))
        .then(assertions::log_summary("voxel_multi_viewer"))
        .build()
}

#[derive(Resource)]
struct ChunkCountSnapshot(usize);

#[derive(Resource)]
struct ChunkSetSnapshot(HashSet<IVec3>);

fn retarget_primary_camera(world: &mut World, eye: Vec3, focus: Vec3) {
    let viewer = world
        .query_filtered::<Entity, With<LabPrimaryViewer>>()
        .single(world)
        .unwrap();
    let (yaw, pitch, distance) = saddle_camera_orbit_camera::orbit_state_from_eye(focus, eye);
    let mut entity = world.entity_mut(viewer);
    let mut camera = entity.get_mut::<OrbitCamera>().unwrap();
    camera.focus_on(focus);
    camera.set_target_angles(yaw, pitch);
    camera.set_target_distance(distance);
}
