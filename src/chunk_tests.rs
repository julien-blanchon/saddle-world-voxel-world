use super::*;

#[test]
fn chunk_index_mapping_is_stable() {
    let dims = UVec3::new(4, 4, 4);
    assert_eq!(index_for(dims, UVec3::new(0, 0, 0)), 0);
    assert_eq!(index_for(dims, UVec3::new(1, 0, 0)), 1);
    assert_eq!(index_for(dims, UVec3::new(0, 1, 0)), 4);
    assert_eq!(index_for(dims, UVec3::new(0, 0, 1)), 16);
}

#[test]
fn chunk_get_set_corners_and_center() {
    let dims = UVec3::new(4, 4, 4);
    let mut chunk = ChunkData::new_filled(dims, BlockId::AIR);
    chunk.set(UVec3::ZERO, BlockId::STONE);
    chunk.set(UVec3::new(3, 3, 3), BlockId::DIRT);
    chunk.set(UVec3::new(2, 2, 2), BlockId::GRASS);
    assert_eq!(chunk.get(UVec3::ZERO), BlockId::STONE);
    assert_eq!(chunk.get(UVec3::new(3, 3, 3)), BlockId::DIRT);
    assert_eq!(chunk.get(UVec3::new(2, 2, 2)), BlockId::GRASS);
}

#[test]
fn chunk_uniform_and_empty_checks_work() {
    let dims = UVec3::splat(4);
    let chunk = ChunkData::new_filled(dims, BlockId::AIR);
    assert!(chunk.is_empty());
    assert!(chunk.is_uniform(BlockId::AIR));
}

#[test]
fn delta_from_overrides_preserves_indices() {
    let mut overrides = std::collections::BTreeMap::new();
    overrides.insert(3, BlockId::DIRT);
    overrides.insert(11, BlockId::STONE);
    let delta = delta_from_overrides(&overrides);
    assert_eq!(delta[0].local_index, 3);
    assert_eq!(delta[1].block, BlockId::STONE);
}
