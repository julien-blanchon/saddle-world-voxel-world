use bevy::prelude::*;

use super::{PaddedChunk, SampledBlock, ao::ambient_occlusion_for_face};
use crate::{BlockId, BlockRegistry};

#[test]
fn ao_is_bright_for_unoccluded_top_face() {
    let mut padded = PaddedChunk::new_unknown(UVec3::splat(2));
    padded.set(
        IVec3::new(1, 1, 1),
        SampledBlock {
            id: BlockId::SOLID,
            known: true,
        },
    );
    let ao = ambient_occlusion_for_face(
        &padded,
        &BlockRegistry::default(),
        IVec3::new(1, 1, 1),
        IVec3::Y,
    );
    assert_eq!(ao, [3, 3, 3, 3]);
}

#[test]
fn ao_darkens_when_corner_is_occluded() {
    let mut padded = PaddedChunk::new_unknown(UVec3::splat(2));
    padded.set(
        IVec3::new(1, 1, 1),
        SampledBlock {
            id: BlockId::SOLID,
            known: true,
        },
    );
    padded.set(
        IVec3::new(0, 2, 1),
        SampledBlock {
            id: BlockId::SOLID,
            known: true,
        },
    );
    padded.set(
        IVec3::new(1, 2, 0),
        SampledBlock {
            id: BlockId::SOLID,
            known: true,
        },
    );
    let ao = ambient_occlusion_for_face(
        &padded,
        &BlockRegistry::default(),
        IVec3::new(1, 1, 1),
        IVec3::Y,
    );
    assert_eq!(ao[0], 0);
    assert!(ao[1] <= 3);
}
