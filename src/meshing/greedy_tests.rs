use bevy::prelude::*;

use super::{MeshBuffers, MeshCounts, PaddedChunk, SampledBlock, greedy::emit_greedy_quads};
use crate::{BlockId, BlockRegistry, ChunkData, VoxelWorldConfig};

fn padded_from_center(center: &ChunkData) -> PaddedChunk {
    let mut padded = PaddedChunk::new_unknown(center.dims());
    for z in 0..center.dims().z as i32 {
        for y in 0..center.dims().y as i32 {
            for x in 0..center.dims().x as i32 {
                padded.set(
                    IVec3::new(x + 1, y + 1, z + 1),
                    SampledBlock {
                        id: center.get(UVec3::new(x as u32, y as u32, z as u32)),
                        known: true,
                    },
                );
            }
        }
    }
    padded
}

#[test]
fn isolated_block_emits_six_quads() {
    let mut chunk = ChunkData::new_filled(UVec3::splat(2), BlockId::AIR);
    chunk.set(UVec3::new(0, 0, 0), BlockId::STONE);
    let padded = padded_from_center(&chunk);
    let mut buffers = MeshBuffers::default();
    let mut counts = MeshCounts::default();
    emit_greedy_quads(
        IVec3::ZERO,
        &chunk,
        &padded,
        &BlockRegistry::default(),
        &VoxelWorldConfig::default(),
        &mut buffers,
        &mut counts,
    );
    assert_eq!(counts.opaque_quads, 6);
}

#[test]
fn full_solid_chunk_only_emits_exterior_faces() {
    let chunk = ChunkData::new_filled(UVec3::splat(2), BlockId::STONE);
    let padded = padded_from_center(&chunk);
    let mut buffers = MeshBuffers::default();
    let mut counts = MeshCounts::default();
    emit_greedy_quads(
        IVec3::ZERO,
        &chunk,
        &padded,
        &BlockRegistry::default(),
        &VoxelWorldConfig::default(),
        &mut buffers,
        &mut counts,
    );
    assert_eq!(counts.opaque_quads, 6);
}

#[test]
fn unknown_neighbor_keeps_boundary_face_visible() {
    let chunk = ChunkData::new_filled(UVec3::splat(1), BlockId::STONE);
    let padded = padded_from_center(&chunk);
    let mut buffers = MeshBuffers::default();
    let mut counts = MeshCounts::default();
    emit_greedy_quads(
        IVec3::ZERO,
        &chunk,
        &padded,
        &BlockRegistry::default(),
        &VoxelWorldConfig::default(),
        &mut buffers,
        &mut counts,
    );
    assert_eq!(counts.opaque_quads, 6);
}

#[test]
fn solid_neighbor_culls_boundary_face_when_known() {
    let chunk = ChunkData::new_filled(UVec3::splat(1), BlockId::STONE);
    let mut padded = padded_from_center(&chunk);
    padded.set(
        IVec3::new(2, 1, 1),
        SampledBlock {
            id: BlockId::STONE,
            known: true,
        },
    );
    let mut buffers = MeshBuffers::default();
    let mut counts = MeshCounts::default();
    emit_greedy_quads(
        IVec3::ZERO,
        &chunk,
        &padded,
        &BlockRegistry::default(),
        &VoxelWorldConfig::default(),
        &mut buffers,
        &mut counts,
    );
    assert_eq!(counts.opaque_quads, 5);
}

#[test]
fn transparent_or_cross_neighbor_does_not_cull_opaque_face() {
    for neighbor in [BlockId::WATER, BlockId::TALL_GRASS] {
        let chunk = ChunkData::new_filled(UVec3::splat(1), BlockId::STONE);
        let mut padded = padded_from_center(&chunk);
        padded.set(
            IVec3::new(2, 1, 1),
            SampledBlock {
                id: neighbor,
                known: true,
            },
        );
        let mut buffers = MeshBuffers::default();
        let mut counts = MeshCounts::default();
        emit_greedy_quads(
            IVec3::ZERO,
            &chunk,
            &padded,
            &BlockRegistry::default(),
            &VoxelWorldConfig::default(),
            &mut buffers,
            &mut counts,
        );
        assert_eq!(
            counts.opaque_quads, 6,
            "neighbor {neighbor:?} should not cull"
        );
    }
}

