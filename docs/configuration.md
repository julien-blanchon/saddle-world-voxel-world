# `saddle-world-voxel-world` Configuration

This file documents the public tuning surface of the crate. `VoxelWorldConfig` covers streaming, meshing, lighting, persistence, and atlas behavior. Block palettes and world generation live in separate resources so the core crate stays generic.

## `VoxelWorldConfig`

| Field | Type | Default | Range / Guidance | Effect |
| --- | --- | --- | --- | --- |
| `chunk_dims` | `UVec3` | `16, 16, 16` | Keep every axis at least `4`. Powers of two are easiest to reason about. | Dimensions of the contiguous block storage, meshing neighborhood, and chunk/world coordinate math. |
| `request_radius` | `u32` | `6` | Usually smaller than or equal to `keep_radius`. | Radius that causes chunks to be requested around each viewer. |
| `keep_radius` | `u32` | `8` | Must be `>= request_radius`. | Hysteresis radius. Chunks stay resident until they leave this larger radius. |
| `max_chunk_requests_per_frame` | `u32` | `12` | Raise for faster catch-up, lower for smoother CPU spikes. | Budget for chunk entity creation and request promotion per frame. |
| `max_chunk_unloads_per_frame` | `u32` | `8` | Raise for faster cleanup, lower for less despawn churn. | Budget for chunk unloads per frame. |
| `max_generation_jobs_in_flight` | `usize` | `4` | Must be `> 0`. Tune against CPU cores and sampler cost. | Async generation concurrency cap. |
| `max_mesh_jobs_in_flight` | `usize` | `4` | Must be `> 0`. Tune against chunk density and remesh frequency. | Async meshing concurrency cap. |
| `seed` | `u64` | `1` | Any value is valid. Keep fixed for deterministic saves and tests. | Primary deterministic world seed. |
| `generator_version` | `u32` | `1` | Bump when sampler or decoration meaning changes incompatibly. | Save compatibility gate for region-delta loading. |
| `save_policy` | `SavePolicy` | disabled defaults | See below. | Persistence root, batching, and region layout. |
| `meshing` | `MeshingConfig` | greedy + AO defaults | See below. | Controls greedy meshing, AO, and unknown-neighbor face policy. |
| `lighting` | `LightingConfig` | baked AO defaults | See below. | Controls baked flood-fill lighting and brightness normalization. |
| `atlas` | `AtlasConfig` | generated 4x3 debug atlas | See below. | Texture layout, UV inset, and optional asset-backed atlas. |

## Generation Resources

Generation is configured outside `VoxelWorldConfig` through two resources:

- `BlockRegistry`: maps `BlockId` values to rendering, collision, and atlas behavior.
- `VoxelWorldGenerator`: owns one base `VoxelBlockSampler` plus zero or more `VoxelDecorationHook`s.

### `BlockRegistry`

The core default registry is intentionally generic. It exists so the crate can run and test itself without assuming a biome or art style.

Default IDs:

- `AIR`
- `SOLID`
- `SOLID_ALT`
- `SOLID_ACCENT`
- `NON_SOLID`
- `CROSS`
- `EMISSIVE`
- `CUTOUT_SOLID`

Real projects should usually insert a custom `BlockRegistry` before adding `VoxelWorldPlugin`.
Sparse registries are supported: undefined IDs are treated as missing for validation and defensively resolve to `AIR` when read through `get()`.

### `VoxelWorldGenerator`

`VoxelWorldGenerator` is built in code, not via reflection tables:

- `VoxelWorldGenerator::new(sampler)` sets the base sampler.
- `.with_decoration(hook)` appends a decoration stage.
- decoration hooks run in insertion order and can replace the current sampled block.

This separation is intentional:

- streaming, meshing, persistence, and diagnostics stay independent of one world preset
- games can swap in authored terrain, biome samplers, structure passes, or editor-fed chunk sources without changing plugin setup
- examples and labs can carry richer presets without making them the crate contract

### `FlatBlockSampler`

The built-in fallback sampler is a generic flat plane.

| Field | Type | Default | Guidance | Effect |
| --- | --- | --- | --- | --- |
| `surface_y` | `i32` | `0` | Raise or lower the plane. | Height of the visible surface layer. |
| `surface_block` | `BlockId` | `SOLID_ALT` | Choose any registry ID. | Block used exactly on the surface plane. |
| `fill_block` | `BlockId` | `SOLID` | Choose any solid or non-solid ID. | Block used below `surface_y`. |
| `empty_block` | `BlockId` | `AIR` | Usually leave as `AIR`. | Block used above `surface_y`. |

Use `FlatBlockSampler` for tests, debug flyovers, or as a starting point before inserting a custom sampler implementation.

## `SavePolicy`

| Field | Type | Default | Range / Guidance | Effect |
| --- | --- | --- | --- | --- |
| `mode` | `SaveMode` | `Disabled` | `DeltaRegions` enables disk persistence. | Chooses whether chunk deltas are written at all. |
| `root` | `String` | `"local/voxel_world"` | Use a per-world or per-profile directory in real games. | Directory that stores region files. |
| `region_dims` | `IVec3` | `8, 8, 8` | Keep every axis positive. Larger regions reduce file count; smaller regions reduce rewrite scope. | Chunk-space dimensions grouped into one region file. |
| `autosave_interval_seconds` | `f32` | `10.0` | `0.0` effectively saves every frame budget window. | Minimum real-time interval between autosave sweeps. |
| `max_chunks_per_frame` | `u32` | `2` | Raise for faster persistence, lower for steadier frame time. | Save batching cap during one autosave sweep. |

