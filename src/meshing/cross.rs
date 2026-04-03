use bevy::prelude::*;

use crate::{
    block::{BlockRegistry, MeshKind},
    chunk::ChunkData,
    config::VoxelWorldConfig,
    meshing::{MeshBuffers, MeshCounts, atlas_uvs, lighting::LightField},
};

pub fn emit_cross_quads(
    chunk_pos: IVec3,
    center: &ChunkData,
    light_field: Option<&LightField>,
    registry: &BlockRegistry,
    config: &VoxelWorldConfig,
    cutout: &mut MeshBuffers,
    counts: &mut MeshCounts,
) {
    let dims = center.dims();
    for z in 0..dims.z {
        for y in 0..dims.y {
            for x in 0..dims.x {
                let local = UVec3::new(x, y, z);
                let block = center.get(local);
                let definition = registry.get(block);
                if definition.mesh_kind != MeshKind::Cross {
                    continue;
                }

                let min = crate::coordinates::local_to_world(chunk_pos, local, config.chunk_dims)
                    .as_vec3();
                let max = min + Vec3::ONE;
                let tile = definition.atlas.sides;
                let uv = atlas_uvs(tile, &config.atlas);
                let ao = [3_u8; 4];
                let light = light_field
                    .map(|field| {
                        field.get(local.as_ivec3() + IVec3::ONE).max(
                            definition
                                .emissive_level
                                .min(config.lighting.max_light_level),
                        )
                    })
                    .unwrap_or(config.lighting.max_light_level);

                cutout.push_quad(
                    [
                        [min.x, min.y, min.z],
                        [max.x, min.y, max.z],
                        [max.x, max.y, max.z],
                        [min.x, max.y, min.z],
                    ],
                    [0.707, 0.0, -0.707],
                    uv,
                    ao,
                    [light; 4],
                    0.0,
                    &config.lighting,
                );
                cutout.push_quad(
                    [
                        [max.x, min.y, min.z],
                        [min.x, min.y, max.z],
                        [min.x, max.y, max.z],
                        [max.x, max.y, min.z],
                    ],
                    [-0.707, 0.0, -0.707],
                    uv,
                    ao,
                    [light; 4],
                    0.0,
                    &config.lighting,
                );
                counts.cutout_quads += 2;
            }
        }
    }
}
