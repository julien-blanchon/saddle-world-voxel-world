use bevy::prelude::*;

#[derive(Clone, Copy, Debug, Default, Reflect, PartialEq, Eq)]
pub enum VoxelDebugColorMode {
    #[default]
    ByLifecycle,
    ByDirty,
}

#[derive(Resource, Clone, Debug, Reflect)]
#[reflect(Resource)]
pub struct VoxelDebugConfig {
    pub show_chunk_bounds: bool,
    pub show_viewer_radii: bool,
    pub show_raycast: bool,
    pub color_mode: VoxelDebugColorMode,
}

impl Default for VoxelDebugConfig {
    fn default() -> Self {
        Self {
            show_chunk_bounds: false,
            show_viewer_radii: false,
            show_raycast: false,
            color_mode: VoxelDebugColorMode::ByLifecycle,
        }
    }
}

#[derive(Resource, Debug, Clone, Default, Reflect)]
#[reflect(Resource)]
pub struct VoxelWorldStats {
    pub loaded_chunks: usize,
    pub meshed_chunks: usize,
    pub dirty_chunks: usize,
    pub pending_generation_jobs: usize,
    pub pending_meshing_jobs: usize,
    pub pending_save_chunks: usize,
    pub generated_chunks: u64,
    pub remeshed_chunks: u64,
    pub unloaded_chunks: u64,
    pub block_modifications: u64,
    pub last_generation_time_ms: f32,
    pub last_meshing_time_ms: f32,
}
