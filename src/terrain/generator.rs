use bevy::prelude::*;

use crate::{
    block::BlockId,
    chunk::ChunkData,
    config::{GeneratorKind, TerrainConfig, VoxelWorldConfig},
    coordinates::{local_to_world, world_to_chunk_local},
};

use super::noise::{fbm2, value3};

#[must_use]
pub fn generate_chunk(chunk: IVec3, config: &VoxelWorldConfig) -> ChunkData {
    let mut data = ChunkData::new_filled(config.chunk_dims, BlockId::AIR);
    for z in 0..config.chunk_dims.z {
        for y in 0..config.chunk_dims.y {
            for x in 0..config.chunk_dims.x {
                let local = UVec3::new(x, y, z);
                let world = local_to_world(chunk, local, config.chunk_dims);
                data.set(local, sample_generated_block(world, config));
            }
        }
    }
    data
}

#[must_use]
pub fn sample_generated_block(world_pos: IVec3, config: &VoxelWorldConfig) -> BlockId {
    match config.terrain.generator {
        GeneratorKind::Flat => sample_flat(world_pos, &config.terrain),
        GeneratorKind::LayeredNoise => sample_layered_noise(world_pos, config),
    }
}

fn sample_flat(world_pos: IVec3, terrain: &TerrainConfig) -> BlockId {
    if world_pos.y < terrain.base_height - 1 {
        BlockId::STONE
    } else if world_pos.y == terrain.base_height - 1 {
        BlockId::DIRT
    } else if world_pos.y == terrain.base_height {
        BlockId::GRASS
    } else {
        BlockId::AIR
    }
}

fn sample_layered_noise(world_pos: IVec3, config: &VoxelWorldConfig) -> BlockId {
    let terrain = &config.terrain;
    let height = terrain.base_height as f32
        + fbm2(
            config.seed,
            Vec2::new(world_pos.x as f32, world_pos.z as f32) * terrain.height_frequency,
            terrain.hill_octaves,
        ) * terrain.height_amplitude as f32;
    let terrain_height = height.round() as i32;

    if world_pos.y > terrain_height {
        if world_pos.y <= terrain.water_level {
            return BlockId::WATER;
        }
        if world_pos.y == terrain_height + 1 {
            let tall_grass_noise = fbm2(
                config.seed ^ 0x7171_1818,
                Vec2::new(world_pos.x as f32, world_pos.z as f32) * 0.24,
                2,
            );
            if tall_grass_noise > 1.0 - terrain.foliage_chance * 2.0 {
                return BlockId::TALL_GRASS;
            }
        }
        return BlockId::AIR;
    }

    let cave = value3(
        config.seed ^ 0x0f0f_aaaa,
        Vec3::new(world_pos.x as f32, world_pos.y as f32, world_pos.z as f32)
            * terrain.cave_frequency,
    );
    if world_pos.y < terrain_height - 2 && cave > terrain.cave_threshold {
        return BlockId::AIR;
    }

    if world_pos.y == terrain_height {
        if terrain_height <= terrain.water_level + 1 {
            BlockId::SAND
        } else {
            let lamp_noise = fbm2(
                config.seed ^ 0x44aa_9911,
                Vec2::new(world_pos.x as f32, world_pos.z as f32) * 0.07,
                1,
            );
            if lamp_noise > 0.92 {
                BlockId::LAMP
            } else {
                BlockId::GRASS
            }
        }
    } else if world_pos.y >= terrain_height - 3 {
        BlockId::DIRT
    } else {
        BlockId::STONE
    }
}

#[allow(dead_code)]
fn _world_to_chunk_for_sampling(world_pos: IVec3, config: &VoxelWorldConfig) -> (IVec3, UVec3) {
    world_to_chunk_local(world_pos, config.chunk_dims)
}
