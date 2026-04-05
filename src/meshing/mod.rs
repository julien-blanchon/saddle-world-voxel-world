mod ao;
mod cross;
mod greedy;
mod lighting;

use bevy::{
    asset::RenderAssetUsages, mesh::Indices, prelude::*, render::render_resource::PrimitiveTopology,
};

use crate::{
    block::{BlockId, BlockRegistry, MeshKind},
    chunk::ChunkData,
    config::VoxelWorldConfig,
};

pub use ao::ambient_occlusion_for_face;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SampledBlock {
    pub id: BlockId,
    pub known: bool,
}

impl SampledBlock {
    #[must_use]
    pub fn unknown() -> Self {
        Self {
            id: BlockId::AIR,
            known: false,
        }
    }
}

#[derive(Clone, Debug)]
pub struct PaddedChunk {
    dims: UVec3,
    blocks: Vec<SampledBlock>,
}

impl PaddedChunk {
    #[must_use]
    pub fn new_unknown(chunk_dims: UVec3) -> Self {
        let dims = chunk_dims + UVec3::splat(2);
        Self {
            dims,
            blocks: vec![SampledBlock::unknown(); (dims.x * dims.y * dims.z) as usize],
        }
    }
    #[must_use]
    pub fn get(&self, padded: IVec3) -> SampledBlock {
        if padded.cmplt(IVec3::ZERO).any() {
            return SampledBlock::unknown();
        }
        let padded = padded.as_uvec3();
        if padded.x >= self.dims.x || padded.y >= self.dims.y || padded.z >= self.dims.z {
            return SampledBlock::unknown();
        }
        let index = padded.x + self.dims.x * (padded.y + self.dims.y * padded.z);
        self.blocks[index as usize]
    }

    pub fn set(&mut self, padded: IVec3, sample: SampledBlock) {
        let padded = padded.as_uvec3();
        let index = padded.x + self.dims.x * (padded.y + self.dims.y * padded.z);
        self.blocks[index as usize] = sample;
    }

    #[must_use]
    pub fn dims(&self) -> UVec3 {
        self.dims
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct MeshCounts {
    pub opaque_quads: usize,
    pub cutout_quads: usize,
}

#[derive(Default)]
struct MeshBuffers {
    positions: Vec<[f32; 3]>,
    normals: Vec<[f32; 3]>,
    uvs: Vec<[f32; 2]>,
    colors: Vec<[f32; 4]>,
    indices: Vec<u32>,
}

impl MeshBuffers {
    fn push_quad(
        &mut self,
        positions: [[f32; 3]; 4],
        normal: [f32; 3],
        uvs: [[f32; 2]; 4],
        ao: [u8; 4],
        light: [u8; 4],
        ao_strength: f32,
        lighting: &crate::config::LightingConfig,
    ) {
        let base = self.positions.len() as u32;
        self.positions.extend_from_slice(&positions);
        self.normals.extend_from_slice(&[normal; 4]);
        self.uvs.extend_from_slice(&uvs);
        self.colors
            .extend(ao.into_iter().zip(light).map(|(ao_value, light_value)| {
                let ao_shade = 1.0 - (3_u8.saturating_sub(ao_value)) as f32 * 0.22 * ao_strength;
                let light_shade = lighting::brightness_for_level(light_value, lighting);
                let shade = ao_shade * light_shade;
                [shade, shade, shade, 1.0]
            }));
        self.indices
            .extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
    }

    #[must_use]
    fn build_mesh(self) -> Option<Mesh> {
        if self.indices.is_empty() {
            return None;
        }

        let mut mesh = Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::default(),
        );
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, self.positions);
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, self.normals);
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, self.uvs);
        mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, self.colors);
        mesh.insert_indices(Indices::U32(self.indices));
        Some(mesh)
    }
}

#[derive(Default)]
pub struct ChunkMeshArtifacts {
    pub opaque_mesh: Option<Mesh>,
    pub cutout_mesh: Option<Mesh>,
    pub counts: MeshCounts,
}

pub fn build_chunk_meshes(
    chunk_pos: IVec3,
    center: &ChunkData,
    padded: &PaddedChunk,
    registry: &BlockRegistry,
    config: &VoxelWorldConfig,
) -> ChunkMeshArtifacts {
    let mut opaque = MeshBuffers::default();
    let mut cutout = MeshBuffers::default();
    let mut counts = MeshCounts::default();
    let light_field = config
        .lighting
        .flood_fill
        .then(|| lighting::build_light_field(padded, registry, &config.lighting));

    greedy::emit_greedy_quads(
        chunk_pos,
        center,
        padded,
        light_field.as_ref(),
        registry,
        config,
        &mut opaque,
        &mut counts,
    );
    cross::emit_cross_quads(
        chunk_pos,
        center,
        light_field.as_ref(),
        registry,
        config,
        &mut cutout,
        &mut counts,
    );
    cross::emit_cutout_cube_faces(
        chunk_pos,
        center,
        padded,
        light_field.as_ref(),
        registry,
        config,
        &mut cutout,
        &mut counts,
    );

    ChunkMeshArtifacts {
        opaque_mesh: opaque.build_mesh(),
        cutout_mesh: cutout.build_mesh(),
        counts,
    }
}

fn atlas_uvs(tile: u16, atlas: &crate::config::AtlasConfig) -> [[f32; 2]; 4] {
    let columns = atlas.columns.max(1) as f32;
    let rows = atlas.rows.max(1) as f32;
    let tile = tile as u32;
    let column = tile % atlas.columns as u32;
    let row = tile / atlas.columns as u32;
    let inset_u = atlas.uv_inset / columns;
    let inset_v = atlas.uv_inset / rows;
    let u0 = column as f32 / columns + inset_u;
    let v0 = row as f32 / rows + inset_v;
    let u1 = (column + 1) as f32 / columns - inset_u;
    let v1 = (row + 1) as f32 / rows - inset_v;
    [[u0, v1], [u1, v1], [u1, v0], [u0, v0]]
}

fn should_emit_cube(sample: SampledBlock, registry: &BlockRegistry) -> bool {
    sample.known && registry.get(sample.id).mesh_kind == MeshKind::Cube
}

fn culls_neighbor(sample: SampledBlock, registry: &BlockRegistry) -> bool {
    sample.known && registry.get(sample.id).culls_opaque_faces()
}

fn atlas_tile_for(registry: &BlockRegistry, id: BlockId, normal: IVec3) -> u16 {
    registry.atlas_tile_for_face(id, normal)
}

#[cfg(test)]
#[path = "ao_tests.rs"]
mod ao_tests;
#[cfg(test)]
#[path = "cross_tests.rs"]
mod cross_tests;
#[cfg(test)]
#[path = "greedy_tests.rs"]
mod greedy_tests;