## `MeshingConfig`

| Field | Type | Default | Range / Guidance | Effect |
| --- | --- | --- | --- | --- |
| `enable_greedy` | `bool` | `true` | Leave enabled for normal runtime use. | Chooses greedy cube meshing instead of face-by-face output. |
| `ambient_occlusion` | `bool` | `true` | Disable for debugging vertex lighting or meshing seams. | Enables per-vertex AO sampling during mesh extraction. |
| `ao_strength` | `f32` | `0.78` | Usually `0.0..=1.0`. | Multiplier applied to AO-darkened vertex colors. |
| `render_faces_against_unknown_neighbors` | `bool` | `true` | Conservative default for streaming worlds. | Keeps boundary faces visible until neighbor chunk truth exists. |

## `LightingConfig`

| Field | Type | Default | Range / Guidance | Effect |
| --- | --- | --- | --- | --- |
| `baked_ao` | `bool` | `true` | Disable only when comparing AO-off geometry. | Public toggle for the current AO lighting stage. |
| `flood_fill` | `bool` | `true` | Disable when you want flat, atlas-only debug shading. | Enables chunk-local skylight and emissive flood-fill lighting during mesh extraction. |
| `max_light_level` | `u8` | `15` | Must stay `> 0`. Higher values preserve more steps before falloff reaches zero. | Maximum scalar light intensity used for normalization and propagation clamps. |
| `sky_light_level` | `u8` | `15` | Must stay `<= max_light_level`. | Seed intensity used for open-to-sky cells at the top of the padded chunk. |
| `light_falloff` | `u8` | `1` | Must stay `> 0`. Higher values make lighting decay more aggressively per step. | Per-neighbor attenuation for both skylight and emissive propagation. |
| `minimum_brightness` | `f32` | `0.18` | Keep in `0.0..=1.0`. | Floor applied when converting propagated light into vertex-color brightness so enclosed spaces never become fully black unless you choose to. |

## `AtlasConfig`

| Field | Type | Default | Range / Guidance | Effect |
| --- | --- | --- | --- | --- |
| `asset_path` | `Option<String>` | `None` | Provide a path when you want a real texture atlas. | If unset, the crate generates a small debug atlas image at runtime. |
| `columns` | `u16` | `4` | Must cover the highest atlas tile used by the active block registry. | Atlas grid width. |
| `rows` | `u16` | `3` | Must cover the highest atlas tile used by the active block registry. | Atlas grid height. |
| `tile_size` | `UVec2` | `16, 16` | Keep square unless your atlas content requires otherwise. | Used for generated debug atlas dimensions and UV computation. |
| `uv_inset` | `f32` | `0.02` | Small positive values reduce bleeding. | Shrinks UVs slightly inside the tile. |

## `ChunkViewerSettings`

Attach this component only when a viewer should override the global config.

| Field | Type | Default | Range / Guidance | Effect |
| --- | --- | --- | --- | --- |
| `request_radius` | `u32` | `0` | `0` means “use the global config radius” in practice. | Per-viewer request radius override. |
| `keep_radius` | `u32` | `0` | `0` means “use the global config radius” in practice. | Per-viewer keep radius override. |
| `priority` | `i32` | `0` | Higher values win when multiple viewers request the same chunk. | Per-viewer residency priority. |

## `VoxelDebugConfig`

| Field | Type | Default | Guidance | Effect |
| --- | --- | --- | --- | --- |
| `show_chunk_bounds` | `bool` | `false` | Enable in labs, BRP sessions, and streaming debugging. | Draws chunk boundary gizmos. |
| `show_viewer_radii` | `bool` | `false` | Useful when tuning request/keep hysteresis. | Draws viewer radius gizmos. |
| `show_raycast` | `bool` | `false` | Reserved for later ray debug rendering. | Public flag for edit-target / ray debug overlays. |
| `color_mode` | `VoxelDebugColorMode` | `ByLifecycle` | `ByDirty` highlights dirty chunk churn. | Chooses debug-gizmo coloring semantics. |

## Tuning Recommendations

- Tight single-player flyover: `request_radius = 4`, `keep_radius = 6`, `max_generation_jobs_in_flight = 2..4`, `max_mesh_jobs_in_flight = 2..4`
- Edit-heavy prototype: keep `request_radius` modest, raise mesh concurrency slightly, and leave `render_faces_against_unknown_neighbors = true`
- Lighting showcase: leave `flood_fill = true`, keep `max_light_level` and `sky_light_level` aligned, and lower `minimum_brightness` if you want caves to read darker
- Split-screen or editor camera: use per-viewer `ChunkViewerSettings.priority` to keep the primary view responsive without disabling secondary viewers
- Persistence-heavy debug session: use smaller `region_dims` when you want saves localized around rapid edits, larger `region_dims` when you prefer fewer files
