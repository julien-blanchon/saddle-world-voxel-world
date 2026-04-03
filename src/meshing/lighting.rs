use std::collections::VecDeque;

use bevy::prelude::*;

use crate::{
    block::BlockRegistry,
    config::LightingConfig,
    meshing::{PaddedChunk, SampledBlock},
};

#[derive(Clone, Debug)]
pub struct LightField {
    dims: UVec3,
    levels: Vec<u8>,
}

impl LightField {
    #[must_use]
    pub fn new(dims: UVec3) -> Self {
        Self {
            dims,
            levels: vec![0; (dims.x * dims.y * dims.z) as usize],
        }
    }

    #[must_use]
    pub fn get(&self, position: IVec3) -> u8 {
        self.index(position).map_or(0, |index| self.levels[index])
    }

    pub fn set_max(&mut self, position: IVec3, level: u8) -> bool {
        let Some(index) = self.index(position) else {
            return false;
        };
        if level > self.levels[index] {
            self.levels[index] = level;
            true
        } else {
            false
        }
    }

    fn index(&self, position: IVec3) -> Option<usize> {
        if position.cmplt(IVec3::ZERO).any() {
            return None;
        }
        let position = position.as_uvec3();
        (position.x < self.dims.x && position.y < self.dims.y && position.z < self.dims.z)
            .then_some(
                (position.x + self.dims.x * (position.y + self.dims.y * position.z)) as usize,
            )
    }
}

#[must_use]
pub fn build_light_field(
    padded: &PaddedChunk,
    registry: &BlockRegistry,
    config: &LightingConfig,
) -> LightField {
    let dims = padded.dims();
    let mut field = LightField::new(dims);
    let mut queue = VecDeque::new();
    let max_light = config.max_light_level;
    let sky_light = config.sky_light_level.min(max_light);

    for z in 0..dims.z as i32 {
        for x in 0..dims.x as i32 {
            for y in (0..dims.y as i32).rev() {
                let position = IVec3::new(x, y, z);
                let sample = padded.get(position);
                if blocks_light(sample, registry) {
                    break;
                }
                if sample.known && field.set_max(position, sky_light) {
                    queue.push_back((position, sky_light));
                }
            }
        }
    }

    for z in 0..dims.z as i32 {
        for y in 0..dims.y as i32 {
            for x in 0..dims.x as i32 {
                let position = IVec3::new(x, y, z);
                let sample = padded.get(position);
                let emit = if sample.known {
                    registry.get(sample.id).emissive_level.min(max_light)
                } else {
                    0
                };
                if emit == 0 {
                    continue;
                }
                let propagated = emit.saturating_sub(config.light_falloff);
                if propagated == 0 {
                    continue;
                }
                for offset in FACE_NEIGHBORS {
                    let neighbor = position + offset;
                    if lets_light_pass(padded.get(neighbor), registry)
                        && field.set_max(neighbor, propagated)
                    {
                        queue.push_back((neighbor, propagated));
                    }
                }
            }
        }
    }

    while let Some((position, level)) = queue.pop_front() {
        if field.get(position) != level || level <= config.light_falloff {
            continue;
        }
        let next = level.saturating_sub(config.light_falloff);
        if next == 0 {
            continue;
        }
        for offset in FACE_NEIGHBORS {
            let neighbor = position + offset;
            if lets_light_pass(padded.get(neighbor), registry) && field.set_max(neighbor, next) {
                queue.push_back((neighbor, next));
            }
        }
    }

    field
}

#[must_use]
pub fn face_light_level(
    field: &LightField,
    solid_position: IVec3,
    normal: IVec3,
    emissive_level: u8,
    config: &LightingConfig,
) -> u8 {
    field
        .get(solid_position + normal)
        .max(emissive_level.min(config.max_light_level))
}

#[must_use]
pub fn brightness_for_level(level: u8, config: &LightingConfig) -> f32 {
    let max_light = config.max_light_level.max(1) as f32;
    let normalized = level.min(config.max_light_level) as f32 / max_light;
    config.minimum_brightness + normalized * (1.0 - config.minimum_brightness)
}

fn lets_light_pass(sample: SampledBlock, registry: &BlockRegistry) -> bool {
    sample.known && !registry.get(sample.id).opaque
}

fn blocks_light(sample: SampledBlock, registry: &BlockRegistry) -> bool {
    sample.known && registry.get(sample.id).opaque
}

const FACE_NEIGHBORS: [IVec3; 6] = [
    IVec3::X,
    IVec3::NEG_X,
    IVec3::Y,
    IVec3::NEG_Y,
    IVec3::Z,
    IVec3::NEG_Z,
];

#[cfg(test)]
#[path = "lighting_tests.rs"]
mod tests;
