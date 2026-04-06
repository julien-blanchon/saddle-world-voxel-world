use super::*;

#[test]
fn default_registry_contains_expected_blocks() {
    let registry = BlockRegistry::default();
    assert!(registry.contains(BlockId::SOLID));
    assert_eq!(registry.get(BlockId::CROSS).mesh_kind, MeshKind::Cross);
    assert!(!registry.get(BlockId::AIR).solid);
}

#[test]
fn atlas_tile_selection_uses_top_sides_bottom() {
    let registry = BlockRegistry::default();
    assert_eq!(registry.atlas_tile_for_face(BlockId::SOLID, IVec3::Y), 1);
    assert_eq!(
        registry.atlas_tile_for_face(BlockId::SOLID, IVec3::NEG_Y),
        1
    );
    assert_eq!(registry.atlas_tile_for_face(BlockId::SOLID, IVec3::X), 1);
}
