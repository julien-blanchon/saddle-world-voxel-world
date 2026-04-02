use bevy::{asset::Assets, ecs::system::SystemState, prelude::*};
use saddle_world_saddle_world_voxel_world::{
    BlockEdit, BlockId, BlockModified, ChunkLifecycle, ChunkPos, ChunkStatus, ChunkViewer,
    VoxelCommand, VoxelDebugConfig, VoxelWorldConfig, VoxelWorldPlugin, VoxelWorldRoot,
    VoxelWorldSystems, VoxelWorldView, sample_generated_block, world_to_chunk,
};

#[derive(Resource, Default)]
struct ModifiedEvents(Vec<BlockModified>);

fn capture_block_modified(
    mut reader: MessageReader<BlockModified>,
    mut collected: ResMut<ModifiedEvents>,
) {
    collected.0.extend(reader.read().copied());
}

fn make_app_with_config(config: VoxelWorldConfig) -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.insert_resource(Assets::<Image>::default());
    app.insert_resource(Assets::<Mesh>::default());
    app.insert_resource(Assets::<StandardMaterial>::default());
    app.init_resource::<VoxelDebugConfig>();
    app.init_resource::<ModifiedEvents>();
    app.insert_resource(config);
    app.add_plugins(VoxelWorldPlugin::default());
    app.add_systems(
        Update,
        capture_block_modified.after(VoxelWorldSystems::Edits),
    );
    app
}

fn make_app() -> App {
    make_app_with_config(VoxelWorldConfig::default())
}

#[derive(States, Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum GateState {
    #[default]
    Disabled,
    Enabled,
}

fn make_gated_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(bevy::state::app::StatesPlugin);
    app.insert_resource(Assets::<Image>::default());
    app.insert_resource(Assets::<Mesh>::default());
    app.insert_resource(Assets::<StandardMaterial>::default());
    app.init_state::<GateState>();
    app.init_resource::<VoxelDebugConfig>();
    app.init_resource::<ModifiedEvents>();
    app.insert_resource(VoxelWorldConfig::default());
    app.add_plugins(VoxelWorldPlugin::new(
        OnEnter(GateState::Enabled),
        OnExit(GateState::Enabled),
        Update,
    ));
    app.add_systems(
        Update,
        capture_block_modified.after(VoxelWorldSystems::Edits),
    );
    app
}

fn spawn_viewer(app: &mut App, position: Vec3, marker: impl Bundle) -> Entity {
    app.world_mut()
        .spawn((
            Name::new("Test Viewer"),
            ChunkViewer,
            Transform::from_translation(position),
            GlobalTransform::from_translation(position),
            marker,
        ))
        .id()
}

fn run_until(app: &mut App, max_frames: usize, mut condition: impl FnMut(&mut App) -> bool) {
    for _ in 0..max_frames {
        app.update();
        if condition(app) {
            return;
        }
    }
    panic!("condition was not met within {max_frames} frames");
}

#[derive(Component)]
struct PrimaryViewer;

#[derive(Component)]
struct SecondaryViewer;

#[test]
fn plugin_initializes_without_panic() {
    let mut app = make_app();
    app.update();
    let count = {
        let mut query = app.world_mut().query::<&VoxelWorldRoot>();
        query.iter(app.world()).count()
    };
    assert!(count <= 1);
}

#[test]
fn viewer_triggers_chunk_requests_and_generation_integrates() {
    let mut app = make_app();
    spawn_viewer(&mut app, Vec3::new(8.0, 8.0, 8.0), PrimaryViewer);
    run_until(&mut app, 120, |app| {
        app.world()
            .resource::<saddle_world_voxel_world::VoxelWorldStats>()
            .generated_chunks
            > 0
            && app
                .world_mut()
                .query::<&ChunkStatus>()
                .iter(app.world())
                .any(|status| {
                    matches!(
                        status.lifecycle,
                        ChunkLifecycle::Generated
                            | ChunkLifecycle::Meshed
                            | ChunkLifecycle::Persisted
                    )
                })
    });
    assert!(
        app.world()
            .resource::<saddle_world_voxel_world::VoxelWorldStats>()
            .loaded_chunks
            > 0
    );
}

