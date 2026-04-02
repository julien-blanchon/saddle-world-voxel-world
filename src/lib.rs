#[doc(hidden)]
pub mod benchmark_support;
mod block;
mod chunk;
mod config;
mod coordinates;
mod meshing;
mod persistence;
mod raycast;
mod stats;
mod terrain;
mod viewer;

pub use block::{
    BlockDefinition, BlockFaceAtlas, BlockId, BlockRegistry, CollisionKind, MaterialClass, MeshKind,
};
pub use chunk::{ChunkData, ChunkLifecycle, ChunkPos, ChunkStatus};
pub use config::{
    AtlasConfig, GeneratorKind, LightingConfig, MeshingConfig, SaveMode, SavePolicy, TerrainConfig,
    VoxelWorldConfig,
};
pub use coordinates::{
    chunk_origin, chunk_translation, is_on_chunk_boundary, local_to_world,
    neighboring_chunks_for_boundary, world_to_chunk, world_to_chunk_local, world_to_local,
};
pub use persistence::{decode_rle_blocks, encode_rle_blocks, load_chunk_delta, save_chunk_delta};
pub use raycast::{
    BlockSampler, RaycastHit, chunk_bounds_world, raycast_blocks, rebuild_world_pos,
};
pub use stats::{VoxelDebugColorMode, VoxelDebugConfig, VoxelWorldStats};
pub use terrain::{generate_chunk, sample_generated_block};
pub use viewer::{ChunkViewer, ChunkViewerSettings};

use std::{cmp::Reverse, collections::HashMap};

use bevy::platform::time::Instant;

use bevy::{
    app::PostStartup,
    ecs::{intern::Interned, schedule::ScheduleLabel, system::SystemParam},
    prelude::*,
    tasks::{AsyncComputeTaskPool, Task, futures::check_ready},
};

use meshing::{ChunkMeshArtifacts, PaddedChunk, SampledBlock, build_chunk_meshes};
#[derive(SystemSet, Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum VoxelWorldSystems {
    Viewers,
    Streaming,
    Generation,
    Edits,
    Meshing,
    Collision,
    Lighting,
    Persistence,
    Diagnostics,
}

#[derive(ScheduleLabel, Debug, Clone, PartialEq, Eq, Hash)]
struct NeverDeactivateSchedule;

#[derive(Component, Debug, Default, Reflect)]
#[reflect(Component, Default)]
pub struct VoxelWorldRoot;

#[derive(Message, Debug, Clone, Copy)]
pub struct ChunkLoaded {
    pub pos: IVec3,
}

#[derive(Message, Debug, Clone, Copy)]
pub struct ChunkUnloaded {
    pub pos: IVec3,
}

#[derive(Message, Debug, Clone, Copy)]
pub struct BlockModified {
    pub world_pos: IVec3,
    pub old: BlockId,
    pub new: BlockId,
}

#[derive(Clone, Debug)]
pub struct BlockEdit {
    pub world_pos: IVec3,
    pub block: BlockId,
}

#[derive(Message, Clone, Debug)]
pub enum VoxelCommand {
    SetBlock(BlockEdit),
    Batch(Vec<BlockEdit>),
}

#[derive(Resource, Default)]
struct RuntimeState {
    active: bool,
    frame: u64,
    root: Option<Entity>,
    chunks: HashMap<IVec3, ChunkRecord>,
    desired: HashMap<IVec3, DesiredChunk>,
    material_state: Option<MaterialState>,
    last_save_tick: f32,
}

struct MaterialState {
    opaque_material: Handle<StandardMaterial>,
    cutout_material: Handle<StandardMaterial>,
}

struct ChunkRecord {
    entity: Entity,
    data: Option<ChunkData>,
    overrides: std::collections::BTreeMap<u32, BlockId>,
    generation_task: Option<Task<GenerationJobResult>>,
    meshing_task: Option<Task<MeshingJobResult>>,
    pending_remesh: bool,
    opaque_mesh_entity: Option<Entity>,
    cutout_mesh_entity: Option<Entity>,
    opaque_mesh: Option<Handle<Mesh>>,
    cutout_mesh: Option<Handle<Mesh>>,
    status: ChunkStatus,
    priority: i32,
    last_distance_sq: i32,
    requested_this_frame: bool,
    keep_loaded: bool,
}

#[derive(Clone, Copy)]
struct DesiredChunk {
    priority: Option<i32>,
    distance_sq: i32,
    keep_loaded: bool,
}

struct GenerationJobResult {
    data: ChunkData,
    duration_ms: f32,
}

struct MeshingJobResult {
    artifacts: ChunkMeshArtifacts,
    duration_ms: f32,
}

#[derive(SystemParam)]
pub struct VoxelWorldView<'w> {
    config: Res<'w, VoxelWorldConfig>,
    runtime: Res<'w, RuntimeState>,
}

