use bevy::prelude::*;

use crate::{
    BlockRegistry, ChunkData, VoxelWorldConfig,
    meshing::{PaddedChunk, SampledBlock, build_chunk_meshes},
};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BenchmarkMeshSummary {
    pub opaque_quads: usize,
    pub cutout_quads: usize,
    pub opaque_vertices: usize,
    pub cutout_vertices: usize,
    pub opaque_indices: usize,
    pub cutout_indices: usize,
}

#[must_use]
pub fn mesh_chunk_with_unknown_neighbors(
    chunk_pos: IVec3,
    center: &ChunkData,
    registry: &BlockRegistry,
    config: &VoxelWorldConfig,
) -> BenchmarkMeshSummary {
    let padded = padded_from_center(center);
    let artifacts = build_chunk_meshes(chunk_pos, center, &padded, registry, config);
    let opaque = mesh_sizes(artifacts.opaque_mesh.as_ref());
    let cutout = mesh_sizes(artifacts.cutout_mesh.as_ref());

    BenchmarkMeshSummary {
        opaque_quads: artifacts.counts.opaque_quads,
        cutout_quads: artifacts.counts.cutout_quads,
        opaque_vertices: opaque.0,
        cutout_vertices: cutout.0,
        opaque_indices: opaque.1,
        cutout_indices: cutout.1,
    }
}

fn padded_from_center(center: &ChunkData) -> PaddedChunk {
    let mut padded = PaddedChunk::new_unknown(center.dims());
    for z in 0..center.dims().z as i32 {
        for y in 0..center.dims().y as i32 {
            for x in 0..center.dims().x as i32 {
                let local = UVec3::new(x as u32, y as u32, z as u32);
                padded.set(
                    IVec3::new(x + 1, y + 1, z + 1),
                    SampledBlock {
                        id: center.get(local),
                        known: true,
                    },
                );
            }
        }
    }
    padded
}

fn mesh_sizes(mesh: Option<&Mesh>) -> (usize, usize) {
    let Some(mesh) = mesh else {
        return (0, 0);
    };
    let vertices = mesh
        .attribute(Mesh::ATTRIBUTE_POSITION)
        .and_then(|values| values.as_float3())
        .map_or(0, |values| values.len());
    let indices = mesh.indices().map_or(0, |indices| indices.len());
    (vertices, indices)
}
