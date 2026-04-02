use bevy::prelude::*;

use super::*;
use crate::{block::BlockRegistry, chunk::ChunkData};

struct TestSampler {
    blocks: std::collections::HashMap<IVec3, BlockId>,
}

impl BlockSampler for TestSampler {
    fn sample_block(&self, world_pos: IVec3) -> Option<BlockId> {
        self.blocks.get(&world_pos).copied()
    }
}

#[test]
fn raycast_hits_block_and_returns_face_normal() {
    let sampler = TestSampler {
        blocks: std::iter::once((IVec3::ZERO, BlockId::STONE)).collect(),
    };
    let hit = raycast_blocks(
        &sampler,
        &BlockRegistry::default(),
        Vec3::new(-1.5, 0.5, 0.5),
        Vec3::X,
        10.0,
    )
    .expect("ray should hit");
    assert_eq!(hit.world_pos, IVec3::ZERO);
    assert_eq!(hit.normal, IVec3::NEG_X);
}

#[test]
fn raycast_misses_when_no_block_is_present() {
    let sampler = TestSampler {
        blocks: Default::default(),
    };
    assert!(
        raycast_blocks(
            &sampler,
            &BlockRegistry::default(),
            Vec3::ZERO,
            Vec3::X,
            4.0
        )
        .is_none()
    );
}

#[test]
fn raycast_honors_max_distance() {
    let sampler = TestSampler {
        blocks: std::iter::once((IVec3::new(5, 0, 0), BlockId::STONE)).collect(),
    };
    assert!(
        raycast_blocks(
            &sampler,
            &BlockRegistry::default(),
            Vec3::new(0.5, 0.5, 0.5),
            Vec3::X,
            2.0,
        )
        .is_none()
    );
}

#[test]
fn raycast_handles_negative_coordinates() {
    let sampler = TestSampler {
        blocks: std::iter::once((IVec3::new(-2, 0, 0), BlockId::STONE)).collect(),
    };
    let hit = raycast_blocks(
        &sampler,
        &BlockRegistry::default(),
        Vec3::new(1.5, 0.5, 0.5),
        Vec3::NEG_X,
        8.0,
    )
    .expect("ray should hit");
    assert_eq!(hit.world_pos, IVec3::new(-2, 0, 0));
    assert_eq!(hit.normal, IVec3::X);
}

#[test]
fn raycast_crosses_chunk_boundaries() {
    let sampler = TestSampler {
        blocks: std::iter::once((IVec3::new(16, 0, 0), BlockId::STONE)).collect(),
    };
    let hit = raycast_blocks(
        &sampler,
        &BlockRegistry::default(),
        Vec3::new(15.2, 0.5, 0.5),
        Vec3::X,
        4.0,
    )
    .expect("ray should cross into the next chunk");
    assert_eq!(hit.world_pos, IVec3::new(16, 0, 0));
}

#[test]
fn rebuild_world_pos_roundtrips_index() {
    let dims = UVec3::splat(4);
    let chunk = IVec3::new(-2, 1, 3);
    let data = ChunkData::new_filled(dims, BlockId::AIR);
    let local = UVec3::new(2, 1, 3);
    let world = rebuild_world_pos(chunk, data.index(local) as u32, dims);
    assert_eq!(world, local_to_world(chunk, local, dims));
}
