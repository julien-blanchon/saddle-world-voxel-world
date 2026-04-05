use bevy::prelude::*;

use crate::{
    block::{BlockId, BlockRegistry, MaterialClass, MeshKind},
    chunk::ChunkData,
    config::VoxelWorldConfig,
    meshing::{
        MeshBuffers, MeshCounts, PaddedChunk, SampledBlock, atlas_uvs, lighting::LightField,
    },
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

/// Emit naive (non-greedy) cube faces for cutout-class cube blocks (e.g. leaves).
/// These go into the cutout mesh buffer with alpha-mask rendering.
pub fn emit_cutout_cube_faces(
    chunk_pos: IVec3,
    center: &ChunkData,
    padded: &PaddedChunk,
    light_field: Option<&LightField>,
    registry: &BlockRegistry,
    config: &VoxelWorldConfig,
    cutout: &mut MeshBuffers,
    counts: &mut MeshCounts,
) {
    let dims = center.dims().as_ivec3();
    let normals: [IVec3; 6] = [
        IVec3::X,
        IVec3::NEG_X,
        IVec3::Y,
        IVec3::NEG_Y,
        IVec3::Z,
        IVec3::NEG_Z,
    ];

    for z in 0..dims.z {
        for y in 0..dims.y {
            for x in 0..dims.x {
                let local = IVec3::new(x, y, z);
                let block = center.get(local.as_uvec3());
                let definition = registry.get(block);
                if definition.mesh_kind != MeshKind::Cube
                    || definition.material_class != MaterialClass::Cutout
                {
                    continue;
                }

                for &normal in &normals {
                    let neighbor = local + normal;
                    let padded_neighbor = neighbor + IVec3::ONE;
                    let neighbor_sample = padded.get(padded_neighbor);

                    // Emit face if neighbor is air, unknown, or non-occluding
                    if !should_cull_cutout_face(neighbor_sample, registry) {
                        let tile = registry.atlas_tile_for_face(block, normal);
                        let light = light_field
                            .map(|field| {
                                super::lighting::face_light_level(
                                    field,
                                    local + IVec3::ONE,
                                    normal,
                                    definition.emissive_level,
                                    &config.lighting,
                                )
                            })
                            .unwrap_or(config.lighting.max_light_level);
                        let ao = if config.meshing.ambient_occlusion && config.lighting.baked_ao {
                            super::ambient_occlusion_for_face(
                                padded,
                                registry,
                                local + IVec3::ONE,
                                normal,
                            )
                        } else {
                            [3_u8; 4]
                        };

                        emit_face(
                            chunk_pos, config, cutout, block, local, normal, tile, ao, light,
                        );
                        counts.cutout_quads += 1;
                    }
                }
            }
        }
    }
}

fn should_cull_cutout_face(neighbor: SampledBlock, registry: &BlockRegistry) -> bool {
    if !neighbor.known {
        return false;
    }
    let def = registry.get(neighbor.id);
    // Only cull against fully opaque solid cubes
    def.culls_opaque_faces()
}

#[allow(clippy::too_many_arguments)]
fn emit_face(
    chunk_pos: IVec3,
    config: &VoxelWorldConfig,
    buffers: &mut MeshBuffers,
    _block: BlockId,
    local: IVec3,
    normal: IVec3,
    tile: u16,
    ao: [u8; 4],
    light: u8,
) {
    let world_base = (chunk_pos * config.chunk_dims.as_ivec3() + local).as_vec3();

    let (p0, p1, p2, p3) = face_vertices(world_base, normal);

    buffers.push_quad(
        [p0, p1, p2, p3],
        normal.as_vec3().to_array(),
        atlas_uvs(tile, &config.atlas),
        ao,
        [light; 4],
        config.meshing.ao_strength,
        &config.lighting,
    );
}

fn face_vertices(base: Vec3, normal: IVec3) -> ([f32; 3], [f32; 3], [f32; 3], [f32; 3]) {
    let b = base;
    let s = Vec3::ONE;
    match (normal.x, normal.y, normal.z) {
        (1, 0, 0) => (
            [b.x + s.x, b.y, b.z],
            [b.x + s.x, b.y, b.z + s.z],
            [b.x + s.x, b.y + s.y, b.z + s.z],
            [b.x + s.x, b.y + s.y, b.z],
        ),
        (-1, 0, 0) => (
            [b.x, b.y, b.z + s.z],
            [b.x, b.y, b.z],
            [b.x, b.y + s.y, b.z],
            [b.x, b.y + s.y, b.z + s.z],
        ),
        (0, 1, 0) => (
            [b.x, b.y + s.y, b.z + s.z],
            [b.x + s.x, b.y + s.y, b.z + s.z],
            [b.x + s.x, b.y + s.y, b.z],
            [b.x, b.y + s.y, b.z],
        ),
        (0, -1, 0) => (
            [b.x, b.y, b.z],
            [b.x + s.x, b.y, b.z],
            [b.x + s.x, b.y, b.z + s.z],
            [b.x, b.y, b.z + s.z],
        ),
        (0, 0, 1) => (
            [b.x + s.x, b.y, b.z + s.z],
            [b.x, b.y, b.z + s.z],
            [b.x, b.y + s.y, b.z + s.z],
            [b.x + s.x, b.y + s.y, b.z + s.z],
        ),
        (0, 0, -1) => (
            [b.x, b.y, b.z],
            [b.x + s.x, b.y, b.z],
            [b.x + s.x, b.y + s.y, b.z],
            [b.x, b.y + s.y, b.z],
        ),
        _ => (
            [b.x, b.y, b.z],
            [b.x + s.x, b.y, b.z],
            [b.x + s.x, b.y + s.y, b.z],
            [b.x, b.y + s.y, b.z],
        ),
    }
}
