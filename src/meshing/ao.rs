use bevy::prelude::*;

use crate::{
    block::BlockRegistry,
    meshing::{PaddedChunk, SampledBlock},
};

pub fn ambient_occlusion_for_face(
    padded: &PaddedChunk,
    registry: &BlockRegistry,
    solid_local: IVec3,
    normal: IVec3,
) -> [u8; 4] {
    let corners = face_corner_offsets(normal);
    let mut values = [3_u8; 4];
    for (index, (side_a, side_b, corner)) in corners.into_iter().enumerate() {
        let side_a = is_occluding(padded.get(solid_local + side_a), registry);
        let side_b = is_occluding(padded.get(solid_local + side_b), registry);
        let corner = is_occluding(padded.get(solid_local + corner), registry);
        values[index] = if side_a && side_b {
            0
        } else {
            3 - side_a as u8 - side_b as u8 - corner as u8
        };
    }
    values
}

fn is_occluding(sample: SampledBlock, registry: &BlockRegistry) -> bool {
    sample.known && registry.get(sample.id).opaque
}

fn face_corner_offsets(normal: IVec3) -> [(IVec3, IVec3, IVec3); 4] {
    match normal {
        IVec3 { x: 1, y: 0, z: 0 } => [
            (
                IVec3::new(1, -1, 0),
                IVec3::new(1, 0, -1),
                IVec3::new(1, -1, -1),
            ),
            (
                IVec3::new(1, 1, 0),
                IVec3::new(1, 0, -1),
                IVec3::new(1, 1, -1),
            ),
            (
                IVec3::new(1, 1, 0),
                IVec3::new(1, 0, 1),
                IVec3::new(1, 1, 1),
            ),
            (
                IVec3::new(1, -1, 0),
                IVec3::new(1, 0, 1),
                IVec3::new(1, -1, 1),
            ),
        ],
        IVec3 { x: -1, y: 0, z: 0 } => [
            (
                IVec3::new(-1, -1, 0),
                IVec3::new(-1, 0, 1),
                IVec3::new(-1, -1, 1),
            ),
            (
                IVec3::new(-1, 1, 0),
                IVec3::new(-1, 0, 1),
                IVec3::new(-1, 1, 1),
            ),
            (
                IVec3::new(-1, 1, 0),
                IVec3::new(-1, 0, -1),
                IVec3::new(-1, 1, -1),
            ),
            (
                IVec3::new(-1, -1, 0),
                IVec3::new(-1, 0, -1),
                IVec3::new(-1, -1, -1),
            ),
        ],
        IVec3 { x: 0, y: 1, z: 0 } => [
            (
                IVec3::new(-1, 1, 0),
                IVec3::new(0, 1, -1),
                IVec3::new(-1, 1, -1),
            ),
            (
                IVec3::new(1, 1, 0),
                IVec3::new(0, 1, -1),
                IVec3::new(1, 1, -1),
            ),
            (
                IVec3::new(1, 1, 0),
                IVec3::new(0, 1, 1),
                IVec3::new(1, 1, 1),
            ),
            (
                IVec3::new(-1, 1, 0),
                IVec3::new(0, 1, 1),
                IVec3::new(-1, 1, 1),
            ),
        ],
        IVec3 { x: 0, y: -1, z: 0 } => [
            (
                IVec3::new(-1, -1, 0),
                IVec3::new(0, -1, 1),
                IVec3::new(-1, -1, 1),
            ),
            (
                IVec3::new(1, -1, 0),
                IVec3::new(0, -1, 1),
                IVec3::new(1, -1, 1),
            ),
            (
                IVec3::new(1, -1, 0),
                IVec3::new(0, -1, -1),
                IVec3::new(1, -1, -1),
            ),
            (
                IVec3::new(-1, -1, 0),
                IVec3::new(0, -1, -1),
                IVec3::new(-1, -1, -1),
            ),
        ],
        IVec3 { x: 0, y: 0, z: 1 } => [
            (
                IVec3::new(-1, 0, 1),
                IVec3::new(0, -1, 1),
                IVec3::new(-1, -1, 1),
            ),
            (
                IVec3::new(-1, 0, 1),
                IVec3::new(0, 1, 1),
                IVec3::new(-1, 1, 1),
            ),
            (
                IVec3::new(1, 0, 1),
                IVec3::new(0, 1, 1),
                IVec3::new(1, 1, 1),
            ),
            (
                IVec3::new(1, 0, 1),
                IVec3::new(0, -1, 1),
                IVec3::new(1, -1, 1),
            ),
        ],
        _ => [
            (
                IVec3::new(1, 0, -1),
                IVec3::new(0, -1, -1),
                IVec3::new(1, -1, -1),
            ),
            (
                IVec3::new(1, 0, -1),
                IVec3::new(0, 1, -1),
                IVec3::new(1, 1, -1),
            ),
            (
                IVec3::new(-1, 0, -1),
                IVec3::new(0, 1, -1),
                IVec3::new(-1, 1, -1),
            ),
            (
                IVec3::new(-1, 0, -1),
                IVec3::new(0, -1, -1),
                IVec3::new(-1, -1, -1),
            ),
        ],
    }
}
