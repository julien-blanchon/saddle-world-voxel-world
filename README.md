# Saddle World Voxel World

Reusable chunk-based voxel world streaming for Bevy. The crate owns chunk residency around one or more viewers, deterministic chunk generation through injected samplers, greedy plus cross meshing, block edits, delta persistence, raycast targeting, and BRP-friendly diagnostics.

It stays project-agnostic: no project-specific crates, no game-state assumptions, and no required project assets. Consumers wire schedules, choose how block IDs map to gameplay meaning, and decide whether chunk messages should drive audio, AI, UI, or save policies.

## Quick Start

```toml
[dependencies]
bevy = "0.18"
saddle-world-voxel-world = { git = "https://github.com/julien-blanchon/saddle-world-voxel-world" }
```

```rust
use bevy::prelude::*;
use saddle_world_voxel_world::{ChunkViewer, ChunkViewerSettings, VoxelWorldConfig, VoxelWorldPlugin};

#[derive(States, Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum DemoState {
    #[default]
    Running,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .init_state::<DemoState>()
        .insert_resource(VoxelWorldConfig::default())
        .add_plugins(VoxelWorldPlugin::new(
            OnEnter(DemoState::Running),
            OnExit(DemoState::Running),
            Update,
        ))
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn((
        Name::new("Viewer Camera"),
        Camera3d::default(),
        ChunkViewer,
        ChunkViewerSettings {
            request_radius: 4,
            keep_radius: 6,
            priority: 10,
        },
        Transform::from_xyz(24.0, 26.0, 24.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}
```

For examples, crate-local labs, and always-on tools, `VoxelWorldPlugin::default()` is the always-on entrypoint. It activates on `PostStartup`, never deactivates, and updates in `Update`.

Before adding the plugin, insert custom `BlockRegistry` and `VoxelWorldGenerator` resources when you want your own block palette or worldgen contract. If you do nothing, the crate falls back to a small generic debug registry plus a flat sampler. Custom registries may use sparse `BlockId` values; undefined IDs are rejected by `contains()` and defensively resolve to `AIR` when sampled through `get()`.

## Public API

- Plugin: `VoxelWorldPlugin::new(activate_schedule, deactivate_schedule, update_schedule)` or `VoxelWorldPlugin::always_on(update_schedule)`
- System sets: `VoxelWorldSystems::{Viewers, Streaming, Generation, Edits, Meshing, Collision, Lighting, Persistence, Diagnostics}`
- Components: `ChunkPos`, `ChunkStatus`, `ChunkViewer`, `ChunkViewerSettings`, `VoxelWorldRoot`
- Resources: `VoxelWorldConfig`, `VoxelWorldStats`, `VoxelDebugConfig`, `BlockRegistry`, `VoxelWorldGenerator`
- Read-only access: `VoxelWorldView`
- Edit path: `VoxelCommand::{SetBlock, Batch}` plus `BlockEdit`
- Messages: `ChunkLoaded`, `ChunkUnloaded`, `BlockModified`
- Core data types: `BlockId` (`AIR`, `SOLID`, `SOLID_ALT`, `SOLID_ACCENT`, `NON_SOLID`, `CROSS`, `EMISSIVE`, `CUTOUT_SOLID`), `BlockDefinition`, `BlockFaceAtlas`, `ChunkData`, `ChunkLifecycle`
- Generation hooks: `VoxelBlockSampler`, `VoxelDecorationHook`, `FlatBlockSampler`
- Helpers: coordinate conversions, `generate_chunk`, `sample_generated_block`, `raycast_blocks`, `save_chunk_delta`, `load_chunk_delta`, `encode_rle_blocks`, `decode_rle_blocks`

## Runtime Model

`saddle-world-voxel-world` separates algorithmic truth from ECS glue.

- Pure Rust core: chunk storage, coordinate math, seeded generation, greedy meshing, cross meshing, AO sampling, ray traversal, RLE helpers, and region-delta encoding.
- ECS/runtime glue: viewer tracking, chunk entities, async job dispatch, mesh asset ownership, messages, debug resources, and gizmos.

Viewer-driven residency is the core contract:

- spawn one or more entities with `ChunkViewer`
- optionally override radii and priority with `ChunkViewerSettings`
- the runtime streams the union of requested chunks and keeps chunks around until they leave the configured keep radius

Edits go through one message-based mutation path:

- send `VoxelCommand::SetBlock` or `VoxelCommand::Batch`
- the runtime mutates the owning chunk
- boundary edits mark neighbor chunks dirty
- meshing is re-queued only for the affected chunk set
- `BlockModified` is emitted for downstream systems

## Generation Contract

The core runtime no longer hardcodes one biome or palette. Generation is driven by a separate `VoxelWorldGenerator` resource:

- one `VoxelBlockSampler` decides the base block for any world-space voxel
- zero or more `VoxelDecorationHook`s can override that base sample for sparse features such as foliage, lights, props, or structures
- `generate_chunk` and `sample_generated_block` stay deterministic because they read only `world_pos`, `VoxelWorldConfig`, and the injected generator resource
- `generator_version` remains the save-compatibility knob when sampler or decoration meaning changes incompatibly