impl VoxelWorldView<'_> {
    #[must_use]
    pub fn sample_loaded_block(&self, world_pos: IVec3) -> Option<BlockId> {
        let (chunk, local) = world_to_chunk_local(world_pos, self.config.chunk_dims);
        self.runtime
            .chunks
            .get(&chunk)
            .and_then(|record| record.data.as_ref())
            .map(|data| data.get(local))
    }

    #[must_use]
    pub fn chunk_present(&self, chunk: IVec3) -> bool {
        self.runtime.chunks.contains_key(&chunk)
    }

    #[must_use]
    pub fn world_to_chunk_local(&self, world_pos: IVec3) -> (IVec3, UVec3) {
        world_to_chunk_local(world_pos, self.config.chunk_dims)
    }
}

pub struct VoxelWorldPlugin {
    pub activate_schedule: Interned<dyn ScheduleLabel>,
    pub deactivate_schedule: Interned<dyn ScheduleLabel>,
    pub update_schedule: Interned<dyn ScheduleLabel>,
}

impl VoxelWorldPlugin {
    #[must_use]
    pub fn new(
        activate_schedule: impl ScheduleLabel,
        deactivate_schedule: impl ScheduleLabel,
        update_schedule: impl ScheduleLabel,
    ) -> Self {
        Self {
            activate_schedule: activate_schedule.intern(),
            deactivate_schedule: deactivate_schedule.intern(),
            update_schedule: update_schedule.intern(),
        }
    }

    #[must_use]
    pub fn always_on(update_schedule: impl ScheduleLabel) -> Self {
        Self::new(PostStartup, NeverDeactivateSchedule, update_schedule)
    }
}

impl Default for VoxelWorldPlugin {
    fn default() -> Self {
        Self::always_on(Update)
    }
}

impl Plugin for VoxelWorldPlugin {
    fn build(&self, app: &mut App) {
        if self.deactivate_schedule == NeverDeactivateSchedule.intern() {
            app.init_schedule(NeverDeactivateSchedule);
        }

        app.init_resource::<BlockRegistry>()
            .init_resource::<RuntimeState>()
            .init_resource::<VoxelDebugConfig>()
            .init_resource::<VoxelWorldConfig>()
            .init_resource::<VoxelWorldStats>()
            .add_message::<VoxelCommand>()
            .add_message::<ChunkLoaded>()
            .add_message::<ChunkUnloaded>()
            .add_message::<BlockModified>()
            .register_type::<AtlasConfig>()
            .register_type::<BlockDefinition>()
            .register_type::<BlockFaceAtlas>()
            .register_type::<BlockId>()
            .register_type::<ChunkLifecycle>()
            .register_type::<ChunkPos>()
            .register_type::<ChunkStatus>()
            .register_type::<ChunkViewer>()
            .register_type::<ChunkViewerSettings>()
            .register_type::<GeneratorKind>()
            .register_type::<LightingConfig>()
            .register_type::<MaterialClass>()
            .register_type::<MeshKind>()
            .register_type::<MeshingConfig>()
            .register_type::<SaveMode>()
            .register_type::<SavePolicy>()
            .register_type::<TerrainConfig>()
            .register_type::<VoxelDebugColorMode>()
            .register_type::<VoxelDebugConfig>()
            .register_type::<VoxelWorldConfig>()
            .register_type::<VoxelWorldRoot>()
            .register_type::<VoxelWorldStats>()
            .configure_sets(
                self.update_schedule,
                (
                    VoxelWorldSystems::Viewers,
                    VoxelWorldSystems::Streaming,
                    VoxelWorldSystems::Generation,
                    VoxelWorldSystems::Edits,
                    VoxelWorldSystems::Meshing,
                    VoxelWorldSystems::Collision,
                    VoxelWorldSystems::Lighting,
                    VoxelWorldSystems::Persistence,
                    VoxelWorldSystems::Diagnostics,
                )
                    .chain(),
            )
            .add_systems(self.activate_schedule, activate_runtime)
            .add_systems(self.deactivate_schedule, deactivate_runtime)
            .add_systems(
                self.update_schedule,
                (
                    refresh_viewer_targets.in_set(VoxelWorldSystems::Viewers),
                    stream_requested_chunks.in_set(VoxelWorldSystems::Streaming),
                    (queue_generation_jobs, poll_generation_jobs)
                        .chain()
                        .in_set(VoxelWorldSystems::Generation),
                    process_voxel_commands.in_set(VoxelWorldSystems::Edits),
                    (queue_meshing_jobs, poll_meshing_jobs)
                        .chain()
                        .in_set(VoxelWorldSystems::Meshing),
                    persist_dirty_chunks.in_set(VoxelWorldSystems::Persistence),
                    refresh_runtime_stats.in_set(VoxelWorldSystems::Diagnostics),
                )
                    .run_if(runtime_is_active),
            );

        if app.is_plugin_added::<bevy::gizmos::GizmoPlugin>() {
            app.add_systems(
                self.update_schedule,
                update_debug_gizmos
                    .in_set(VoxelWorldSystems::Diagnostics)
                    .run_if(runtime_is_active),
            );
        }
    }
}

