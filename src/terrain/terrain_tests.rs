use bevy::prelude::*;

use super::*;
use crate::{BlockId, config::VoxelWorldConfig};

#[derive(Clone)]
struct CheckerSampler {
    solid: BlockId,
    alternate: BlockId,
}

impl VoxelBlockSampler for CheckerSampler {
    fn sample_block(&self, world_pos: IVec3, _config: &VoxelWorldConfig) -> BlockId {
        if world_pos.y > 0 {
            BlockId::AIR
        } else if (world_pos.x + world_pos.z).rem_euclid(2) == 0 {
            self.solid
        } else {
            self.alternate
        }
    }
}

#[derive(Clone)]
struct ColumnDecoration {
    x: i32,
    z: i32,
    block: BlockId,
}

impl VoxelDecorationHook for ColumnDecoration {
    fn decorate_block(
        &self,
        world_pos: IVec3,
        sampled: BlockId,
        _config: &VoxelWorldConfig,
    ) -> Option<BlockId> {
        (sampled == BlockId::AIR
            && world_pos.x == self.x
            && world_pos.z == self.z
            && world_pos.y == 1)
            .then_some(self.block)
    }
}

#[test]
fn generation_is_deterministic_for_same_sampler_chain() {
    let config = VoxelWorldConfig::default();
    let generator = VoxelWorldGenerator::new(CheckerSampler {
        solid: BlockId::SOLID,
        alternate: BlockId::SOLID_ALT,
    })
    .with_decoration(ColumnDecoration {
        x: 2,
        z: -3,
        block: BlockId::EMISSIVE,
    });
    let a = generate_chunk(IVec3::new(2, 0, -3), &config, &generator);
    let b = generate_chunk(IVec3::new(2, 0, -3), &config, &generator);
    assert_eq!(a, b);
}

#[test]
fn swapping_sampler_changes_generated_chunk() {
    let config = VoxelWorldConfig::default();
    let flat = VoxelWorldGenerator::default();
    let checker = VoxelWorldGenerator::new(CheckerSampler {
        solid: BlockId::SOLID,
        alternate: BlockId::SOLID_ALT,
    });
    let a = generate_chunk(IVec3::ZERO, &config, &flat);
    let b = generate_chunk(IVec3::ZERO, &config, &checker);
    assert_ne!(a.blocks(), b.blocks());
}

#[test]
fn default_flat_sampler_produces_surface_fill_and_air_layers() {
    let config = VoxelWorldConfig::default();
    let generator = VoxelWorldGenerator::default();

    assert_eq!(
        sample_generated_block(IVec3::new(0, -1, 0), &config, &generator),
        BlockId::SOLID
    );
    assert_eq!(
        sample_generated_block(IVec3::new(0, 0, 0), &config, &generator),
        BlockId::SOLID_ALT
    );
    assert_eq!(
        sample_generated_block(IVec3::new(0, 1, 0), &config, &generator),
        BlockId::AIR
    );
}

#[test]
fn decoration_hooks_can_override_air_samples() {
    let config = VoxelWorldConfig::default();
    let generator = VoxelWorldGenerator::default().with_decoration(ColumnDecoration {
        x: 0,
        z: 0,
        block: BlockId::CROSS,
    });

    assert_eq!(
        sample_generated_block(IVec3::new(0, 1, 0), &config, &generator),
        BlockId::CROSS
    );
    assert_eq!(
        sample_generated_block(IVec3::new(1, 1, 0), &config, &generator),
        BlockId::AIR
    );
}
