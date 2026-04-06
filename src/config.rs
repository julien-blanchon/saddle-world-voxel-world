use bevy::prelude::*;

#[derive(Clone, Debug, Reflect)]
pub struct AtlasConfig {
    pub asset_path: Option<String>,
    pub columns: u16,
    pub rows: u16,
    pub tile_size: UVec2,
    pub uv_inset: f32,
}

impl Default for AtlasConfig {
    fn default() -> Self {
        Self {
            asset_path: None,
            columns: 4,
            rows: 3,
            tile_size: UVec2::splat(16),
            uv_inset: 0.02,
        }
    }
}

#[derive(Clone, Debug, Reflect)]
pub struct MeshingConfig {
    pub enable_greedy: bool,
    pub ambient_occlusion: bool,
    pub ao_strength: f32,
    pub render_faces_against_unknown_neighbors: bool,
}

impl Default for MeshingConfig {
    fn default() -> Self {
        Self {
            enable_greedy: true,
            ambient_occlusion: true,
            ao_strength: 0.78,
            render_faces_against_unknown_neighbors: true,
        }
    }
}

#[derive(Clone, Debug, Reflect)]
pub struct LightingConfig {
    pub baked_ao: bool,
    pub flood_fill: bool,
    pub max_light_level: u8,
    pub sky_light_level: u8,
    pub light_falloff: u8,
    pub minimum_brightness: f32,
}

impl Default for LightingConfig {
    fn default() -> Self {
        Self {
            baked_ao: true,
            flood_fill: true,
            max_light_level: 15,
            sky_light_level: 15,
            light_falloff: 1,
            minimum_brightness: 0.18,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Reflect, PartialEq, Eq)]
pub enum SaveMode {
    #[default]
    Disabled,
    DeltaRegions,
}

#[derive(Clone, Debug, Reflect)]
pub struct SavePolicy {
    pub mode: SaveMode,
    pub root: String,
    pub region_dims: IVec3,
    pub autosave_interval_seconds: f32,
    pub max_chunks_per_frame: u32,
}

impl Default for SavePolicy {
    fn default() -> Self {
        Self {
            mode: SaveMode::Disabled,
            root: "local/voxel_world".into(),
            region_dims: IVec3::splat(8),
            autosave_interval_seconds: 10.0,
            max_chunks_per_frame: 2,
        }
    }
}

#[derive(Resource, Clone, Debug, Reflect)]
#[reflect(Resource)]
pub struct VoxelWorldConfig {
    pub chunk_dims: UVec3,
    pub request_radius: u32,
    pub keep_radius: u32,
    pub max_chunk_requests_per_frame: u32,
    pub max_chunk_unloads_per_frame: u32,
    pub max_generation_jobs_in_flight: usize,
    pub max_mesh_jobs_in_flight: usize,
    pub seed: u64,
    pub generator_version: u32,
    pub save_policy: SavePolicy,
    pub meshing: MeshingConfig,
    pub lighting: LightingConfig,
    pub atlas: AtlasConfig,
}

impl Default for VoxelWorldConfig {
    fn default() -> Self {
        Self {
            chunk_dims: UVec3::splat(16),
            request_radius: 6,
            keep_radius: 8,
            max_chunk_requests_per_frame: 12,
            max_chunk_unloads_per_frame: 8,
            max_generation_jobs_in_flight: 4,
            max_mesh_jobs_in_flight: 4,
            seed: 1,
            generator_version: 1,
            save_policy: SavePolicy::default(),
            meshing: MeshingConfig::default(),
            lighting: LightingConfig::default(),
            atlas: AtlasConfig::default(),
        }
    }
}

impl VoxelWorldConfig {
    pub fn validate(&self) -> Result<(), String> {
        if self.chunk_dims.min_element() < 4 {
            return Err("chunk dimensions must be at least 4 on every axis".into());
        }
        if self.keep_radius < self.request_radius {
            return Err("keep_radius must be greater than or equal to request_radius".into());
        }
        if self.max_generation_jobs_in_flight == 0 {
            return Err("at least one generation job must be allowed".into());
        }
        if self.max_mesh_jobs_in_flight == 0 {
            return Err("at least one mesh job must be allowed".into());
        }
        if self.atlas.columns == 0 || self.atlas.rows == 0 {
            return Err("atlas grid must have at least one column and row".into());
        }
        if self.lighting.max_light_level == 0 {
            return Err("lighting.max_light_level must be greater than zero".into());
        }
        if self.lighting.sky_light_level > self.lighting.max_light_level {
            return Err("lighting.sky_light_level cannot exceed lighting.max_light_level".into());
        }
        if self.lighting.light_falloff == 0 {
            return Err("lighting.light_falloff must be greater than zero".into());
        }
        if !(0.0..=1.0).contains(&self.lighting.minimum_brightness) {
            return Err("lighting.minimum_brightness must stay between 0.0 and 1.0".into());
        }
        Ok(())
    }
}
