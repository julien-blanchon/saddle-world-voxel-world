use bevy::prelude::*;

#[derive(Component, Debug, Default, Clone, Copy, Reflect)]
#[reflect(Component, Default)]
pub struct ChunkViewer;

#[derive(Component, Debug, Default, Clone, Copy, Reflect)]
#[reflect(Component)]
pub struct ChunkViewerSettings {
    pub request_radius: u32,
    pub keep_radius: u32,
    pub priority: i32,
}

#[cfg(test)]
#[path = "viewer_tests.rs"]
mod tests;
