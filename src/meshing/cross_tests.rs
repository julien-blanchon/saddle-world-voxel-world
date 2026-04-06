use bevy::prelude::*;

use super::{MeshBuffers, MeshCounts, cross::emit_cross_quads};
use crate::{BlockId, BlockRegistry, ChunkData, VoxelWorldConfig};

#[test]
fn cross_meshing_emits_two_quads_per_cross_block() {
    let mut chunk = ChunkData::new_filled(UVec3::splat(2), BlockId::AIR);
    chunk.set(UVec3::new(0, 0, 0), BlockId::CROSS);
    let mut buffers = MeshBuffers::default();
    let mut counts = MeshCounts::default();
    emit_cross_quads(
        IVec3::ZERO,
        &chunk,
        None,
        &BlockRegistry::default(),
        &VoxelWorldConfig::default(),
        &mut buffers,
        &mut counts,
    );
    assert_eq!(counts.cutout_quads, 2);
}

#[test]
fn separate_cross_blocks_do_not_merge() {
    let mut chunk = ChunkData::new_filled(UVec3::new(2, 1, 1), BlockId::AIR);
    chunk.set(UVec3::new(0, 0, 0), BlockId::CROSS);
    chunk.set(UVec3::new(1, 0, 0), BlockId::CROSS);
    let mut buffers = MeshBuffers::default();
    let mut counts = MeshCounts::default();
    emit_cross_quads(
        IVec3::ZERO,
        &chunk,
        None,
        &BlockRegistry::default(),
        &VoxelWorldConfig::default(),
        &mut buffers,
        &mut counts,
    );
    assert_eq!(counts.cutout_quads, 4);
}
