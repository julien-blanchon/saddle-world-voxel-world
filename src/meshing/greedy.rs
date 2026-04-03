use bevy::prelude::*;

use crate::{
    block::{BlockId, BlockRegistry, MaterialClass},
    chunk::ChunkData,
    config::VoxelWorldConfig,
    meshing::{
        MeshBuffers, MeshCounts, PaddedChunk, SampledBlock, ambient_occlusion_for_face,
        atlas_tile_for, atlas_uvs, culls_neighbor, lighting::LightField, should_emit_cube,
    },
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct FaceKey {
    block: BlockId,
    normal: IVec3,
    tile: u16,
    ao: [u8; 4],
    light: u8,
    material_class: MaterialClass,
}

pub fn emit_greedy_quads(
    chunk_pos: IVec3,
    center: &ChunkData,
    padded: &PaddedChunk,
    light_field: Option<&LightField>,
    registry: &BlockRegistry,
    config: &VoxelWorldConfig,
    opaque: &mut MeshBuffers,
    counts: &mut MeshCounts,
) {
    let dims = center.dims().as_ivec3();
    for axis in 0..3 {
        let u = (axis + 1) % 3;
        let v = (axis + 2) % 3;
        let du = dims[u] as usize;
        let dv = dims[v] as usize;
        let mut mask = vec![None::<FaceKey>; du * dv];

        for slice in -1..dims[axis] {
            mask.fill(None);

            for j in 0..dv {
                for i in 0..du {
                    let mut a = IVec3::ZERO;
                    let mut b = IVec3::ZERO;
                    a[axis] = slice;
                    b[axis] = slice + 1;
                    a[u] = i as i32;
                    b[u] = i as i32;
                    a[v] = j as i32;
                    b[v] = j as i32;

                    let a_sample = padded.get(a + IVec3::ONE);
                    let b_sample = padded.get(b + IVec3::ONE);
                    let face =
                        visible_face(
                            axis,
                            a,
                            b,
                            a_sample,
                            b_sample,
                            padded,
                            light_field,
                            registry,
                            config,
                        );
                    mask[i + j * du] = face;
                }
            }

            let mut j = 0;
            while j < dv {
                let mut i = 0;
                while i < du {
                    let Some(key) = mask[i + j * du] else {
                        i += 1;
                        continue;
                    };

                    let (width, height) = if config.meshing.enable_greedy {
                        let mut width = 1;
                        while i + width < du && mask[i + width + j * du] == Some(key) {
                            width += 1;
                        }

                        let mut height = 1;
                        'outer: while j + height < dv {
                            for x in 0..width {
                                if mask[i + x + (j + height) * du] != Some(key) {
                                    break 'outer;
                                }
                            }
                            height += 1;
                        }
                        (width, height)
                    } else {
                        (1, 1)
                    };

                    emit_quad(
                        chunk_pos,
                        config,
                        opaque,
                        key,
                        axis,
                        slice,
                        i as i32,
                        j as i32,
                        width as i32,
                        height as i32,
                    );
                    counts.opaque_quads += 1;

                    for dy in 0..height {
                        for dx in 0..width {
                            mask[i + dx + (j + dy) * du] = None;
                        }
                    }

                    i += width;
                }
                j += 1;
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn visible_face(
    axis: usize,
    a_local: IVec3,
    b_local: IVec3,
    a_sample: SampledBlock,
    b_sample: SampledBlock,
    padded: &PaddedChunk,
    light_field: Option<&LightField>,
    registry: &BlockRegistry,
    config: &VoxelWorldConfig,
) -> Option<FaceKey> {
    let normal = axis_normal(axis);
    let negative_normal = -normal;

    if should_emit_cube(a_sample, registry)
        && (!culls_neighbor(b_sample, registry)
            || (!b_sample.known && config.meshing.render_faces_against_unknown_neighbors))
    {
        let block = a_sample.id;
        let definition = registry.get(block);
        if definition.material_class == MaterialClass::Opaque {
            let ao = if config.meshing.ambient_occlusion && config.lighting.baked_ao {
                ambient_occlusion_for_face(padded, registry, a_local + IVec3::ONE, normal)
            } else {
                [3_u8; 4]
            };
            return Some(FaceKey {
                block,
                normal,
                tile: atlas_tile_for(registry, block, normal),
                ao,
                light: light_field
                    .map(|field| {
                        super::lighting::face_light_level(
                            field,
                            a_local + IVec3::ONE,
                            normal,
                            definition.emissive_level,
                            &config.lighting,
                        )
                    })
                    .unwrap_or(config.lighting.max_light_level),
                material_class: definition.material_class,
            });
        }
    }

    if should_emit_cube(b_sample, registry)
        && (!culls_neighbor(a_sample, registry)
            || (!a_sample.known && config.meshing.render_faces_against_unknown_neighbors))
    {
        let block = b_sample.id;
        let definition = registry.get(block);
        if definition.material_class == MaterialClass::Opaque {
            let ao = if config.meshing.ambient_occlusion && config.lighting.baked_ao {
                ambient_occlusion_for_face(padded, registry, b_local + IVec3::ONE, negative_normal)
            } else {
                [3_u8; 4]
            };
            return Some(FaceKey {
                block,
                normal: negative_normal,
                tile: atlas_tile_for(registry, block, negative_normal),
                ao,
                light: light_field
                    .map(|field| {
                        super::lighting::face_light_level(
                            field,
                            b_local + IVec3::ONE,
                            negative_normal,
                            definition.emissive_level,
                            &config.lighting,
                        )
                    })
                    .unwrap_or(config.lighting.max_light_level),
                material_class: definition.material_class,
            });
        }
    }

    None
}

fn axis_normal(axis: usize) -> IVec3 {
    match axis {
        0 => IVec3::X,
        1 => IVec3::Y,
        _ => IVec3::Z,
    }
}

#[allow(clippy::too_many_arguments)]
fn emit_quad(
    chunk_pos: IVec3,
    config: &VoxelWorldConfig,
    buffers: &mut MeshBuffers,
    key: FaceKey,
    axis: usize,
    slice: i32,
    i: i32,
    j: i32,
    width: i32,
    height: i32,
) {
    let (u_axis, v_axis) = ((axis + 1) % 3, (axis + 2) % 3);
    let mut origin = IVec3::ZERO;
    origin[axis] = slice + 1;
    origin[u_axis] = i;
    origin[v_axis] = j;

    let mut u_vec = IVec3::ZERO;
    u_vec[u_axis] = width;
    let mut v_vec = IVec3::ZERO;
    v_vec[v_axis] = height;

    let p0 = world_point(chunk_pos, config, origin);
    let p1 = world_point(chunk_pos, config, origin + u_vec);
    let p2 = world_point(chunk_pos, config, origin + u_vec + v_vec);
    let p3 = world_point(chunk_pos, config, origin + v_vec);
    let positions = if key.normal[axis] > 0 {
        [p0.to_array(), p1.to_array(), p2.to_array(), p3.to_array()]
    } else {
        [p3.to_array(), p2.to_array(), p1.to_array(), p0.to_array()]
    };

    buffers.push_quad(
        positions,
        key.normal.as_vec3().to_array(),
        atlas_uvs(key.tile, &config.atlas),
        key.ao,
        [key.light; 4],
        config.meshing.ao_strength,
        &config.lighting,
    );
}

fn world_point(chunk_pos: IVec3, config: &VoxelWorldConfig, local_vertex: IVec3) -> Vec3 {
    (chunk_pos * config.chunk_dims.as_ivec3() + local_vertex).as_vec3()
}
