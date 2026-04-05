use bevy::prelude::*;

use crate::{
    block::BlockId,
    chunk::ChunkData,
    config::{GeneratorKind, TerrainConfig, VoxelWorldConfig},
    coordinates::{local_to_world, world_to_chunk_local},
};

use super::noise::{fbm2, tree_hash, value3};

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
    let height = terrain_height_at(config, world_pos.x, world_pos.z);
    let terrain_height = height.round() as i32;

    if world_pos.y > terrain_height {
        // Check for tree blocks above surface
        if let Some(tree_block) = sample_tree_at(world_pos, config) {
            return tree_block;
        }
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

fn terrain_height_at(config: &VoxelWorldConfig, x: i32, z: i32) -> f32 {
    let terrain = &config.terrain;
    terrain.base_height as f32
        + fbm2(
            config.seed,
            Vec2::new(x as f32, z as f32) * terrain.height_frequency,
            terrain.hill_octaves,
        ) * terrain.height_amplitude as f32
}

/// Check if a tree contributes a block at this world position.
/// Trees are placed on a grid (every 5 blocks in X/Z) with a hash-based
/// probability. Each tree is a 1-block trunk (4-6 blocks tall) topped
/// with a roughly spherical leaf canopy (radius 2).
fn sample_tree_at(world_pos: IVec3, config: &VoxelWorldConfig) -> Option<BlockId> {
    let terrain = &config.terrain;
    let tree_chance = terrain.foliage_chance * 1.5;
    let search_radius = 3_i32; // leaf canopy can extend 2-3 blocks from trunk

    for dz in -search_radius..=search_radius {
        for dx in -search_radius..=search_radius {
            let trunk_x = world_pos.x + dx;
            let trunk_z = world_pos.z + dz;

            // Trees only placed at positions where hash passes threshold
            let h = tree_hash(config.seed, trunk_x, trunk_z);
            if h > tree_chance {
                continue;
            }

            // Compute terrain height at trunk base
            let ground = terrain_height_at(config, trunk_x, trunk_z).round() as i32;

            // No trees underwater or on sand
            if ground <= terrain.water_level + 1 {
                continue;
            }

            // Deterministic tree height (4-6 blocks)
            let tree_height = 4 + ((h * 1000.0) as i32 % 3);
            let trunk_top = ground + tree_height;
            let canopy_center = trunk_top;
            let canopy_radius = 2;

            // Check if world_pos is the trunk
            if world_pos.x == trunk_x && world_pos.z == trunk_z {
                if world_pos.y > ground && world_pos.y <= trunk_top {
                    return Some(BlockId::WOOD);
                }
            }

            // Check if world_pos is in the canopy
            let rel_x = world_pos.x - trunk_x;
            let rel_y = world_pos.y - canopy_center;
            let rel_z = world_pos.z - trunk_z;
            let dist_sq = rel_x * rel_x + rel_y * rel_y + rel_z * rel_z;
            if dist_sq <= canopy_radius * canopy_radius + 1
                && world_pos.y >= canopy_center - 1
                && world_pos.y <= canopy_center + canopy_radius
            {
                // Don't overwrite trunk
                if !(world_pos.x == trunk_x && world_pos.z == trunk_z && world_pos.y <= trunk_top) {
                    return Some(BlockId::LEAVES);
                }
            }
        }
    }
    None
}

#[allow(dead_code)]
fn _world_to_chunk_for_sampling(world_pos: IVec3, config: &VoxelWorldConfig) -> (IVec3, UVec3) {
    world_to_chunk_local(world_pos, config.chunk_dims)
}
