use super::*;
use crate::{BlockId, BlockRegistry, config::LightingConfig, meshing::PaddedChunk};

#[test]
fn skylight_fills_open_columns_and_wraps_around_obstacles() {
    let registry = BlockRegistry::default();
    let mut padded = PaddedChunk::new_unknown(UVec3::new(2, 2, 2));

    for z in 0..4 {
        for y in 0..4 {
            for x in 0..4 {
                padded.set(
                    IVec3::new(x, y, z),
                    crate::meshing::SampledBlock {
                        id: BlockId::AIR,
                        known: true,
                    },
                );
            }
        }
    }
    padded.set(
        IVec3::new(1, 1, 1),
        crate::meshing::SampledBlock {
            id: BlockId::STONE,
            known: true,
        },
    );

    let field = build_light_field(&padded, &registry, &LightingConfig::default());
    assert_eq!(field.get(IVec3::new(0, 3, 0)), 15);
    assert_eq!(field.get(IVec3::new(0, 0, 0)), 15);
    assert_eq!(field.get(IVec3::new(1, 1, 1)), 0);
    assert_eq!(field.get(IVec3::new(1, 0, 1)), 14);
}

#[test]
fn emissive_blocks_seed_neighboring_air_cells() {
    let registry = BlockRegistry::default();
    let mut padded = PaddedChunk::new_unknown(UVec3::splat(1));
    for z in 0..3 {
        for y in 0..3 {
            for x in 0..3 {
                padded.set(
                    IVec3::new(x, y, z),
                    crate::meshing::SampledBlock {
                        id: BlockId::AIR,
                        known: true,
                    },
                );
            }
        }
    }

    padded.set(
        IVec3::new(1, 1, 1),
        crate::meshing::SampledBlock {
            id: BlockId::LAMP,
            known: true,
        },
    );

    let config = LightingConfig {
        sky_light_level: 0,
        ..Default::default()
    };
    let field = build_light_field(&padded, &registry, &config);
    assert_eq!(field.get(IVec3::new(2, 1, 1)), 11);
    assert_eq!(field.get(IVec3::new(2, 2, 1)), 10);
}

#[test]
fn brightness_respects_minimum_floor() {
    let config = LightingConfig {
        minimum_brightness: 0.25,
        ..Default::default()
    };

    assert!((brightness_for_level(0, &config) - 0.25).abs() < 0.0001);
    assert!((brightness_for_level(15, &config) - 1.0).abs() < 0.0001);
}