fn activate_runtime(
    mut commands: Commands,
    config: Res<VoxelWorldConfig>,
    mut runtime: ResMut<RuntimeState>,
) {
    if runtime.active {
        return;
    }
    if let Err(error) = config.validate() {
        error!("[voxel_world] invalid config: {error}");
        return;
    }

    let root = commands
        .spawn((
            Name::new("Voxel World"),
            VoxelWorldRoot,
            Transform::default(),
        ))
        .id();
    runtime.root = Some(root);
    runtime.active = true;
}

fn deactivate_runtime(
    mut commands: Commands,
    config: Res<VoxelWorldConfig>,
    mut meshes: Option<ResMut<Assets<Mesh>>>,
    mut runtime: ResMut<RuntimeState>,
) {
    runtime.active = false;
    let positions: Vec<IVec3> = runtime.chunks.keys().copied().collect();
    for pos in positions {
        if let Some(mut record) = runtime.chunks.remove(&pos) {
            let _ = save_chunk_delta(
                &config.save_policy,
                pos,
                config.seed,
                config.generator_version,
                record.status.version,
                &record.overrides,
            );
            if let Some(meshes) = meshes.as_deref_mut() {
                release_chunk_mesh_assets(&mut record, meshes);
            }
            commands.entity(record.entity).despawn();
        }
    }
    if let Some(root) = runtime.root.take() {
        commands.entity(root).despawn();
    }
}

fn runtime_is_active(runtime: Res<RuntimeState>) -> bool {
    runtime.active
}

fn refresh_viewer_targets(
    config: Res<VoxelWorldConfig>,
    mut runtime: ResMut<RuntimeState>,
    viewers: Query<(&GlobalTransform, Option<&ChunkViewerSettings>), With<ChunkViewer>>,
) {
    runtime.frame = runtime.frame.wrapping_add(1);
    for record in runtime.chunks.values_mut() {
        record.requested_this_frame = false;
        record.keep_loaded = false;
        record.priority = i32::MIN;
        record.last_distance_sq = i32::MAX;
    }

    let mut desired: HashMap<IVec3, (i32, i32)> = HashMap::new();
    runtime.desired.clear();
    for (transform, settings) in &viewers {
        let position = transform.translation().floor().as_ivec3();
        let viewer_chunk = world_to_chunk(position, config.chunk_dims);
        let request_radius = settings
            .map(|settings| settings.request_radius.max(config.request_radius))
            .unwrap_or(config.request_radius);
        let keep_radius = settings
            .map(|settings| settings.keep_radius.max(config.keep_radius))
            .unwrap_or(config.keep_radius);
        let priority = settings.map(|settings| settings.priority).unwrap_or(0);

        for z in -(keep_radius as i32)..=keep_radius as i32 {
            for y in -(keep_radius as i32)..=keep_radius as i32 {
                for x in -(keep_radius as i32)..=keep_radius as i32 {
                    let offset = IVec3::new(x, y, z);
                    let distance_sq = offset.length_squared();
                    let chunk = viewer_chunk + offset;
                    let keep_limit = (keep_radius * keep_radius) as i32;
                    let request_limit = (request_radius * request_radius) as i32;
                    let entry = desired.entry(chunk).or_insert((i32::MIN, i32::MAX));
                    if distance_sq <= request_limit {
                        entry.0 = entry.0.max(priority * 10_000 - distance_sq);
                        entry.1 = entry.1.min(distance_sq);
                    }
                    runtime
                        .desired
                        .entry(chunk)
                        .and_modify(|entry| {
                            entry.distance_sq = entry.distance_sq.min(distance_sq);
                            entry.keep_loaded |= distance_sq <= keep_limit;
                            if distance_sq <= request_limit {
                                let candidate = priority * 10_000 - distance_sq;
                                entry.priority = Some(
                                    entry
                                        .priority
                                        .map_or(candidate, |current| current.max(candidate)),
                                );
                            }
                        })
                        .or_insert(DesiredChunk {
                            priority: (distance_sq <= request_limit)
                                .then_some(priority * 10_000 - distance_sq),
                            distance_sq,
                            keep_loaded: distance_sq <= keep_limit,
                        });
                    if let Some(record) = runtime.chunks.get_mut(&chunk)
                        && distance_sq <= keep_limit
                    {
                        record.keep_loaded = true;
                        record.priority = record.priority.max(priority * 10_000 - distance_sq);
                        record.last_distance_sq = record.last_distance_sq.min(distance_sq);
                    }
                }
            }
        }
    }

    for (chunk, (priority, distance_sq)) in desired {
        if let Some(record) = runtime.chunks.get_mut(&chunk)
            && priority != i32::MIN
        {
            record.requested_this_frame = true;
            record.priority = record.priority.max(priority);
            record.last_distance_sq = record.last_distance_sq.min(distance_sq);
        }
    }
}

