use bevy::prelude::*;

use crate::{
    block::{BlockId, BlockRegistry},
    chunk::ChunkData,
    coordinates::{chunk_origin, local_to_world},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RaycastHit {
    pub world_pos: IVec3,
    pub normal: IVec3,
    pub block: BlockId,
}

pub trait BlockSampler {
    fn sample_block(&self, world_pos: IVec3) -> Option<BlockId>;
}

pub fn raycast_blocks(
    sampler: &impl BlockSampler,
    registry: &BlockRegistry,
    origin: Vec3,
    direction: Vec3,
    max_distance: f32,
) -> Option<RaycastHit> {
    let direction = direction.normalize_or_zero();
    if direction == Vec3::ZERO || max_distance <= 0.0 {
        return None;
    }

    let mut voxel = origin.floor().as_ivec3();
    let step = direction.signum().as_ivec3();
    let inv = direction.recip();
    let delta = inv.abs();

    let mut side_dist = Vec3::ZERO;
    side_dist.x = initial_side_distance(origin.x, voxel.x, step.x, inv.x);
    side_dist.y = initial_side_distance(origin.y, voxel.y, step.y, inv.y);
    side_dist.z = initial_side_distance(origin.z, voxel.z, step.z, inv.z);

    let mut traveled = 0.0;
    let mut normal = IVec3::ZERO;

    while traveled <= max_distance {
        if let Some(block) = sampler.sample_block(voxel)
            && registry.get(block).solid
        {
            return Some(RaycastHit {
                world_pos: voxel,
                normal,
                block,
            });
        }

        if side_dist.x < side_dist.y && side_dist.x < side_dist.z {
            traveled = side_dist.x;
            side_dist.x += delta.x;
            voxel.x += step.x;
            normal = IVec3::new(-step.x, 0, 0);
        } else if side_dist.y < side_dist.z {
            traveled = side_dist.y;
            side_dist.y += delta.y;
            voxel.y += step.y;
            normal = IVec3::new(0, -step.y, 0);
        } else {
            traveled = side_dist.z;
            side_dist.z += delta.z;
            voxel.z += step.z;
            normal = IVec3::new(0, 0, -step.z);
        }
    }

    None
}

fn initial_side_distance(origin: f32, voxel: i32, step: i32, inv: f32) -> f32 {
    if step == 0 {
        f32::INFINITY
    } else {
        let boundary = if step > 0 { voxel + 1 } else { voxel };
        (boundary as f32 - origin) * inv
    }
}

#[must_use]
pub fn rebuild_world_pos(chunk: IVec3, local_index: u32, dims: UVec3) -> IVec3 {
    let local = ChunkData::new_filled(dims, BlockId::AIR).local_from_index(local_index);
    local_to_world(chunk, local, dims)
}

#[must_use]
pub fn chunk_bounds_world(chunk: IVec3, dims: UVec3) -> (IVec3, IVec3) {
    let min = chunk_origin(chunk, dims);
    let max = min + dims.as_ivec3() - IVec3::ONE;
    (min, max)
}

#[cfg(test)]
#[path = "raycast_tests.rs"]
mod tests;
