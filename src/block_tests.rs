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

#[test]
fn sparse_registry_holes_are_not_treated_as_defined_blocks() {
    let registry = BlockRegistry::from_blocks(vec![
        BlockDefinition::air(),
        BlockDefinition {
            id: BlockId(9),
            name: "Sparse Solid".into(),
            mesh_kind: MeshKind::Cube,
            material_class: MaterialClass::Opaque,
            solid: true,
            opaque: true,
            collision: CollisionKind::Solid,
            emissive_level: 0,
            atlas: BlockFaceAtlas::uniform(3),
        },
    ]);

    assert!(!registry.contains(BlockId(5)));
    assert!(registry.contains(BlockId(9)));
    assert_eq!(registry.get(BlockId(5)).id, BlockId::AIR);
}