fn stream_requested_chunks(
    mut commands: Commands,
    config: Res<VoxelWorldConfig>,
    mut meshes: Option<ResMut<Assets<Mesh>>>,
    mut runtime: ResMut<RuntimeState>,
    mut stats: ResMut<VoxelWorldStats>,
    mut unloaded: MessageWriter<ChunkUnloaded>,
) {
    let mut unloaded_this_frame = 0_u32;
    let stale: Vec<IVec3> = runtime
        .chunks
        .iter()
        .filter_map(|(chunk, _record)| {
            let keep = runtime
                .desired
                .get(chunk)
                .map(|desired| desired.keep_loaded)
                .unwrap_or(false);
            (!keep).then_some(*chunk)
        })
        .collect();
    for chunk in stale {
        if unloaded_this_frame >= config.max_chunk_unloads_per_frame {
            break;
        }
        if let Some(mut record) = runtime.chunks.remove(&chunk) {
            unloaded.write(ChunkUnloaded { pos: chunk });
            if let Some(meshes) = meshes.as_deref_mut() {
                release_chunk_mesh_assets(&mut record, meshes);
            }
            commands.entity(record.entity).despawn();
            let _ = save_chunk_delta(
                &config.save_policy,
                chunk,
                config.seed,
                config.generator_version,
                record.status.version,
                &record.overrides,
            );
            stats.unloaded_chunks = stats.unloaded_chunks.saturating_add(1);
            unloaded_this_frame += 1;
        }
    }

    let mut candidates: Vec<(IVec3, DesiredChunk)> = runtime
        .desired
        .iter()
        .filter_map(|(chunk, desired)| {
            (!runtime.chunks.contains_key(chunk))
                .then_some((*chunk, *desired))
                .filter(|(_, desired)| desired.priority.is_some())
        })
        .collect();
    candidates.sort_by(|left, right| {
        right
            .1
            .priority
            .unwrap_or(i32::MIN)
            .cmp(&left.1.priority.unwrap_or(i32::MIN))
            .then_with(|| left.1.distance_sq.cmp(&right.1.distance_sq))
    });

    for (chunk, desired) in candidates
        .into_iter()
        .take(config.max_chunk_requests_per_frame as usize)
    {
        let entity = commands
            .spawn((
                Name::new(format!(
                    "Voxel Chunk ({}, {}, {})",
                    chunk.x, chunk.y, chunk.z
                )),
                ChunkPos(chunk),
                ChunkStatus::default(),
                Transform::from_translation(chunk_translation(chunk, config.chunk_dims)),
            ))
            .id();
        if let Some(root) = runtime.root {
            commands.entity(entity).set_parent_in_place(root);
        }
        runtime.chunks.insert(
            chunk,
            ChunkRecord {
                entity,
                data: None,
                overrides: Default::default(),
                generation_task: None,
                meshing_task: None,
                pending_remesh: false,
                opaque_mesh_entity: None,
                cutout_mesh_entity: None,
                opaque_mesh: None,
                cutout_mesh: None,
                status: ChunkStatus::default(),
                priority: desired.priority.unwrap_or_default(),
                last_distance_sq: desired.distance_sq,
                requested_this_frame: true,
                keep_loaded: desired.keep_loaded,
            },
        );
    }
}

fn queue_generation_jobs(config: Res<VoxelWorldConfig>, mut runtime: ResMut<RuntimeState>) {
    let in_flight = runtime
        .chunks
        .values()
        .filter(|record| record.generation_task.is_some())
        .count();
    let budget = config
        .max_generation_jobs_in_flight
        .saturating_sub(in_flight);
    if budget == 0 {
        return;
    }

    let mut queued: Vec<(IVec3, i32)> = runtime
        .chunks
        .iter()
        .filter_map(|(chunk, record)| {
            if record.data.is_none() && record.generation_task.is_none() {
                Some((*chunk, record.priority))
            } else {
                None
            }
        })
        .collect();
    queued.sort_by_key(|entry| Reverse(entry.1));

    let pool = AsyncComputeTaskPool::get();
    for (chunk, _) in queued.into_iter().take(budget) {
        let config = config.clone();
        let task = pool.spawn(async move {
            let start = Instant::now();
            let data = generate_chunk(chunk, &config);
            GenerationJobResult {
                data,
                duration_ms: start.elapsed().as_secs_f32() * 1000.0,
            }
        });
        if let Some(record) = runtime.chunks.get_mut(&chunk) {
            record.status.lifecycle = ChunkLifecycle::Generating;
            record.generation_task = Some(task);
        }
    }
}

fn poll_generation_jobs(
    mut commands: Commands,
    config: Res<VoxelWorldConfig>,
    mut runtime: ResMut<RuntimeState>,
    mut stats: ResMut<VoxelWorldStats>,
    mut loaded: MessageWriter<ChunkLoaded>,
) {
    let positions: Vec<IVec3> = runtime.chunks.keys().copied().collect();
    for chunk in positions {
        let ready = runtime
            .chunks
            .get_mut(&chunk)
            .and_then(|record| record.generation_task.as_mut())
            .and_then(check_ready);
        let Some(result) = ready else {
            continue;
        };
        let record = runtime.chunks.get_mut(&chunk).unwrap();
        record.generation_task = None;
        let mut data = result.data;
        if let Ok(Some(edits)) = load_chunk_delta(
            &config.save_policy,
            chunk,
            config.seed,
            config.generator_version,
        ) {
            for edit in edits {
                record.overrides.insert(edit.local_index, edit.block);
                let local = data.local_from_index(edit.local_index);
                data.set(local, edit.block);
            }
        }
        record.status.lifecycle = ChunkLifecycle::Generated;
        record.data = Some(data);
        stats.generated_chunks += 1;
        stats.last_generation_time_ms = result.duration_ms;
        loaded.write(ChunkLoaded { pos: chunk });
        commands.entity(record.entity).insert(record.status.clone());
        for (entity, status) in mark_neighbor_generation_dependencies_dirty(chunk, &mut runtime) {
            commands.entity(entity).insert(status);
        }
    }
}

