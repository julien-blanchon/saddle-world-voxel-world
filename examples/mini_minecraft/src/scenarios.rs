use bevy::{ecs::system::SystemState, prelude::*};
use saddle_bevy_e2e::{action::Action, actions::assertions, scenario::Scenario};
use saddle_world_voxel_world::{BlockId, BlockRegistry, VoxelWorldStats, VoxelWorldView};

use crate::{
    BLOCK_INTERACTION_RANGE, DebugOverlay, FpsPlayer, PlayerState, raycast_interaction_blocks,
    sample_loaded_solid,
};

pub fn list_scenarios() -> Vec<&'static str> {
    vec!["mini_minecraft_interaction"]
}

pub fn scenario_by_name(name: &str) -> Option<Scenario> {
    match name {
        "mini_minecraft_interaction" => Some(mini_minecraft_interaction()),
        _ => None,
    }
}

fn mini_minecraft_interaction() -> Scenario {
    Scenario::builder("mini_minecraft_interaction")
        .description(
            "Load the sandbox, verify mouse look rotates the player, place a block with right click, remove it with left click, and toggle the debug overlay with F3.",
        )
        .then(Action::WaitFrames(140))
        .then(assertions::resource_satisfies::<
            saddle_world_voxel_world::VoxelWorldStats,
        >("mini minecraft streamed terrain", |stats| {
            stats.loaded_chunks > 0 && stats.meshed_chunks > 0
        }))
        .then(Action::Custom(Box::new(|world| {
            if let Some(setup) = find_interaction_setup(world) {
                let Ok(mut transform) = world.query_filtered::<&mut Transform, With<FpsPlayer>>().single_mut(world) else {
                    return;
                };
                *transform = Transform::from_translation(setup.eye).looking_to(setup.direction, Vec3::Y);
                let (yaw, pitch, _) = transform.rotation.to_euler(EulerRot::YXZ);
                let block_modifications_before = world.resource::<VoxelWorldStats>().block_modifications;
                let mut state = world.resource_mut::<PlayerState>();
                state.yaw = yaw;
                state.pitch = pitch;
                state.cursor_grabbed = true;
                let placed_block = state.selected_block;
                let _ = state;
                world.insert_resource(InteractionSnapshot {
                    place_pos: setup.place_pos,
                    placed_block,
                    block_modifications_before,
                });
            }
        })))
        .then(assertions::custom("interaction setup found a valid build target", |world| {
            world.contains_resource::<InteractionSnapshot>()
        }))
        .then(Action::Custom(Box::new(|world| {
            let Ok(transform) = world.query_filtered::<&Transform, With<FpsPlayer>>().single(world) else {
                return;
            };
            let rotation = transform.rotation;
            world.insert_resource(LookSnapshot(rotation));
        })))
        .then(Action::MouseMotion {
            delta: Vec2::new(96.0, -40.0),
        })
        .then(Action::WaitFrames(2))
        .then(Action::Custom(Box::new(|world| {
            let before = world.resource::<LookSnapshot>().0;
            let after = world
                .query_filtered::<&Transform, With<FpsPlayer>>()
                .single(world)
                .expect("fps player transform should exist")
                .rotation;
            assert_ne!(after, before);
        })))
        .then(Action::Custom(Box::new(|world| {
            if let Some(setup) = find_interaction_setup(world) {
                let Ok(mut transform) = world.query_filtered::<&mut Transform, With<FpsPlayer>>().single_mut(world) else {
                    return;
                };
                *transform = Transform::from_translation(setup.eye).looking_to(setup.direction, Vec3::Y);
                let (yaw, pitch, _) = transform.rotation.to_euler(EulerRot::YXZ);
                let block_modifications_before = world.resource::<VoxelWorldStats>().block_modifications;
                let mut state = world.resource_mut::<PlayerState>();
                state.yaw = yaw;
                state.pitch = pitch;
                state.cursor_grabbed = true;
                let placed_block = state.selected_block;
                let _ = state;
                world.insert_resource(InteractionSnapshot {
                    place_pos: setup.place_pos,
                    placed_block,
                    block_modifications_before,
                });
            }
        })))
        .then(Action::Screenshot("mini_minecraft_start".into()))
        .then(Action::WaitFrames(2))
        .then(Action::PressMouseButton(MouseButton::Right))
        .then(Action::WaitFrames(2))
        .then(Action::ReleaseMouseButton(MouseButton::Right))
        .then(Action::WaitFrames(20))
        .then(Action::Custom(Box::new(|world| {
            let snapshot = world.resource::<InteractionSnapshot>().clone();
            let block_modifications = world.resource::<VoxelWorldStats>().block_modifications;
            let actual = sample_loaded_block(world, snapshot.place_pos);
            assert_eq!(
                actual,
                Some(snapshot.placed_block),
                "expected placed block at {:?}, actual {:?}, block_modifications {}",
                snapshot.place_pos,
                actual,
                block_modifications
            );
            assert_eq!(
                block_modifications,
                snapshot.block_modifications_before + 1
            );
        })))
        .then(Action::Screenshot("mini_minecraft_placed".into()))
        .then(Action::WaitFrames(2))
        .then(Action::PressMouseButton(MouseButton::Left))
        .then(Action::WaitFrames(2))
        .then(Action::ReleaseMouseButton(MouseButton::Left))
        .then(Action::WaitFrames(20))
        .then(Action::Custom(Box::new(|world| {
            let snapshot = world.resource::<InteractionSnapshot>().clone();
            let block_modifications = world.resource::<VoxelWorldStats>().block_modifications;
            let actual = sample_loaded_block(world, snapshot.place_pos);
            assert_eq!(
                actual,
                Some(BlockId::AIR),
                "expected broken block at {:?}, actual {:?}, block_modifications {}",
                snapshot.place_pos,
                actual,
                block_modifications
            );
            assert_eq!(
                block_modifications,
                snapshot.block_modifications_before + 2
            );
        })))
        .then(Action::PressKey(KeyCode::F3))
        .then(Action::WaitFrames(1))
        .then(Action::ReleaseKey(KeyCode::F3))
        .then(Action::WaitFrames(5))
        .then(Action::Custom(Box::new(|world| {
            let visibility = world
                .query_filtered::<&Visibility, With<DebugOverlay>>()
                .single(world)
                .expect("debug overlay entity should exist");
            assert_ne!(*visibility, Visibility::Hidden);
        })))
        .then(Action::Screenshot("mini_minecraft_debug".into()))
        .then(Action::WaitFrames(10))
        .then(assertions::log_summary("mini_minecraft_interaction"))
        .build()
}

