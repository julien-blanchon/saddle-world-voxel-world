# `saddle-world-voxel-world-lab`

Crate-local lab for the `saddle-world-voxel-world` shared crate. It provides:

- a runnable voxel-world app that opts into the example-support showcase preset
- `saddle-camera-orbit-camera`-driven manual navigation
- BRP-friendly runtime inspection
- focused E2E scenarios with screenshots and assertions

## Run

```bash
cargo run -p saddle-world-voxel-world-lab
```

Controls:

- `LMB`: orbit
- `MMB`: pan
- mouse wheel: zoom
- `Space`: toggle the secondary viewer
- `E`: apply a boundary edit burst
- `R`: reset the camera

## E2E

```bash
cargo run -p saddle-world-voxel-world-lab --features e2e -- voxel_smoke_launch
cargo run -p saddle-world-voxel-world-lab --features e2e -- voxel_terrain_generation
cargo run -p saddle-world-voxel-world-lab --features e2e -- voxel_streaming_motion
cargo run -p saddle-world-voxel-world-lab --features e2e -- voxel_block_editing
cargo run -p saddle-world-voxel-world-lab --features e2e -- voxel_multi_viewer
```

## BRP

```bash
uv run --project .codex/skills/bevy-brp/script brp app launch saddle-world-voxel-world-lab
uv run --project .codex/skills/bevy-brp/script brp world query saddle_world_voxel_world::ChunkPos
```