fn process_voxel_commands(
    config: Res<VoxelWorldConfig>,
    mut commands: Commands,
    registry: Res<BlockRegistry>,
    mut runtime: ResMut<RuntimeState>,
    mut edits: MessageReader<VoxelCommand>,
    mut modified: MessageWriter<BlockModified>,
    mut stats: ResMut<VoxelWorldStats>,
) {
    let mut apply = |edit: &BlockEdit,
                     config: &VoxelWorldConfig,
                     runtime: &mut RuntimeState,
                     modified: &mut MessageWriter<BlockModified>,
                     stats: &mut VoxelWorldStats| {
        let (chunk, local) = world_to_chunk_local(edit.world_pos, config.chunk_dims);
        if !registry.contains(edit.block) {
            return;
        }
        if !runtime.chunks.contains_key(&chunk) {
            let entity = commands
                .spawn((
                    Name::new(format!(
                        "Voxel Chunk ({}, {}, {})",
                        chunk.x, chunk.y, chunk.z
                    )),
                    ChunkPos(chunk),
                    ChunkStatus::default(),
                    Transform::from_translation(chunk_translation(chunk, config.chunk_dims)),
                ))
                .id();
            if let Some(root) = runtime.root {
                commands.entity(entity).set_parent_in_place(root);
            }
            runtime.chunks.insert(
                chunk,
                ChunkRecord {
                    entity,
                    data: Some(generate_chunk(chunk, config)),
                    overrides: Default::default(),
                    generation_task: None,
                    meshing_task: None,
                    pending_remesh: false,
                    opaque_mesh_entity: None,
                    cutout_mesh_entity: None,
                    opaque_mesh: None,
                    cutout_mesh: None,
                    status: ChunkStatus {
                        lifecycle: ChunkLifecycle::Dirty,
                        dirty: true,
                        version: 1,
                        persisted_version: 0,
                    },
                    priority: i32::MAX / 2,
                    last_distance_sq: 0,
                    requested_this_frame: true,
                    keep_loaded: true,
                },
            );
        }

        let record = runtime.chunks.get_mut(&chunk).unwrap();
        let old = record
            .data
            .as_ref()
            .map(|data| data.get(local))
            .unwrap_or(BlockId::AIR);
        if old == edit.block {
            return;
        }

        let local_index = record.data.as_ref().unwrap().index(local) as u32;
        record.overrides.insert(local_index, edit.block);
        record.data.as_mut().unwrap().set(local, edit.block);
        record.status.lifecycle = ChunkLifecycle::Dirty;
        record.status.dirty = true;
        record.status.version = record.status.version.wrapping_add(1);
        record.priority = i32::MAX;
        commands.entity(record.entity).insert(record.status.clone());
        modified.write(BlockModified {
            world_pos: edit.world_pos,
            old,
            new: edit.block,
        });
        stats.block_modifications += 1;

        for neighbor in neighboring_chunks_for_boundary(local, config.chunk_dims) {
            if let Some(neighbor_record) = runtime.chunks.get_mut(&(chunk + neighbor)) {
                mark_chunk_for_remesh(neighbor_record, i32::MAX - 1);
                commands
                    .entity(neighbor_record.entity)
                    .insert(neighbor_record.status.clone());
            }
        }
    };

    for command in edits.read() {
        match command {
            VoxelCommand::SetBlock(edit) => {
                apply(edit, &config, &mut runtime, &mut modified, &mut stats);
            }
            VoxelCommand::Batch(batch) => {
                for edit in batch {
                    apply(edit, &config, &mut runtime, &mut modified, &mut stats);
                }
            }
        }
    }
}

