# Persistence

## Current Save Model

The shipped persistence path is intentionally conservative:

- generated terrain is reproducible from `seed + generator config`
- only edited voxels are persisted
- edits are grouped into region files
- save compatibility is checked with `seed` and `generator_version`

This keeps first-pass saves simple, deterministic, and cheap for mostly-procedural worlds.

## Modes

`SaveMode::Disabled`

- no disk reads
- no disk writes
- all chunks are regenerated from the seed on every run

`SaveMode::DeltaRegions`

- dirty chunk overrides are batched into region files
- load integrates the generator result first, then applies the sparse delta

## Region File Naming

Region files live under `SavePolicy::root` and use this naming scheme:

```text
region.<rx>.<ry>.<rz>.vwr
```

`rx`, `ry`, and `rz` are the chunk-space region coordinates computed with Euclidean division against `SavePolicy::region_dims`.

Example:

```text
local/voxel_world/region.0.-1.2.vwr
```

## Binary Layout

All integer fields are encoded in little-endian order.

### File Header

| Field | Type | Bytes | Notes |
| --- | --- | --- | --- |
| `magic` | `[u8; 8]` | `8` | Current value: `VXREGN02` |
| `seed` | `u64` | `8` | World seed compatibility gate |
| `generator_version` | `u32` | `4` | Generator compatibility gate |
| `entry_count` | `u32` | `4` | Number of chunk entries in the file |

### Chunk Entry

| Field | Type | Bytes | Notes |
| --- | --- | --- | --- |
| `entry_len` | `u32` | `4` | Payload byte length for this one chunk entry |
| `chunk_x` | `i32` | `4` | Absolute chunk coordinate |
| `chunk_y` | `i32` | `4` | Absolute chunk coordinate |
| `chunk_z` | `i32` | `4` | Absolute chunk coordinate |
| `chunk_version` | `u64` | `8` | Runtime version stamp used for future incremental-save evolution |
| `edit_count` | `u32` | `4` | Number of sparse edits in this chunk |

Each edit then appends:

| Field | Type | Bytes | Notes |
| --- | --- | --- | --- |
| `local_index` | `u32` | `4` | Flat local voxel index inside the chunk |
| `block_id` | `u16` | `2` | New block ID for that local voxel |

After the entry payload, the file stores:

| Field | Type | Bytes | Notes |
| --- | --- | --- | --- |
| `entry_checksum` | `u32` | `4` | FNV-1a checksum of the entry payload |

Chunk entries are sorted by `(x, y, z)` before write for stable output.

## Delta Semantics

The runtime stores overrides, not full chunk snapshots.

Load flow:

1. generate the base chunk from seed and config
2. locate the owning region file
3. verify file magic, seed, and generator version
4. apply any stored local-index overrides onto the generated chunk

Save flow:

1. watch chunk dirty/version state
2. collect the sparse override map for the chunk
3. convert overrides into sorted `local_index + block_id` deltas
4. replace or insert that chunk entry in the owning region file

This means unchanged generated chunks never need a full serialized copy.

## Versioning

Two public knobs control compatibility:

- `VoxelWorldConfig::seed`
- `VoxelWorldConfig::generator_version`

If either value differs from the save file header, the runtime ignores the file contents for that load and falls back to generated terrain.

Recommended policy:

- keep `seed` stable for one world/save slot
- bump `generator_version` when terrain meaning changes incompatibly, such as layer ordering, cave thresholds, or structure placement rules

## Corruption Handling

The runtime is intentionally fail-soft at the world level.

- a missing region file is treated as “no saved edits”
- a seed or generator mismatch is treated as “ignore this file”
- a corrupted region file does not stop world generation; the chunk falls back to generated terrain
- a corrupted V2 entry checksum skips only that one chunk entry while later entries in the same region file remain readable
- a truncated region file loads all intact entries that appear before the truncation point

Legacy note:

- the loader still accepts legacy `VXREGN01` files for compatibility, but those files do not have per-entry checksums

## Compression Boundary

The crate already ships RLE helpers:

- `encode_rle_blocks(&ChunkData) -> Vec<u8>`
- `decode_rle_blocks(dims, bytes) -> Result<ChunkData, String>`

Those helpers are not yet wired into the region format. They exist so a future save path can choose between:

- sparse edit deltas for mostly-generated chunks
- dense RLE chunk payloads for heavily edited or imported chunks
- alternate codecs without changing the edit pipeline itself

## Portability Notes

- coordinates are stored as signed `i32`
- local voxel references are stored as flat `u32` indices
- block IDs are stored as `u16`
- the format is explicitly little-endian
- region filenames encode coordinates as decimal strings, including negative values

These choices keep the current format easy to inspect and straightforward to reimplement in offline tools.

## Recommended Operations

- Use `SaveMode::Disabled` for throwaway labs and deterministic tests.
- Use `SaveMode::DeltaRegions` for block-editing prototypes, editors, and survival-style worlds.
- Prefer a per-save-slot `root` directory rather than sharing one persistence root across unrelated worlds.
- Keep `autosave_interval_seconds` above zero unless you are explicitly stress-testing persistence churn.
