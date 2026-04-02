# `saddle-world-voxel-world` Configuration

This file documents the public tuning surface of the crate. Defaults target a small rolling terrain showcase with one viewer, generated atlas colors, greedy opaque meshing, and persistence disabled.

## `VoxelWorldConfig`

| Field | Type | Default | Range / Guidance | Effect |
| --- | --- | --- | --- | --- |
| `chunk_dims` | `UVec3` | `16, 16, 16` | Keep every axis at least `4`. Powers of two are easiest to reason about. | Dimensions of the contiguous block storage, meshing neighborhood, and chunk/world coordinate math. |
| `request_radius` | `u32` | `6` | Usually smaller than or equal to `keep_radius`. | Radius that causes chunks to be requested around each viewer. |
| `keep_radius` | `u32` | `8` | Must be `>= request_radius`. | Hysteresis radius. Chunks stay resident until they leave this larger radius. |
| `max_chunk_requests_per_frame` | `u32` | `12` | Raise for faster catch-up, lower for smoother CPU spikes. | Budget for chunk entity creation and request promotion per frame. |
| `max_chunk_unloads_per_frame` | `u32` | `8` | Raise for faster cleanup, lower for less despawn churn. | Budget for chunk unloads per frame. |
| `max_generation_jobs_in_flight` | `usize` | `4` | Must be `> 0`. Tune against CPU cores and generator cost. | Async terrain generation concurrency cap. |
| `max_mesh_jobs_in_flight` | `usize` | `4` | Must be `> 0`. Tune against chunk density and remesh frequency. | Async meshing concurrency cap. |
| `seed` | `u64` | `1` | Any value is valid. Keep fixed for deterministic saves and tests. | Primary deterministic world seed. |
| `generator_version` | `u32` | `1` | Bump when generator meaning changes incompatibly. | Save compatibility gate for region-delta loading. |
| `save_policy` | `SavePolicy` | disabled defaults | See below. | Persistence root, batching, and region layout. |
| `terrain` | `TerrainConfig` | layered-noise defaults | See below. | Terrain generator selection and parameterization. |
| `meshing` | `MeshingConfig` | greedy + AO defaults | See below. | Controls greedy meshing, AO, and unknown-neighbor face policy. |
| `lighting` | `LightingConfig` | baked AO only | See below. | Current and future lighting-stage toggles. |
| `atlas` | `AtlasConfig` | generated 3x3 atlas | See below. | Texture layout, UV inset, and optional asset-backed atlas. |

## `SavePolicy`

| Field | Type | Default | Range / Guidance | Effect |
| --- | --- | --- | --- | --- |
| `mode` | `SaveMode` | `Disabled` | `DeltaRegions` enables disk persistence. | Chooses whether chunk deltas are written at all. |
| `root` | `String` | `"local/voxel_world"` | Use a per-world or per-profile directory in real games. | Directory that stores region files. |
| `region_dims` | `IVec3` | `8, 8, 8` | Keep every axis positive. Larger regions reduce file count; smaller regions reduce rewrite scope. | Chunk-space dimensions grouped into one region file. |
| `autosave_interval_seconds` | `f32` | `10.0` | `0.0` effectively saves every frame budget window. | Minimum real-time interval between autosave sweeps. |
| `max_chunks_per_frame` | `u32` | `2` | Raise for faster persistence, lower for steadier frame time. | Save batching cap during one autosave sweep. |

## `TerrainConfig`

| Field | Type | Default | Range / Guidance | Effect |
| --- | --- | --- | --- | --- |
| `generator` | `GeneratorKind` | `LayeredNoise` | `Flat` is useful for tests and debugging. | Selects the terrain sampling strategy. |
| `base_height` | `i32` | `14` | Scene-scale dependent. | Baseline terrain elevation before hills. |
| `height_amplitude` | `i32` | `18` | Raise for taller terrain; lower for flatter landscapes. | Peak contribution from the height noise field. |
| `height_frequency` | `f32` | `0.012` | Lower values produce broader hills, higher values produce tighter terrain. | XY frequency of the main height field. |
| `hill_octaves` | `u8` | `4` | `1` to `6` is a practical range. | Layer count for the 2D fBm height field. |
| `cave_frequency` | `f32` | `0.06` | Higher values make denser, smaller cave features. | XYZ frequency of the cave noise field. |
| `cave_threshold` | `f32` | `0.68` | Higher values carve fewer caves. | Threshold above which underground voxels become air. |
| `water_level` | `i32` | `10` | Keep coherent with `base_height`. | Y level used for water fill and shoreline sand selection. |
| `foliage_chance` | `f32` | `0.08` | Usually `0.0..=0.3`. | Chance factor for the simple tall-grass decoration pass. |
| `structure_version` | `u32` | `1` | Reserved for future structure passes. | Public version hook for future structure generation changes. |

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
| `future_flood_fill_enabled` | `bool` | `false` | Reserved for future evolution. | Placeholder public surface for a later flood-fill lighting pass. |

## `AtlasConfig`

| Field | Type | Default | Range / Guidance | Effect |
| --- | --- | --- | --- | --- |
| `asset_path` | `Option<String>` | `None` | Provide a path when you want a real texture atlas. | If unset, the crate generates a small debug atlas image at runtime. |
| `columns` | `u16` | `3` | Must cover the highest atlas tile used by the block registry. | Atlas grid width. |
| `rows` | `u16` | `3` | Must cover the highest atlas tile used by the block registry. | Atlas grid height. |
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
- Split-screen or editor camera: use per-viewer `ChunkViewerSettings.priority` to keep the primary view responsive without disabling secondary viewers
- Persistence-heavy debug session: use smaller `region_dims` when you want saves localized around rapid edits, larger `region_dims` when you prefer fewer files