fn queue_meshing_jobs(
    config: Res<VoxelWorldConfig>,
    registry: Res<BlockRegistry>,
    mut runtime: ResMut<RuntimeState>,
) {
    let in_flight = runtime
        .chunks
        .values()
        .filter(|record| record.meshing_task.is_some())
        .count();
    let budget = config.max_mesh_jobs_in_flight.saturating_sub(in_flight);
    if budget == 0 {
        return;
    }

    let mut queued: Vec<(IVec3, i32)> = runtime
        .chunks
        .iter()
        .filter_map(|(chunk, record)| {
            if record.data.is_some()
                && record.meshing_task.is_none()
                && matches!(
                    record.status.lifecycle,
                    ChunkLifecycle::Generated | ChunkLifecycle::Dirty
                )
            {
                Some((*chunk, record.priority))
            } else {
                None
            }
        })
        .collect();
    queued.sort_by_key(|entry| Reverse(entry.1));
    let pool = AsyncComputeTaskPool::get();

    for (chunk, _) in queued.into_iter().take(budget) {
        let config = config.clone();
        let registry = registry.clone();
        let center = runtime
            .chunks
            .get(&chunk)
            .and_then(|record| record.data.clone())
            .unwrap();
        let padded = build_padded_chunk(chunk, &runtime, &config);
        let task = pool.spawn(async move {
            let start = Instant::now();
            let artifacts = build_chunk_meshes(chunk, &center, &padded, &registry, &config);
            MeshingJobResult {
                artifacts,
                duration_ms: start.elapsed().as_secs_f32() * 1000.0,
            }
        });
        if let Some(record) = runtime.chunks.get_mut(&chunk) {
            record.status.lifecycle = ChunkLifecycle::Meshing;
            record.meshing_task = Some(task);
        }
    }
}

fn build_padded_chunk(
    chunk: IVec3,
    runtime: &RuntimeState,
    config: &VoxelWorldConfig,
) -> PaddedChunk {
    let mut padded = PaddedChunk::new_unknown(config.chunk_dims);
    let dims = config.chunk_dims.as_ivec3();
    for z in -1..=dims.z {
        for y in -1..=dims.y {
            for x in -1..=dims.x {
                let padded_local = IVec3::new(x + 1, y + 1, z + 1);
                let local = IVec3::new(x, y, z);
                if local.cmpge(IVec3::ZERO).all() && local.cmplt(dims).all() {
                    if let Some(data) = runtime
                        .chunks
                        .get(&chunk)
                        .and_then(|record| record.data.as_ref())
                    {
                        padded.set(
                            padded_local,
                            SampledBlock {
                                id: data.get(local.as_uvec3()),
                                known: true,
                            },
                        );
                    }
                    continue;
                }

                let world = chunk * dims + local;
                let (neighbor_chunk, neighbor_local) =
                    world_to_chunk_local(world, config.chunk_dims);
                if let Some(data) = runtime
                    .chunks
                    .get(&neighbor_chunk)
                    .and_then(|record| record.data.as_ref())
                {
                    padded.set(
                        padded_local,
                        SampledBlock {
                            id: data.get(neighbor_local),
                            known: true,
                        },
                    );
                }
            }
        }
    }
    padded
}

#[allow(clippy::too_many_arguments)]
fn poll_meshing_jobs(
    mut commands: Commands,
    asset_server: Option<Res<AssetServer>>,
    mut images: Option<ResMut<Assets<Image>>>,
    meshes: Option<ResMut<Assets<Mesh>>>,
    mut materials: Option<ResMut<Assets<StandardMaterial>>>,
    config: Res<VoxelWorldConfig>,
    mut runtime: ResMut<RuntimeState>,
    mut stats: ResMut<VoxelWorldStats>,
) {
    if runtime.material_state.is_none() {
        let Some(materials) = materials.as_mut() else {
            return;
        };
        let atlas =
            generated_or_loaded_atlas(&config, asset_server.as_deref(), images.as_deref_mut());
        runtime.material_state = Some(MaterialState {
            opaque_material: materials.add(StandardMaterial {
                base_color_texture: Some(atlas.clone()),
                perceptual_roughness: 1.0,
                ..default()
            }),
            cutout_material: materials.add(StandardMaterial {
                base_color_texture: Some(atlas),
                alpha_mode: AlphaMode::Mask(0.5),
                cull_mode: None,
                perceptual_roughness: 1.0,
                ..default()
            }),
        });
    }

    let Some(mut meshes) = meshes else {
        return;
    };
    let material_handles = {
        let material_state = runtime.material_state.as_ref().unwrap();
        (
            material_state.opaque_material.clone(),
            material_state.cutout_material.clone(),
        )
    };
    let positions: Vec<IVec3> = runtime.chunks.keys().copied().collect();
    for chunk in positions {
        let ready = runtime
            .chunks
            .get_mut(&chunk)
            .and_then(|record| record.meshing_task.as_mut())
            .and_then(check_ready);
        let Some(result) = ready else {
            continue;
        };

        let record = runtime.chunks.get_mut(&chunk).unwrap();
        record.meshing_task = None;
        let _ = result.artifacts.counts;
        stats.remeshed_chunks += 1;
        stats.last_meshing_time_ms = result.duration_ms;
        apply_mesh_part(
            &mut commands,
            &mut meshes,
            record,
            result.artifacts.opaque_mesh,
            result.artifacts.cutout_mesh,
            &MaterialState {
                opaque_material: material_handles.0.clone(),
                cutout_material: material_handles.1.clone(),
            },
        );
        record.status.lifecycle = if record.pending_remesh {
            record.pending_remesh = false;
            ChunkLifecycle::Dirty
        } else {
            ChunkLifecycle::Meshed
        };
        commands.entity(record.entity).insert(record.status.clone());
    }
}