#[test]
fn disabling_greedy_emits_face_by_face_quads() {
    let dims = UVec3::new(4, 1, 4);
    let chunk = ChunkData::new_filled(dims, BlockId::STONE);
    let padded = padded_from_center(&chunk);

    let mut greedy_buffers = MeshBuffers::default();
    let mut greedy_counts = MeshCounts::default();
    emit_greedy_quads(
        IVec3::ZERO,
        &chunk,
        &padded,
        &BlockRegistry::default(),
        &VoxelWorldConfig::default(),
        &mut greedy_buffers,
        &mut greedy_counts,
    );

    let config = VoxelWorldConfig {
        chunk_dims: dims,
        meshing: crate::config::MeshingConfig {
            enable_greedy: false,
            ..Default::default()
        },
        ..Default::default()
    };
    let mut face_buffers = MeshBuffers::default();
    let mut face_counts = MeshCounts::default();
    emit_greedy_quads(
        IVec3::ZERO,
        &chunk,
        &padded,
        &BlockRegistry::default(),
        &config,
        &mut face_buffers,
        &mut face_counts,
    );

    assert_eq!(greedy_counts.opaque_quads, 6);
    assert!(face_counts.opaque_quads > greedy_counts.opaque_quads);
}

#[test]
fn checkerboard_pattern_resists_large_merges() {
    let dims = UVec3::new(4, 1, 4);
    let mut chunk = ChunkData::new_filled(dims, BlockId::AIR);
    for z in 0..dims.z {
        for x in 0..dims.x {
            let block = if (x + z) % 2 == 0 {
                BlockId::STONE
            } else {
                BlockId::DIRT
            };
            chunk.set(UVec3::new(x, 0, z), block);
        }
    }
    let padded = padded_from_center(&chunk);
    let mut buffers = MeshBuffers::default();
    let mut counts = MeshCounts::default();
    emit_greedy_quads(
        IVec3::ZERO,
        &chunk,
        &padded,
        &BlockRegistry::default(),
        &VoxelWorldConfig::default(),
        &mut buffers,
        &mut counts,
    );
    assert!(counts.opaque_quads > 6);
}

#[test]
fn ao_mismatch_breaks_greedy_merge() {
    let dims = UVec3::new(2, 1, 1);
    let chunk = ChunkData::new_filled(dims, BlockId::STONE);

    let flat_padded = padded_from_center(&chunk);
    let mut flat_buffers = MeshBuffers::default();
    let mut flat_counts = MeshCounts::default();
    let config = VoxelWorldConfig { chunk_dims: dims, ..Default::default() };
    emit_greedy_quads(
        IVec3::ZERO,
        &chunk,
        &flat_padded,
        &BlockRegistry::default(),
        &config,
        &mut flat_buffers,
        &mut flat_counts,
    );

    let mut ao_padded = padded_from_center(&chunk);
    ao_padded.set(
        IVec3::new(3, 2, 1),
        SampledBlock {
            id: BlockId::STONE,
            known: true,
        },
    );
    let mut ao_buffers = MeshBuffers::default();
    let mut ao_counts = MeshCounts::default();
    emit_greedy_quads(
        IVec3::ZERO,
        &chunk,
        &ao_padded,
        &BlockRegistry::default(),
        &config,
        &mut ao_buffers,
        &mut ao_counts,
    );

    assert!(ao_counts.opaque_quads > flat_counts.opaque_quads);
}

#[test]
fn disabling_ao_ignores_ao_based_merge_splits() {
    let dims = UVec3::new(2, 1, 1);
    let chunk = ChunkData::new_filled(dims, BlockId::STONE);
    let mut padded = padded_from_center(&chunk);
    padded.set(
        IVec3::new(3, 2, 1),
        SampledBlock {
            id: BlockId::STONE,
            known: true,
        },
    );

    let ao_config = VoxelWorldConfig { chunk_dims: dims, ..Default::default() };
    let mut ao_buffers = MeshBuffers::default();
    let mut ao_counts = MeshCounts::default();
    emit_greedy_quads(
        IVec3::ZERO,
        &chunk,
        &padded,
        &BlockRegistry::default(),
        &ao_config,
        &mut ao_buffers,
        &mut ao_counts,
    );

    let mut no_ao_config = ao_config.clone();
    no_ao_config.meshing.ambient_occlusion = false;
    let mut no_ao_buffers = MeshBuffers::default();
    let mut no_ao_counts = MeshCounts::default();
    emit_greedy_quads(
        IVec3::ZERO,
        &chunk,
        &padded,
        &BlockRegistry::default(),
        &no_ao_config,
        &mut no_ao_buffers,
        &mut no_ao_counts,
    );

    assert!(ao_counts.opaque_quads > no_ao_counts.opaque_quads);
}
