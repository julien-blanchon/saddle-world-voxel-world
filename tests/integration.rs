use bevy::{asset::Assets, ecs::system::SystemState, prelude::*};
use saddle_world_voxel_world::{
    BlockEdit, BlockId, BlockModified, ChunkLifecycle, ChunkPos, ChunkStatus, ChunkViewer,
    VoxelBlockSampler, VoxelCommand, VoxelDebugConfig, VoxelDecorationHook, VoxelWorldConfig,
    VoxelWorldGenerator, VoxelWorldPlugin, VoxelWorldRoot, VoxelWorldSystems, VoxelWorldView,
    sample_generated_block, world_to_chunk,
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

fn make_app_with_generator(config: VoxelWorldConfig, generator: VoxelWorldGenerator) -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.insert_resource(Assets::<Image>::default());
    app.insert_resource(Assets::<Mesh>::default());
    app.insert_resource(Assets::<StandardMaterial>::default());
    app.init_resource::<VoxelDebugConfig>();
    app.init_resource::<ModifiedEvents>();
    app.insert_resource(config);
    app.insert_resource(generator);
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

#[derive(Clone)]
struct RaisedPlatformSampler;

impl VoxelBlockSampler for RaisedPlatformSampler {
    fn sample_block(&self, world_pos: IVec3, _config: &VoxelWorldConfig) -> BlockId {
        if world_pos.y <= 2 && world_pos.x.abs() <= 1 && world_pos.z.abs() <= 1 {
            BlockId::SOLID_ACCENT
        } else {
            BlockId::AIR
        }
    }
}

#[derive(Clone)]
struct MarkerDecoration;

impl VoxelDecorationHook for MarkerDecoration {
    fn decorate_block(
        &self,
        world_pos: IVec3,
        sampled: BlockId,
        _config: &VoxelWorldConfig,
    ) -> Option<BlockId> {
        (sampled == BlockId::AIR && world_pos == IVec3::new(0, 3, 0)).then_some(BlockId::EMISSIVE)
    }
}

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
    let generator = app
        .world()
        .resource::<saddle_world_voxel_world::VoxelWorldGenerator>()
        .clone();
    let world_pos = IVec3::new(0, 10, 0);
    let current = sample_generated_block(world_pos, &config, &generator);
    let replacement = if current == BlockId::AIR {
        BlockId::SOLID
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
        .find_map(|(pos, status): (&ChunkPos, &ChunkStatus)| {
            (pos.0 == target_chunk).then_some(status.clone())
        })
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
    let config = VoxelWorldConfig {
        request_radius: 0,
        keep_radius: 1,
        max_chunk_requests_per_frame: 1,
        max_generation_jobs_in_flight: 1,
        max_mesh_jobs_in_flight: 1,
        ..Default::default()
    };
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
fn custom_generator_resource_drives_runtime_generation() {
    let generator =
        VoxelWorldGenerator::new(RaisedPlatformSampler).with_decoration(MarkerDecoration);
    let mut app = make_app_with_generator(VoxelWorldConfig::default(), generator);
    spawn_viewer(&mut app, Vec3::new(0.5, 6.0, 0.5), PrimaryViewer);
    run_until(&mut app, 180, |app| {
        let mut system_state = SystemState::<VoxelWorldView>::new(app.world_mut());
        let view = system_state.get(app.world_mut());
        view.sample_loaded_block(IVec3::new(0, 3, 0)) == Some(BlockId::EMISSIVE)
    });

    let mut system_state = SystemState::<VoxelWorldView>::new(app.world_mut());
    let view = system_state.get(app.world_mut());
    assert_eq!(
        view.sample_loaded_block(IVec3::new(0, 2, 0)),
        Some(BlockId::SOLID_ACCENT)
    );
    assert_eq!(
        view.sample_loaded_block(IVec3::new(0, 3, 0)),
        Some(BlockId::EMISSIVE)
    );
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
