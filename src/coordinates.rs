use bevy::prelude::*;

#[must_use]
pub fn world_to_chunk(world: IVec3, chunk_dims: UVec3) -> IVec3 {
    IVec3::new(
        world.x.div_euclid(chunk_dims.x as i32),
        world.y.div_euclid(chunk_dims.y as i32),
        world.z.div_euclid(chunk_dims.z as i32),
    )
}

#[must_use]
pub fn world_to_local(world: IVec3, chunk_dims: UVec3) -> UVec3 {
    UVec3::new(
        world.x.rem_euclid(chunk_dims.x as i32) as u32,
        world.y.rem_euclid(chunk_dims.y as i32) as u32,
        world.z.rem_euclid(chunk_dims.z as i32) as u32,
    )
}

#[must_use]
pub fn world_to_chunk_local(world: IVec3, chunk_dims: UVec3) -> (IVec3, UVec3) {
    (
        world_to_chunk(world, chunk_dims),
        world_to_local(world, chunk_dims),
    )
}

#[must_use]
pub fn chunk_origin(chunk: IVec3, chunk_dims: UVec3) -> IVec3 {
    IVec3::new(
        chunk.x * chunk_dims.x as i32,
        chunk.y * chunk_dims.y as i32,
        chunk.z * chunk_dims.z as i32,
    )
}

#[must_use]
pub fn chunk_translation(chunk: IVec3, chunk_dims: UVec3) -> Vec3 {
    chunk_origin(chunk, chunk_dims).as_vec3()
}

#[must_use]
pub fn local_to_world(chunk: IVec3, local: UVec3, chunk_dims: UVec3) -> IVec3 {
    chunk_origin(chunk, chunk_dims) + local.as_ivec3()
}

#[must_use]
pub fn is_on_chunk_boundary(local: UVec3, chunk_dims: UVec3) -> bool {
    local.x == 0
        || local.y == 0
        || local.z == 0
        || local.x + 1 == chunk_dims.x
        || local.y + 1 == chunk_dims.y
        || local.z + 1 == chunk_dims.z
}

#[must_use]
pub fn neighboring_chunks_for_boundary(local: UVec3, chunk_dims: UVec3) -> Vec<IVec3> {
    let mut neighbors = Vec::new();
    for (axis, low, high) in [
        (IVec3::NEG_X, local.x == 0, local.x + 1 == chunk_dims.x),
        (IVec3::NEG_Y, local.y == 0, local.y + 1 == chunk_dims.y),
        (IVec3::NEG_Z, local.z == 0, local.z + 1 == chunk_dims.z),
    ] {
        if low {
            neighbors.push(axis);
        }
        if high {
            neighbors.push(-axis);
        }
    }
    neighbors
}

#[cfg(test)]
#[path = "coordinates_tests.rs"]
mod tests;