#[test]
fn block_edit_triggers_dirty_chunk_and_emitted_message() {
    let mut app = make_app();
    spawn_viewer(&mut app, Vec3::new(8.0, 8.0, 8.0), PrimaryViewer);
    run_until(&mut app, 120, |app| {
        app.world()
            .resource::<saddle_world_voxel_world::VoxelWorldStats>()
            .generated_chunks
            > 0
    });

    let config = app.world().resource::<VoxelWorldConfig>().clone();
    let world_pos = IVec3::new(0, 10, 0);
    let current = sample_generated_block(world_pos, &config);
    let replacement = if current == BlockId::AIR {
        BlockId::STONE
    } else {
        BlockId::AIR
    };
    app.world_mut()
        .resource_mut::<Messages<VoxelCommand>>()
        .write(VoxelCommand::SetBlock(BlockEdit {
            world_pos,
            block: replacement,
        }));
    run_until(&mut app, 60, |app| {
        !app.world().resource::<ModifiedEvents>().0.is_empty()
    });

    let modified = &app.world().resource::<ModifiedEvents>().0;
    assert_eq!(modified[0].world_pos, world_pos);
    let target_chunk = world_to_chunk(world_pos, config.chunk_dims);
    let chunk_status = app
        .world_mut()
        .query::<(&ChunkPos, &ChunkStatus)>()
        .iter(app.world())
        .find_map(|(pos, status)| (pos.0 == target_chunk).then_some(status.clone()))
        .expect("edited chunk should exist");
    assert!(chunk_status.dirty);
    assert!(
        app.world()
            .resource::<saddle_world_voxel_world::VoxelWorldStats>()
            .block_modifications
            >= 1
    );
}

#[test]
fn neighbor_generation_triggers_boundary_remesh() {
    let mut config = VoxelWorldConfig::default();
    config.request_radius = 0;
    config.keep_radius = 1;
    config.max_chunk_requests_per_frame = 1;
    config.max_generation_jobs_in_flight = 1;
    config.max_mesh_jobs_in_flight = 1;
    let mut app = make_app_with_config(config);

    spawn_viewer(&mut app, Vec3::new(8.0, 8.0, 8.0), PrimaryViewer);
    run_until(&mut app, 180, |app| {
        app.world()
            .resource::<saddle_world_voxel_world::VoxelWorldStats>()
            .remeshed_chunks
            >= 1
    });

    spawn_viewer(&mut app, Vec3::new(24.0, 8.0, 8.0), SecondaryViewer);
    run_until(&mut app, 240, |app| {
        app.world()
            .resource::<saddle_world_voxel_world::VoxelWorldStats>()
            .remeshed_chunks
            >= 3
    });

    let stats = app
        .world()
        .resource::<saddle_world_voxel_world::VoxelWorldStats>();
    assert!(stats.remeshed_chunks >= 3);
}

#[test]
fn injectable_schedules_gate_activation_cleanup_and_mesh_release() {
    let mut app = make_gated_app();
    app.update();

    let initial_roots = {
        let mut query = app.world_mut().query::<&VoxelWorldRoot>();
        query.iter(app.world()).count()
    };
    assert_eq!(initial_roots, 0);

    app.world_mut()
        .resource_mut::<NextState<GateState>>()
        .set(GateState::Enabled);
    app.update();
    spawn_viewer(&mut app, Vec3::new(8.0, 8.0, 8.0), PrimaryViewer);
    run_until(&mut app, 180, |app| {
        app.world()
            .resource::<saddle_world_voxel_world::VoxelWorldStats>()
            .meshed_chunks
            > 0
    });

    let mesh_count_before = app.world().resource::<Assets<Mesh>>().len();
    assert!(mesh_count_before > 0);

    app.world_mut()
        .resource_mut::<NextState<GateState>>()
        .set(GateState::Disabled);
    app.update();

    let remaining_roots = {
        let mut query = app.world_mut().query::<&VoxelWorldRoot>();
        query.iter(app.world()).count()
    };
    let remaining_chunks = {
        let mut query = app.world_mut().query::<&ChunkPos>();
        query.iter(app.world()).count()
    };
    assert_eq!(remaining_roots, 0);
    assert_eq!(remaining_chunks, 0);
    assert_eq!(app.world().resource::<Assets<Mesh>>().len(), 0);
}

#[test]
fn read_only_view_exposes_loaded_blocks() {
    let mut app = make_app();
    spawn_viewer(&mut app, Vec3::new(8.0, 8.0, 8.0), PrimaryViewer);
    run_until(&mut app, 120, |app| {
        app.world()
            .resource::<saddle_world_voxel_world::VoxelWorldStats>()
            .generated_chunks
            > 0
    });

    let mut system_state = SystemState::<VoxelWorldView>::new(app.world_mut());
    let view = system_state.get(app.world_mut());
    let loaded = view.sample_loaded_block(IVec3::new(0, 8, 0));
    assert!(loaded.is_some());
}
