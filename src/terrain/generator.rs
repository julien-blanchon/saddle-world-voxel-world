use std::sync::Arc;

use bevy::prelude::*;

use crate::{
    block::BlockId, chunk::ChunkData, config::VoxelWorldConfig, coordinates::local_to_world,
};

pub trait VoxelBlockSampler: Send + Sync + 'static {
    fn sample_block(&self, world_pos: IVec3, config: &VoxelWorldConfig) -> BlockId;
}

pub trait VoxelDecorationHook: Send + Sync + 'static {
    fn decorate_block(
        &self,
        world_pos: IVec3,
        sampled: BlockId,
        config: &VoxelWorldConfig,
    ) -> Option<BlockId>;
}

#[derive(Clone, Debug)]
pub struct FlatBlockSampler {
    pub surface_y: i32,
    pub surface_block: BlockId,
    pub fill_block: BlockId,
    pub empty_block: BlockId,
}

impl Default for FlatBlockSampler {
    fn default() -> Self {
        Self {
            surface_y: 0,
            surface_block: BlockId::SOLID_ALT,
            fill_block: BlockId::SOLID,
            empty_block: BlockId::AIR,
        }
    }
}

impl VoxelBlockSampler for FlatBlockSampler {
    fn sample_block(&self, world_pos: IVec3, _config: &VoxelWorldConfig) -> BlockId {
        if world_pos.y < self.surface_y {
            self.fill_block
        } else if world_pos.y == self.surface_y {
            self.surface_block
        } else {
            self.empty_block
        }
    }
}

#[derive(Resource, Clone)]
pub struct VoxelWorldGenerator {
    sampler: Arc<dyn VoxelBlockSampler>,
    decorations: Vec<Arc<dyn VoxelDecorationHook>>,
}

impl Default for VoxelWorldGenerator {
    fn default() -> Self {
        Self::new(FlatBlockSampler::default())
    }
}

impl VoxelWorldGenerator {
    #[must_use]
    pub fn new(sampler: impl VoxelBlockSampler) -> Self {
        Self {
            sampler: Arc::new(sampler),
            decorations: Vec::new(),
        }
    }

    #[must_use]
    pub fn with_sampler(mut self, sampler: impl VoxelBlockSampler) -> Self {
        self.sampler = Arc::new(sampler);
        self
    }

    #[must_use]
    pub fn with_decoration(mut self, decoration: impl VoxelDecorationHook) -> Self {
        self.decorations.push(Arc::new(decoration));
        self
    }

    pub fn push_decoration(&mut self, decoration: impl VoxelDecorationHook) {
        self.decorations.push(Arc::new(decoration));
    }

    #[must_use]
    pub fn sample_block(&self, world_pos: IVec3, config: &VoxelWorldConfig) -> BlockId {
        let mut sampled = self.sampler.sample_block(world_pos, config);
        for decoration in &self.decorations {
            if let Some(replacement) = decoration.decorate_block(world_pos, sampled, config) {
                sampled = replacement;
            }
        }
        sampled
    }
}

#[must_use]
pub fn generate_chunk(
    chunk: IVec3,
    config: &VoxelWorldConfig,
    generator: &VoxelWorldGenerator,
) -> ChunkData {
    let mut data = ChunkData::new_filled(config.chunk_dims, BlockId::AIR);
    for z in 0..config.chunk_dims.z {
        for y in 0..config.chunk_dims.y {
            for x in 0..config.chunk_dims.x {
                let local = UVec3::new(x, y, z);
                let world = local_to_world(chunk, local, config.chunk_dims);
                data.set(local, sample_generated_block(world, config, generator));
            }
        }
    }
    data
}

#[must_use]
pub fn sample_generated_block(
    world_pos: IVec3,
    config: &VoxelWorldConfig,
    generator: &VoxelWorldGenerator,
) -> BlockId {
    generator.sample_block(world_pos, config)
}
