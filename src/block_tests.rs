use super::*;

#[test]
fn default_registry_contains_expected_blocks() {
    let registry = BlockRegistry::default();
    assert!(registry.contains(BlockId::GRASS));
    assert_eq!(registry.get(BlockId::TALL_GRASS).mesh_kind, MeshKind::Cross);
    assert_eq!(registry.get(BlockId::AIR).solid, false);
}

#[test]
fn atlas_tile_selection_uses_top_sides_bottom() {
    let registry = BlockRegistry::default();
    assert_eq!(registry.atlas_tile_for_face(BlockId::GRASS, IVec3::Y), 1);
    assert_eq!(
        registry.atlas_tile_for_face(BlockId::GRASS, IVec3::NEG_Y),
        3
    );
    assert_eq!(registry.atlas_tile_for_face(BlockId::GRASS, IVec3::X), 2);
}
