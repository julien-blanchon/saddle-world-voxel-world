use bevy::prelude::*;

use super::*;
use crate::config::VoxelWorldConfig;

#[test]
fn generation_is_deterministic_for_same_seed() {
    let config = VoxelWorldConfig::default();
    let a = generate_chunk(IVec3::new(2, 0, -3), &config);
    let b = generate_chunk(IVec3::new(2, 0, -3), &config);
    assert_eq!(a, b);
}

#[test]
fn generation_changes_with_seed() {
    let config_a = VoxelWorldConfig::default();
    let config_b = VoxelWorldConfig { seed: 999, ..Default::default() };
    let a = generate_chunk(IVec3::new(0, 0, 0), &config_a);
    let b = generate_chunk(IVec3::new(0, 0, 0), &config_b);
    assert_ne!(a.blocks(), b.blocks());
}

#[test]
fn sampled_height_stays_in_expected_band() {
    let config = VoxelWorldConfig::default();
    let mut max_height = i32::MIN;
    let mut min_height = i32::MAX;
    for x in -32..=32 {
        for z in -32..=32 {
            let mut surface = -64;
            for y in (-8)..48 {
                let block = sample_generated_block(IVec3::new(x, y, z), &config);
                if matches!(
                    block,
                    crate::BlockId::GRASS
                        | crate::BlockId::DIRT
                        | crate::BlockId::STONE
                        | crate::BlockId::SAND
                        | crate::BlockId::LAMP
                ) {
                    surface = y;
                }
            }
            max_height = max_height.max(surface);
            min_height = min_height.min(surface);
        }
    }
    assert!(max_height <= 48);
    assert!(min_height >= -64);
    assert!(max_height > min_height);
}

#[test]
fn known_samples_stay_stable() {
    let config = VoxelWorldConfig::default();
    let samples = [
        (IVec3::new(0, 14, 0), crate::BlockId::AIR),
        (IVec3::new(0, 8, 0), crate::BlockId::WATER),
        (IVec3::new(8, 20, -8), crate::BlockId::STONE),
        (IVec3::new(0, 4, 0), crate::BlockId::DIRT),
        (IVec3::new(-8, 10, 4), crate::BlockId::STONE),
    ];

    for (position, expected) in samples {
        assert_eq!(
            sample_generated_block(position, &config),
            expected,
            "{position:?}"
        );
    }
}
