use std::collections::BTreeMap;

use bevy::prelude::*;

use crate::block::BlockId;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect)]
#[reflect(Component)]
pub struct ChunkPos(pub IVec3);

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Reflect)]
pub enum ChunkLifecycle {
    #[default]
    Requested,
    Generating,
    Generated,
    Meshing,
    Meshed,
    Dirty,
    Persisted,
    Unloading,
}

#[derive(Component, Clone, Debug, Reflect)]
#[reflect(Component)]
pub struct ChunkStatus {
    pub lifecycle: ChunkLifecycle,
    pub dirty: bool,
    pub version: u64,
    pub persisted_version: u64,
}

impl Default for ChunkStatus {
    fn default() -> Self {
        Self {
            lifecycle: ChunkLifecycle::Requested,
            dirty: false,
            version: 0,
            persisted_version: 0,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ChunkData {
    dims: UVec3,
    blocks: Vec<BlockId>,
}

impl ChunkData {
    #[must_use]
    pub fn new_filled(dims: UVec3, block: BlockId) -> Self {
        let len = (dims.x * dims.y * dims.z) as usize;
        Self {
            dims,
            blocks: vec![block; len],
        }
    }

    #[must_use]
    pub fn dims(&self) -> UVec3 {
        self.dims
    }

    #[must_use]
    pub fn blocks(&self) -> &[BlockId] {
        &self.blocks
    }

    #[must_use]
    pub fn blocks_mut(&mut self) -> &mut [BlockId] {
        &mut self.blocks
    }

    #[must_use]
    pub fn index(&self, local: UVec3) -> usize {
        index_for(self.dims, local)
    }

    #[must_use]
    pub fn local_from_index(&self, index: u32) -> UVec3 {
        local_from_index(self.dims, index)
    }

    #[must_use]
    pub fn get(&self, local: UVec3) -> BlockId {
        self.blocks[self.index(local)]
    }

    pub fn set(&mut self, local: UVec3, block: BlockId) {
        let index = self.index(local);
        self.blocks[index] = block;
    }

    #[must_use]
    pub fn is_uniform(&self, block: BlockId) -> bool {
        self.blocks.iter().all(|current| *current == block)
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.is_uniform(BlockId::AIR)
    }
}

#[must_use]
pub fn index_for(dims: UVec3, local: UVec3) -> usize {
    debug_assert!(local.x < dims.x && local.y < dims.y && local.z < dims.z);
    (local.x + dims.x * (local.y + dims.y * local.z)) as usize
}

#[must_use]
pub fn local_from_index(dims: UVec3, index: u32) -> UVec3 {
    let plane = dims.x * dims.y;
    let z = index / plane;
    let rem = index % plane;
    let y = rem / dims.x;
    let x = rem % dims.x;
    UVec3::new(x, y, z)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ChunkEditDelta {
    pub local_index: u32,
    pub block: BlockId,
}

#[must_use]
pub fn delta_from_overrides(overrides: &BTreeMap<u32, BlockId>) -> Vec<ChunkEditDelta> {
    overrides
        .iter()
        .map(|(local_index, block)| ChunkEditDelta {
            local_index: *local_index,
            block: *block,
        })
        .collect()
}

#[cfg(test)]
#[path = "chunk_tests.rs"]
mod tests;