fn generated_or_loaded_atlas(
    config: &VoxelWorldConfig,
    asset_server: Option<&AssetServer>,
    images: Option<&mut Assets<Image>>,
) -> Handle<Image> {
    if let Some(asset_path) = &config.atlas.asset_path {
        asset_server
            .expect("AssetServer is required when atlas.asset_path is set")
            .load(asset_path.clone())
    } else {
        let image = generate_debug_atlas(config);
        images
            .expect("Assets<Image> is required for generated atlas images")
            .add(image)
    }
}

fn generate_debug_atlas(config: &VoxelWorldConfig) -> Image {
    let width = config.atlas.tile_size.x * config.atlas.columns as u32;
    let height = config.atlas.tile_size.y * config.atlas.rows as u32;
    let mut data = vec![0_u8; (width * height * 4) as usize];
    let palette: [[u8; 4]; 9] = [
        [34, 44, 56, 255],
        [96, 172, 79, 255],
        [94, 128, 71, 255],
        [118, 87, 60, 255],
        [126, 134, 145, 255],
        [215, 201, 135, 255],
        [74, 141, 205, 192],
        [148, 184, 92, 170],
        [247, 215, 106, 255],
    ];

    for (tile, color) in palette.iter().copied().enumerate() {
        let column = tile as u32 % config.atlas.columns as u32;
        let row = tile as u32 / config.atlas.columns as u32;
        for y in 0..config.atlas.tile_size.y {
            for x in 0..config.atlas.tile_size.x {
                let gx = column * config.atlas.tile_size.x + x;
                let gy = row * config.atlas.tile_size.y + y;
                let index = ((gy * width + gx) * 4) as usize;
                let shade: u8 = if (x + y) % 5 == 0 { 18 } else { 0 };
                data[index..index + 4].copy_from_slice(&[
                    color[0].saturating_add(shade),
                    color[1].saturating_add(shade),
                    color[2].saturating_add(shade),
                    color[3],
                ]);
            }
        }
    }

    Image::new_fill(
        bevy::render::render_resource::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        bevy::render::render_resource::TextureDimension::D2,
        &data,
        bevy::render::render_resource::TextureFormat::Rgba8UnormSrgb,
        bevy::asset::RenderAssetUsages::default(),
    )
}

fn apply_mesh_part(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    record: &mut ChunkRecord,
    opaque_mesh: Option<Mesh>,
    cutout_mesh: Option<Mesh>,
    materials: &MaterialState,
) {
    update_mesh_child(
        commands,
        meshes,
        record.entity,
        &mut record.opaque_mesh_entity,
        &mut record.opaque_mesh,
        opaque_mesh,
        materials.opaque_material.clone(),
        "Opaque",
    );
    update_mesh_child(
        commands,
        meshes,
        record.entity,
        &mut record.cutout_mesh_entity,
        &mut record.cutout_mesh,
        cutout_mesh,
        materials.cutout_material.clone(),
        "Cutout",
    );
}

fn release_chunk_mesh_assets(record: &mut ChunkRecord, meshes: &mut Assets<Mesh>) {
    if let Some(handle) = record.opaque_mesh.take() {
        meshes.remove(handle.id());
    }
    if let Some(handle) = record.cutout_mesh.take() {
        meshes.remove(handle.id());
    }
}

fn mark_neighbor_generation_dependencies_dirty(
    chunk: IVec3,
    runtime: &mut RuntimeState,
) -> Vec<(Entity, ChunkStatus)> {
    let mut updated = Vec::new();
    for neighbor in face_neighbor_offsets() {
        let Some(record) = runtime.chunks.get_mut(&(chunk + neighbor)) else {
            continue;
        };
        if record.data.is_none() {
            continue;
        }
        mark_chunk_for_remesh(record, i32::MAX - 2);
        updated.push((record.entity, record.status.clone()));
    }
    updated
}

fn mark_chunk_for_remesh(record: &mut ChunkRecord, priority: i32) {
    if matches!(
        record.status.lifecycle,
        ChunkLifecycle::Generating | ChunkLifecycle::Requested | ChunkLifecycle::Unloading
    ) {
        return;
    }
    if record.meshing_task.is_some() {
        record.pending_remesh = true;
    }
    record.status.lifecycle = ChunkLifecycle::Dirty;
    record.priority = record.priority.max(priority);
}

fn face_neighbor_offsets() -> [IVec3; 6] {
    [
        IVec3::X,
        IVec3::NEG_X,
        IVec3::Y,
        IVec3::NEG_Y,
        IVec3::Z,
        IVec3::NEG_Z,
    ]
}

