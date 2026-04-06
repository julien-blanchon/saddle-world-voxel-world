use bevy::prelude::*;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord, Reflect)]
#[reflect(Default)]
pub struct BlockId(pub u16);

impl BlockId {
    pub const AIR: Self = Self(0);
    pub const SOLID: Self = Self(1);
    pub const SOLID_ALT: Self = Self(2);
    pub const SOLID_ACCENT: Self = Self(3);
    pub const NON_SOLID: Self = Self(4);
    pub const CROSS: Self = Self(5);
    pub const EMISSIVE: Self = Self(6);
    pub const CUTOUT_SOLID: Self = Self(7);
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Reflect)]
pub enum MeshKind {
    #[default]
    Empty,
    Cube,
    Cross,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Reflect)]
pub enum MaterialClass {
    #[default]
    Opaque,
    Cutout,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Reflect)]
pub enum CollisionKind {
    #[default]
    None,
    Solid,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Reflect)]
pub struct BlockFaceAtlas {
    pub top: u16,
    pub sides: u16,
    pub bottom: u16,
}

impl BlockFaceAtlas {
    #[must_use]
    pub const fn uniform(tile: u16) -> Self {
        Self {
            top: tile,
            sides: tile,
            bottom: tile,
        }
    }
}

#[derive(Clone, Debug, Reflect)]
pub struct BlockDefinition {
    pub id: BlockId,
    pub name: String,
    pub mesh_kind: MeshKind,
    pub material_class: MaterialClass,
    pub solid: bool,
    pub opaque: bool,
    pub collision: CollisionKind,
    pub emissive_level: u8,
    pub atlas: BlockFaceAtlas,
}

impl BlockDefinition {
    #[must_use]
    pub fn air() -> Self {
        Self {
            id: BlockId::AIR,
            name: "Air".into(),
            mesh_kind: MeshKind::Empty,
            material_class: MaterialClass::Opaque,
            solid: false,
            opaque: false,
            collision: CollisionKind::None,
            emissive_level: 0,
            atlas: BlockFaceAtlas::uniform(0),
        }
    }

    #[must_use]
    pub fn renders_cube_face(&self) -> bool {
        self.mesh_kind == MeshKind::Cube && self.solid
    }

    #[must_use]
    pub fn culls_opaque_faces(&self) -> bool {
        self.renders_cube_face() && self.opaque
    }
}

#[derive(Resource, Clone, Debug, Reflect)]
#[reflect(Resource)]
pub struct BlockRegistry {
    blocks: Vec<BlockDefinition>,
}

impl Default for BlockRegistry {
    fn default() -> Self {
        Self::from_blocks(vec![
            BlockDefinition::air(),
            BlockDefinition {
                id: BlockId::SOLID,
                name: "Solid".into(),
                mesh_kind: MeshKind::Cube,
                material_class: MaterialClass::Opaque,
                solid: true,
                opaque: true,
                collision: CollisionKind::Solid,
                emissive_level: 0,
                atlas: BlockFaceAtlas::uniform(1),
            },
            BlockDefinition {
                id: BlockId::SOLID_ALT,
                name: "Solid Alt".into(),
                mesh_kind: MeshKind::Cube,
                material_class: MaterialClass::Opaque,
                solid: true,
                opaque: true,
                collision: CollisionKind::Solid,
                emissive_level: 0,
                atlas: BlockFaceAtlas::uniform(2),
            },
            BlockDefinition {
                id: BlockId::SOLID_ACCENT,
                name: "Solid Accent".into(),
                mesh_kind: MeshKind::Cube,
                material_class: MaterialClass::Opaque,
                solid: true,
                opaque: true,
                collision: CollisionKind::Solid,
                emissive_level: 0,
                atlas: BlockFaceAtlas::uniform(3),
            },
            BlockDefinition {
                id: BlockId::NON_SOLID,
                name: "Non Solid".into(),
                mesh_kind: MeshKind::Empty,
                material_class: MaterialClass::Cutout,
                solid: false,
                opaque: false,
                collision: CollisionKind::None,
                emissive_level: 0,
                atlas: BlockFaceAtlas::uniform(4),
            },
            BlockDefinition {
                id: BlockId::CROSS,
                name: "Cross".into(),
                mesh_kind: MeshKind::Cross,
                material_class: MaterialClass::Cutout,
                solid: false,
                opaque: false,
                collision: CollisionKind::None,
                emissive_level: 0,
                atlas: BlockFaceAtlas::uniform(5),
            },
            BlockDefinition {
                id: BlockId::EMISSIVE,
                name: "Emissive".into(),
                mesh_kind: MeshKind::Cube,
                material_class: MaterialClass::Opaque,
                solid: true,
                opaque: true,
                collision: CollisionKind::Solid,
                emissive_level: 12,
                atlas: BlockFaceAtlas::uniform(6),
            },
            BlockDefinition {
                id: BlockId::CUTOUT_SOLID,
                name: "Cutout Solid".into(),
                mesh_kind: MeshKind::Cube,
                material_class: MaterialClass::Cutout,
                solid: true,
                opaque: false,
                collision: CollisionKind::Solid,
                emissive_level: 0,
                atlas: BlockFaceAtlas::uniform(7),
            },
        ])
    }
}

impl BlockRegistry {
    #[must_use]
    pub fn from_blocks(blocks: Vec<BlockDefinition>) -> Self {
        let max_index = blocks
            .iter()
            .map(|block| block.id.0 as usize)
            .max()
            .unwrap_or(BlockId::AIR.0 as usize)
            .max(BlockId::AIR.0 as usize);
        let mut indexed = vec![BlockDefinition::air(); max_index + 1];
        for block in blocks {
            let index = block.id.0 as usize;
            indexed[index] = block;
        }
        Self { blocks: indexed }
    }

    #[must_use]
    pub fn get(&self, id: BlockId) -> &BlockDefinition {
        self.blocks
            .get(id.0 as usize)
            .unwrap_or_else(|| &self.blocks[BlockId::AIR.0 as usize])
    }

    #[must_use]
    pub fn contains(&self, id: BlockId) -> bool {
        self.blocks.get(id.0 as usize).is_some()
    }

    #[must_use]
    pub fn definitions(&self) -> &[BlockDefinition] {
        &self.blocks
    }

    #[must_use]
    pub fn atlas_tile_for_face(&self, id: BlockId, normal: IVec3) -> u16 {
        let definition = self.get(id);
        if normal == IVec3::Y {
            definition.atlas.top
        } else if normal == IVec3::NEG_Y {
            definition.atlas.bottom
        } else {
            definition.atlas.sides
        }
    }

    #[must_use]
    pub fn max_atlas_tile(&self) -> u16 {
        self.blocks
            .iter()
            .flat_map(|block| [block.atlas.top, block.atlas.sides, block.atlas.bottom])
            .max()
            .unwrap_or(0)
    }
}

#[cfg(test)]
#[path = "block_tests.rs"]
mod tests;