#[derive(Clone, Copy, Resource)]
struct InteractionSnapshot {
    place_pos: IVec3,
    placed_block: BlockId,
    block_modifications_before: u64,
}

#[derive(Clone, Copy, Resource)]
struct LookSnapshot(Quat);

#[derive(Clone, Copy)]
struct InteractionSetup {
    eye: Vec3,
    direction: Vec3,
    place_pos: IVec3,
}

fn find_interaction_setup(world: &mut World) -> Option<InteractionSetup> {
    let registry = world.resource::<BlockRegistry>().clone();
    let mut system_state = SystemState::<VoxelWorldView>::new(world);
    let view = system_state.get(world);
    for x in (-8..=20).step_by(4) {
        for z in (-8..=20).step_by(4) {
            let mut top = None;
            for y in (0..=28).rev() {
                if view.sample_loaded_block(IVec3::new(x, y, z)) != Some(BlockId::AIR) {
                    top = Some(y);
                    break;
                }
            }
            let Some(surface_y) = top else {
                continue;
            };

            let eye = Vec3::new(x as f32 + 0.5, surface_y as f32 + 3.5, z as f32 + 5.5);
            if sample_loaded_solid(&view, &registry, eye.floor().as_ivec3()) {
                continue;
            }
            let focus = Vec3::new(x as f32 + 0.5, surface_y as f32 + 0.5, z as f32 + 0.5);
            let direction = (focus - eye).normalize_or_zero();
            if direction == Vec3::ZERO {
                continue;
            }

            let Some(hit) = raycast_interaction_blocks(
                &view,
                &registry,
                eye,
                direction,
                BLOCK_INTERACTION_RANGE,
            ) else {
                continue;
            };
            let place_pos = hit.world_pos + hit.normal;
            if view.sample_loaded_block(place_pos) == Some(BlockId::AIR) {
                return Some(InteractionSetup {
                    eye,
                    direction,
                    place_pos,
                });
            }
        }
    }

    None
}

fn sample_loaded_block(world: &mut World, world_pos: IVec3) -> Option<BlockId> {
    let mut system_state = SystemState::<VoxelWorldView>::new(world);
    let view = system_state.get(world);
    view.sample_loaded_block(world_pos)
}
