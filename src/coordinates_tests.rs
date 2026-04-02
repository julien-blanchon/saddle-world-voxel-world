use super::*;

#[test]
fn world_to_chunk_local_handles_positive_values() {
    let dims = UVec3::splat(16);
    let (chunk, local) = world_to_chunk_local(IVec3::new(17, 31, 32), dims);
    assert_eq!(chunk, IVec3::new(1, 1, 2));
    assert_eq!(local, UVec3::new(1, 15, 0));
}

#[test]
fn world_to_chunk_local_handles_negative_values() {
    let dims = UVec3::splat(16);
    let (chunk, local) = world_to_chunk_local(IVec3::new(-1, -16, -17), dims);
    assert_eq!(chunk, IVec3::new(-1, -1, -2));
    assert_eq!(local, UVec3::new(15, 0, 15));
}

#[test]
fn chunk_local_roundtrips() {
    let dims = UVec3::new(16, 8, 16);
    let world = IVec3::new(-17, 7, 33);
    let (chunk, local) = world_to_chunk_local(world, dims);
    assert_eq!(local_to_world(chunk, local, dims), world);
}

#[test]
fn boundary_values_map_to_expected_chunks() {
    let dims = UVec3::splat(16);
    assert_eq!(
        world_to_chunk_local(IVec3::ZERO, dims),
        (IVec3::ZERO, UVec3::ZERO)
    );
    assert_eq!(
        world_to_chunk_local(IVec3::splat(15), dims),
        (IVec3::ZERO, UVec3::splat(15))
    );
    assert_eq!(
        world_to_chunk_local(IVec3::splat(16), dims),
        (IVec3::ONE, UVec3::ZERO)
    );
    assert_eq!(
        world_to_chunk_local(IVec3::splat(-1), dims),
        (IVec3::NEG_ONE, UVec3::splat(15))
    );
    assert_eq!(
        world_to_chunk_local(IVec3::splat(-16), dims),
        (IVec3::NEG_ONE, UVec3::ZERO)
    );
}

#[test]
fn neighboring_chunks_report_boundaries() {
    let dims = UVec3::splat(16);
    let neighbors = neighboring_chunks_for_boundary(UVec3::new(0, 15, 8), dims);
    assert!(neighbors.contains(&IVec3::NEG_X));
    assert!(neighbors.contains(&IVec3::Y));
    assert_eq!(neighbors.len(), 2);
}