#[allow(clippy::too_many_arguments)]
fn update_mesh_child(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    parent: Entity,
    child_entity: &mut Option<Entity>,
    child_mesh: &mut Option<Handle<Mesh>>,
    mesh: Option<Mesh>,
    material: Handle<StandardMaterial>,
    label: &str,
) {
    match mesh {
        Some(mesh) => {
            let handle = if let Some(handle) = child_mesh.clone() {
                let _ = meshes.insert(handle.id(), mesh);
                handle
            } else {
                let handle = meshes.add(mesh);
                *child_mesh = Some(handle.clone());
                handle
            };

            if let Some(entity) = child_entity {
                commands.entity(*entity).insert(Mesh3d(handle));
            } else {
                let entity = commands
                    .spawn((
                        Name::new(format!("Voxel Chunk {label}")),
                        Mesh3d(handle),
                        MeshMaterial3d(material),
                    ))
                    .id();
                commands.entity(entity).set_parent_in_place(parent);
                *child_entity = Some(entity);
            }
        }
        None => {
            if let Some(entity) = child_entity.take() {
                commands.entity(entity).despawn();
            }
            if let Some(handle) = child_mesh.take() {
                meshes.remove(handle.id());
            }
        }
    }
}

fn persist_dirty_chunks(
    time: Res<Time>,
    config: Res<VoxelWorldConfig>,
    mut runtime: ResMut<RuntimeState>,
) {
    if config.save_policy.mode == SaveMode::Disabled {
        return;
    }
    runtime.last_save_tick += time.delta_secs();
    if runtime.last_save_tick < config.save_policy.autosave_interval_seconds {
        return;
    }
    runtime.last_save_tick = 0.0;

    let mut saved = 0_u32;
    for (chunk, record) in &mut runtime.chunks {
        if !record.status.dirty {
            continue;
        }
        if saved >= config.save_policy.max_chunks_per_frame {
            break;
        }
        if save_chunk_delta(
            &config.save_policy,
            *chunk,
            config.seed,
            config.generator_version,
            record.status.version,
            &record.overrides,
        )
        .is_ok()
        {
            record.status.dirty = false;
            record.status.persisted_version = record.status.version;
            if record.status.lifecycle == ChunkLifecycle::Meshed {
                record.status.lifecycle = ChunkLifecycle::Persisted;
            }
            saved += 1;
        }
    }
}

fn refresh_runtime_stats(runtime: Res<RuntimeState>, mut stats: ResMut<VoxelWorldStats>) {
    stats.loaded_chunks = runtime.chunks.len();
    stats.pending_generation_jobs = runtime
        .chunks
        .values()
        .filter(|record| record.generation_task.is_some())
        .count();
    stats.pending_meshing_jobs = runtime
        .chunks
        .values()
        .filter(|record| record.meshing_task.is_some())
        .count();
    stats.dirty_chunks = runtime
        .chunks
        .values()
        .filter(|record| record.status.dirty || record.status.lifecycle == ChunkLifecycle::Dirty)
        .count();
    stats.meshed_chunks = runtime
        .chunks
        .values()
        .filter(|record| record.opaque_mesh_entity.is_some() || record.cutout_mesh_entity.is_some())
        .count();
    stats.pending_save_chunks = runtime
        .chunks
        .values()
        .filter(|record| record.status.dirty)
        .count();
}

fn update_debug_gizmos(
    config: Res<VoxelWorldConfig>,
    debug: Res<VoxelDebugConfig>,
    runtime: Res<RuntimeState>,
    viewers: Query<&GlobalTransform, With<ChunkViewer>>,
    mut gizmos: Gizmos,
) {
    if debug.show_viewer_radii {
        for transform in &viewers {
            gizmos.sphere(
                Isometry3d::from_translation(transform.translation()),
                config.request_radius as f32 * config.chunk_dims.x as f32,
                Color::srgba(0.29, 0.71, 0.96, 0.2),
            );
            gizmos.sphere(
                Isometry3d::from_translation(transform.translation()),
                config.keep_radius as f32 * config.chunk_dims.x as f32,
                Color::srgba(0.96, 0.73, 0.21, 0.2),
            );
        }
    }

    if !debug.show_chunk_bounds {
        return;
    }

    let chunk_size = config.chunk_dims.as_vec3();
    for (chunk, record) in &runtime.chunks {
        let color = match debug.color_mode {
            VoxelDebugColorMode::ByLifecycle => match record.status.lifecycle {
                ChunkLifecycle::Requested => Color::srgb(0.27, 0.36, 0.98),
                ChunkLifecycle::Generating => Color::srgb(0.42, 0.72, 0.95),
                ChunkLifecycle::Generated => Color::srgb(0.40, 0.83, 0.58),
                ChunkLifecycle::Meshing => Color::srgb(0.92, 0.63, 0.23),
                ChunkLifecycle::Meshed => Color::srgb(0.76, 0.85, 0.39),
                ChunkLifecycle::Dirty => Color::srgb(0.97, 0.38, 0.36),
                ChunkLifecycle::Persisted => Color::srgb(0.62, 0.79, 0.88),
                ChunkLifecycle::Unloading => Color::srgb(0.67, 0.51, 0.85),
            },
            VoxelDebugColorMode::ByDirty => {
                if record.status.dirty {
                    Color::srgb(0.96, 0.31, 0.31)
                } else {
                    Color::srgb(0.28, 0.82, 0.56)
                }
            }
        };
        gizmos.cube(
            Transform::from_translation(
                chunk_translation(*chunk, config.chunk_dims) + chunk_size * 0.5,
            )
            .with_scale(chunk_size),
            color,
        );
    }
}
