# Architecture

## Reference Decisions

This crate was designed from a small set of stable ideas rather than by mirroring any single reference implementation.

### What was kept

- From 0fps: greedy meshing as the default opaque path, AO-aware merge boundaries, and the conservative rule that an unknown neighbor is not the same thing as air.
- From Amanatides and Woo: DDA-style voxel traversal for block picking and edit targeting.
- From `block-mesh`: the separation between chunk truth and mesh extraction logic.
- From `bevy_voxel_world` and `rsmc`: Bevy-native viewer-driven chunk residency, async build queues, and practical runtime diagnostics.

### What was simplified

- No flood-fill lighting yet. AO is baked during meshing so the first shipped runtime still has depth cues and stable tests.
- No collision mesh stage yet. The block registry exposes collision intent, but physics backend binding stays outside the crate.
- No LOD or clipmap path yet. The runtime uses fixed-size chunks with hysteresis because that keeps persistence, edits, and diagnostics straightforward.
- Persistence is sparse delta-based instead of storing full chunk snapshots or sector tables.

### What was outdated for modern Bevy

- Older references assume pre-0.18 Bevy bundle names and event APIs. This crate uses Bevy 0.18-style `Message`s, schedule labels, current mesh/material APIs, and task-pool polling.
- Several older voxel examples couple runtime logic to one camera or one game state. This crate instead exposes injectable schedules and explicit viewer components.

### What conflicted with shared-crate rules

- Project-specific states, screen enums, and gameplay contracts were intentionally excluded.
- Third-party voxel storage and meshing types are not exposed in the public surface.
- The crate does not require project assets; it can generate a debug atlas on demand.

## Chunk Lifecycle

The internal runtime tracks chunk state explicitly even though the full state machine is not exposed as a public enum.

Conceptual flow:

1. absent
2. requested
3. generating
4. generated
5. meshing
6. meshed
7. dirty
8. persisted
9. unloading

Publicly visible lifecycle state is exposed through `ChunkStatus.lifecycle`.

Important behavior:

- chunks are created from viewer demand, not from camera assumptions
- generation is deterministic from `seed + chunk position + generator config`
- boundary edits mark neighbor chunks dirty
- unknown neighbor data keeps boundary faces visible until neighboring truth arrives
- empty chunks keep chunk entities and data but skip mesh asset allocation

## ECS Pipeline

`VoxelWorldSystems` defines the public scheduling contract:

1. `Viewers`
2. `Streaming`
3. `Generation`
4. `Edits`
5. `Meshing`
6. `Collision`
7. `Lighting`
8. `Persistence`
9. `Diagnostics`

Current runtime mapping:

- `Viewers`: gather `ChunkViewer` positions, apply per-viewer overrides, compute the desired chunk union, and update per-chunk priority
- `Streaming`: spawn new chunk entities and unload stale ones
- `Generation`: enqueue and integrate async generation tasks
- `Edits`: apply `VoxelCommand` mutations, mark chunks dirty, and emit `BlockModified`
- `Meshing`: enqueue and integrate greedy plus cross mesh jobs
- `Persistence`: autosave dirty chunk deltas when enabled
- `Diagnostics`: refresh `VoxelWorldStats`, update debug gizmos when that plugin is present

Generation integration also invalidates already-loaded face neighbors. Chunks that previously rendered conservative boundary faces against unknown neighbors are marked for remesh once adjacent chunk truth exists.

`Collision` and `Lighting` are currently reserved extension points. They are part of the public ordering surface so downstream crates can already order future systems against them without a breaking rename later.

## Chunk Residency Model

Viewer entities define desired chunk demand.

Per frame:

1. read all active `ChunkViewer` transforms
2. derive a chunk-space center from each viewer
3. compute requested chunks inside the request radius
4. compute retained chunks inside the keep radius
5. union all viewer demand
6. retain the strongest priority when multiple viewers request the same chunk
7. load nearest/strongest-first and unload only after the chunk leaves the keep radius

This gives the runtime:

- multiple viewer support
- hysteresis between request and keep radii
- a small priority hook for split-screen, editor, or spectator use cases

## Generation Pipeline

The current generation path is deterministic layered noise terrain.

For every local voxel in a chunk:

1. convert chunk-local coordinates to world-space integer coordinates
2. sample a 2D fBm height field
3. sample a 3D cave noise field
4. choose a block ID based on terrain height, water level, cave threshold, and a small decorative foliage/lamp pass

The public architecture still leaves room for alternate generators:

- `GeneratorKind::Flat`
- `GeneratorKind::LayeredNoise`
- future strategy-based or biome-aware generation without changing chunk/entity APIs

## Meshing Pipeline

Meshing is intentionally split into two render classes.

### Opaque cubes

- padded chunk samples give each face access to neighbor truth
- greedy meshing runs per axis slice
- merge keys include block ID, material class, tile, face normal, and AO pattern
- AO mismatch breaks a merge to avoid visible seams

### Cross meshes

- non-cube foliage-style blocks generate crossed quads
- they are emitted into a separate cutout mesh path
- they do not cull neighboring opaque cube faces by default

### Mesh ownership

- chunk mesh entities are children of the chunk entity
- mesh handles are replaced intentionally on remesh
- empty outputs remove old mesh children instead of leaving stale render state behind

## Edit Pipeline

All block mutation goes through `VoxelCommand`.

For each edit:

1. convert world position to `chunk + local`
2. validate the target block ID against the registry
3. create the chunk on demand if it does not exist yet
4. mutate the chunk’s contiguous block storage
5. update the chunk’s sparse override map
6. mark the chunk dirty and bump its version
7. mark boundary neighbors dirty when the edit touches a chunk edge
8. raise the edited chunk’s priority so remeshing wins over far-away streaming work
9. emit `BlockModified`

The runtime therefore remeshes only the affected chunk set instead of triggering a global rebuild.
Chunks whose data changed stay persistence-dirty until autosave completes, while already-loaded neighbors are marked for remesh without being queued for save.

## Async Job Flow

Generation and meshing both use `AsyncComputeTaskPool`.

Important constraints:

- in-flight generation jobs are bounded by `max_generation_jobs_in_flight`
- in-flight meshing jobs are bounded by `max_mesh_jobs_in_flight`
- the main thread owns ECS integration, mesh asset mutation, and entity lifetime
- worker tasks own deterministic data generation and mesh extraction only

This split keeps the algorithmic core testable as plain Rust and keeps ECS mutation on the main thread.

## Debugging And Runtime Diagnosis

The crate is designed to be inspectable in a live app.

- chunk entities are named
- `ChunkPos` and `ChunkStatus` are public components
- `VoxelWorldStats` exposes loaded, meshed, dirty, pending-job, save-queue, and timing counters
- `VoxelDebugConfig` controls chunk bounds and viewer radius gizmos
- `VoxelWorldConfig` and `VoxelWorldStats` are reflect-registered for BRP inspection

Useful diagnosis patterns:

- compare `ChunkPos` count against `VoxelWorldStats.loaded_chunks`
- watch `pending_generation_jobs` and `pending_meshing_jobs` for backlog growth
- inspect `BlockModified` counts while stress-testing edits
- use the crate-local lab screenshots to catch seam, churn, or remesh regressions