The built-in fallback is a simple flat sampler using the generic debug registry. The richer rolling-terrain showcase now lives only in example support and the lab app.

## Rendering Notes

- Opaque cube faces use greedy meshing for efficient draw calls.
- Cutout cube blocks use per-face emission with alpha-mask rendering.
- Foliage-style blocks use a separate cross-mesh path with cutout material handling.
- Monochrome skylight plus emissive flood-fill lighting are baked into vertex colors during mesh build.
- AO is multiplied into that same vertex-color lighting stage during mesh build.
- If `AtlasConfig::asset_path` is unset, the crate generates a 4x3 debug atlas at runtime so examples stay asset-free.
- Empty chunks skip mesh asset allocation entirely.

## Persistence

The shipped save path is intentionally conservative:

- `SaveMode::Disabled` keeps the runtime purely ephemeral
- `SaveMode::DeltaRegions` writes per-region files that store only edited local indices and block IDs
- each region file is versioned by world seed and generator version
- RLE helpers exist for dense chunk snapshots or future codecs, but the current region format stores edit deltas directly

More detail lives in [docs/persistence.md](docs/persistence.md).

## Examples

| Example | Purpose | Run |
| --- | --- | --- |
| `basic` | Minimal flyover using the optional showcase preset from example support | `cargo run -p saddle-world-voxel-world-example-basic` |
| `block_editing` | Manual block edits and remeshing near chunk boundaries | `cargo run -p saddle-world-voxel-world-example-block-editing` |
| `multi_viewer` | Union streaming across two viewers with different priorities | `cargo run -p saddle-world-voxel-world-example-multi-viewer` |
| `persistence` | Manual save-stamped edits with delta-region persistence | `cargo run -p saddle-world-voxel-world-example-persistence` |
| `debug_gizmos` | Chunk bounds and viewer radii visualization | `cargo run -p saddle-world-voxel-world-example-debug-gizmos` |
| `mini_minecraft` | Playable FPS voxel sandbox layered on the optional showcase preset | `cargo run -p saddle-world-voxel-world-example-mini-minecraft` |

The richer validation app lives in [`examples/lab`](examples/lab/README.md).

## Crate-Local Lab

```bash
cargo run -p saddle-world-voxel-world-lab
```

The lab adds a debug overlay, BRP wiring, `orbit_camera`-based navigation, viewer choreography, and focused E2E scenarios without expanding the runtime crate surface. It intentionally opts into the example-support showcase preset so the core crate can stay generic.

Lab controls:

- `LMB`: orbit
- `MMB`: pan
- mouse wheel: zoom
- `Space`: toggle the secondary viewer
- `E`: fire a boundary edit burst
- `R`: reset the primary camera

E2E verification commands:

```bash
cargo run -p saddle-world-voxel-world-lab --features e2e -- voxel_smoke_launch
cargo run -p saddle-world-voxel-world-lab --features e2e -- voxel_example_basic
cargo run -p saddle-world-voxel-world-lab --features e2e -- voxel_example_debug_gizmos
cargo run -p saddle-world-voxel-world-lab --features e2e -- voxel_example_block_editing
cargo run -p saddle-world-voxel-world-lab --features e2e -- voxel_example_multi_viewer
cargo run -p saddle-world-voxel-world-lab --features e2e -- voxel_example_persistence
cargo run -p saddle-world-voxel-world-lab --features e2e -- voxel_streaming_motion
cargo run -p saddle-world-voxel-world-example-mini-minecraft --features e2e -- mini_minecraft_interaction
```

## BRP

Useful BRP commands against the lab:

```bash
uv run --project .codex/skills/bevy-brp/script brp app launch saddle-world-voxel-world-lab
uv run --project .codex/skills/bevy-brp/script brp world query saddle_world_voxel_world::ChunkPos
uv run --project .codex/skills/bevy-brp/script brp resource get saddle_world_voxel_world::VoxelWorldStats
uv run --project .codex/skills/bevy-brp/script brp resource get saddle_world_voxel_world::VoxelWorldConfig
uv run --project .codex/skills/bevy-brp/script brp extras screenshot /tmp/saddle-world-voxel-world-lab.png
uv run --project .codex/skills/bevy-brp/script brp extras shutdown
```

## Benchmarks

```bash
cargo bench -p saddle-world-voxel-world --bench voxel_world_benches
```

The bench target covers meshing, terrain generation, RLE encode/decode, and voxel ray traversal on representative chunk patterns.

## Known Limits And Non-Goals

- The shipped lighting model is scalar skylight plus emissive flood fill. There is no RGB light transport, sunlight directionality, or persistent light volumes yet.
- Collision mesh extraction is intentionally deferred; the block registry exposes collision intent, but the crate does not bind any physics backend.
- The current persistence format stores sparse edit deltas, not full chunk snapshots.
- Atlas UVs on merged greedy quads use stretched face UVs instead of tiled repetition.
- No chunk LOD, clipmaps, or distant impostors are shipped in v1.

## More Docs

- [Architecture](docs/architecture.md)
- [Configuration](docs/configuration.md)
- [Persistence](docs/persistence.md)
