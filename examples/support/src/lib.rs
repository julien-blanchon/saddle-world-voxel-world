use bevy::prelude::*;
use saddle_pane::prelude::*;
use saddle_world_voxel_world::{
    BlockDefinition, BlockFaceAtlas, BlockId, BlockRegistry, ChunkViewer, ChunkViewerSettings,
    CollisionKind, MaterialClass, MeshKind, SaveMode, SavePolicy, VoxelBlockSampler,
    VoxelDebugConfig, VoxelDecorationHook, VoxelWorldConfig, VoxelWorldGenerator,
};

#[derive(Component)]
pub struct ExampleViewer;

#[derive(Component)]
pub struct SecondaryViewer;

#[derive(Resource, Pane)]
#[pane(title = "Voxel World")]
pub struct VoxelExamplePane {
    #[pane(slider, min = 2.0, max = 10.0, step = 1.0)]
    pub request_radius: f32,
    #[pane(slider, min = 3.0, max = 12.0, step = 1.0)]
    pub keep_radius: f32,
    #[pane(toggle)]
    pub show_chunk_bounds: bool,
    #[pane(toggle)]
    pub show_viewer_radii: bool,
    #[pane(slider, min = 18.0, max = 72.0, step = 1.0)]
    pub orbit_radius: f32,
    #[pane(slider, min = 12.0, max = 42.0, step = 0.5)]
    pub orbit_height: f32,
    #[pane(slider, min = 0.04, max = 0.45, step = 0.01)]
    pub orbit_speed: f32,
}

impl Default for VoxelExamplePane {
    fn default() -> Self {
        Self {
            request_radius: 4.0,
            keep_radius: 6.0,
            show_chunk_bounds: false,
            show_viewer_radii: false,
            orbit_radius: 40.0,
            orbit_height: 28.0,
            orbit_speed: 0.18,
        }
    }
}

pub const SHOWCASE_GRASS: BlockId = BlockId(1);
pub const SHOWCASE_DIRT: BlockId = BlockId(2);
pub const SHOWCASE_STONE: BlockId = BlockId(3);
pub const SHOWCASE_SAND: BlockId = BlockId(4);
pub const SHOWCASE_WATER: BlockId = BlockId(5);
pub const SHOWCASE_TALL_GRASS: BlockId = BlockId(6);
pub const SHOWCASE_LAMP: BlockId = BlockId(7);
pub const SHOWCASE_WOOD: BlockId = BlockId(8);
pub const SHOWCASE_LEAVES: BlockId = BlockId(9);

pub const SHOWCASE_PLACEABLE_BLOCKS: [(BlockId, &str); 6] = [
    (SHOWCASE_GRASS, "Grass"),
    (SHOWCASE_DIRT, "Dirt"),
    (SHOWCASE_STONE, "Stone"),
    (SHOWCASE_SAND, "Sand"),
    (SHOWCASE_WOOD, "Wood"),
    (SHOWCASE_LEAVES, "Leaves"),
];

pub fn pane_plugins() -> (
    bevy_flair::FlairPlugin,
    bevy_input_focus::InputDispatchPlugin,
    bevy_ui_widgets::UiWidgetsPlugins,
    bevy_input_focus::tab_navigation::TabNavigationPlugin,
    saddle_pane::PanePlugin,
) {
    (
        bevy_flair::FlairPlugin,
        bevy_input_focus::InputDispatchPlugin,
        bevy_ui_widgets::UiWidgetsPlugins,
        bevy_input_focus::tab_navigation::TabNavigationPlugin,
        saddle_pane::PanePlugin,
    )
}

pub fn default_config() -> VoxelWorldConfig {
    VoxelWorldConfig {
        request_radius: 4,
        keep_radius: 6,
        max_chunk_requests_per_frame: 16,
        save_policy: SavePolicy {
            mode: SaveMode::Disabled,
            ..SavePolicy::default()
        },
        ..VoxelWorldConfig::default()
    }
}

pub fn showcase_registry() -> BlockRegistry {
    BlockRegistry::from_blocks(vec![
        BlockDefinition::air(),
        BlockDefinition {
            id: SHOWCASE_GRASS,
            name: "Grass".into(),
            mesh_kind: MeshKind::Cube,
            material_class: MaterialClass::Opaque,
            solid: true,
            opaque: true,
            collision: CollisionKind::Solid,
            emissive_level: 0,
            atlas: BlockFaceAtlas {
                top: 1,
                sides: 2,
                bottom: 3,
            },
        },
        BlockDefinition {
            id: SHOWCASE_DIRT,
            name: "Dirt".into(),
            mesh_kind: MeshKind::Cube,
            material_class: MaterialClass::Opaque,
            solid: true,
            opaque: true,
            collision: CollisionKind::Solid,
            emissive_level: 0,
            atlas: BlockFaceAtlas::uniform(3),
        },
        BlockDefinition {
            id: SHOWCASE_STONE,
            name: "Stone".into(),
            mesh_kind: MeshKind::Cube,
            material_class: MaterialClass::Opaque,
            solid: true,
            opaque: true,
            collision: CollisionKind::Solid,
            emissive_level: 0,
            atlas: BlockFaceAtlas::uniform(4),
        },
        BlockDefinition {
            id: SHOWCASE_SAND,
            name: "Sand".into(),
            mesh_kind: MeshKind::Cube,
            material_class: MaterialClass::Opaque,
            solid: true,
            opaque: true,
            collision: CollisionKind::Solid,
            emissive_level: 0,
            atlas: BlockFaceAtlas::uniform(5),
        },
        BlockDefinition {
            id: SHOWCASE_WATER,
            name: "Water".into(),
            mesh_kind: MeshKind::Empty,
            material_class: MaterialClass::Cutout,
            solid: false,
            opaque: false,
            collision: CollisionKind::None,
            emissive_level: 0,
            atlas: BlockFaceAtlas::uniform(6),
        },
        BlockDefinition {
            id: SHOWCASE_TALL_GRASS,
            name: "Tall Grass".into(),
            mesh_kind: MeshKind::Cross,
            material_class: MaterialClass::Cutout,
            solid: false,
            opaque: false,
            collision: CollisionKind::None,
            emissive_level: 0,
            atlas: BlockFaceAtlas::uniform(7),
        },
        BlockDefinition {
            id: SHOWCASE_LAMP,
            name: "Lamp".into(),
            mesh_kind: MeshKind::Cube,
            material_class: MaterialClass::Opaque,
            solid: true,
            opaque: true,
            collision: CollisionKind::Solid,
            emissive_level: 12,
            atlas: BlockFaceAtlas::uniform(8),
        },
        BlockDefinition {
            id: SHOWCASE_WOOD,
            name: "Wood".into(),
            mesh_kind: MeshKind::Cube,
            material_class: MaterialClass::Opaque,
            solid: true,
            opaque: true,
            collision: CollisionKind::Solid,
            emissive_level: 0,
            atlas: BlockFaceAtlas {
                top: 10,
                sides: 9,
                bottom: 10,
            },
        },
        BlockDefinition {
            id: SHOWCASE_LEAVES,
            name: "Leaves".into(),
            mesh_kind: MeshKind::Cube,
            material_class: MaterialClass::Cutout,
            solid: true,
            opaque: false,
            collision: CollisionKind::Solid,
            emissive_level: 0,
            atlas: BlockFaceAtlas::uniform(11),
        },
    ])
}

pub fn showcase_generator() -> VoxelWorldGenerator {
    VoxelWorldGenerator::new(ShowcaseTerrainSampler::default())
        .with_decoration(ShowcaseLampDecorator::default())
        .with_decoration(ShowcaseTreeDecorator::default())
        .with_decoration(ShowcaseFoliageDecorator::default())
}

pub fn sync_example_pane(
    pane: Res<VoxelExamplePane>,
    mut config: ResMut<VoxelWorldConfig>,
    mut debug: ResMut<VoxelDebugConfig>,
) {
    config.request_radius = pane.request_radius.round().max(1.0) as u32;
    config.keep_radius = pane.keep_radius.round().max(config.request_radius as f32) as u32;
    debug.show_chunk_bounds = pane.show_chunk_bounds;
    debug.show_viewer_radii = pane.show_viewer_radii;
}

pub fn spawn_scene(mut commands: Commands) {
    commands.spawn((
        Name::new("Sun"),
        DirectionalLight {
            illuminance: light_consts::lux::OVERCAST_DAY,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.9, 0.8, 0.0)),
    ));

    commands.insert_resource(GlobalAmbientLight {
        brightness: 120.0,
        ..default()
    });

    commands.spawn((
        Name::new("Main Camera"),
        Camera3d::default(),
        ChunkViewer,
        ChunkViewerSettings {
            request_radius: 4,
            keep_radius: 6,
            priority: 10,
        },
        ExampleViewer,
        Transform::from_xyz(24.0, 26.0, 24.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

pub fn spawn_overlay(commands: &mut Commands, title: &str, instructions: &str) {
    commands.spawn((
        Name::new("Example Overlay"),
        Text::new(format!("{title}\n{instructions}")),
        TextFont {
            font_size: 16.0,
            ..default()
        },
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(12.0),
            top: Val::Px(12.0),
            width: Val::Px(420.0),
            padding: UiRect::all(Val::Px(12.0)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.04, 0.06, 0.10, 0.82)),
    ));
}

#[allow(dead_code)]
pub fn enable_debug(mut debug: ResMut<VoxelDebugConfig>) {
    debug.show_chunk_bounds = true;
    debug.show_viewer_radii = true;
}

pub fn spin_viewer(
    time: Res<Time>,
    pane: Res<VoxelExamplePane>,
    mut viewers: Query<&mut Transform, (With<ExampleViewer>, Without<SecondaryViewer>)>,
) {
    let angle = time.elapsed_secs() * pane.orbit_speed;
    for mut transform in &mut viewers {
        transform.translation = Vec3::new(
            angle.cos() * pane.orbit_radius,
            pane.orbit_height,
            angle.sin() * pane.orbit_radius,
        );
        transform.look_at(Vec3::new(0.0, 6.0, 0.0), Vec3::Y);
    }
}

#[derive(Clone, Debug)]
struct ShowcaseTerrainSampler {
    base_height: i32,
    height_amplitude: i32,
    height_frequency: f32,
    hill_octaves: u8,
    cave_frequency: f32,
    cave_threshold: f32,
    water_level: i32,
}

impl Default for ShowcaseTerrainSampler {
    fn default() -> Self {
        Self {
            base_height: 14,
            height_amplitude: 18,
            height_frequency: 0.012,
            hill_octaves: 4,
            cave_frequency: 0.06,
            cave_threshold: 0.68,
            water_level: 10,
        }
    }
}

impl VoxelBlockSampler for ShowcaseTerrainSampler {
    fn sample_block(&self, world_pos: IVec3, config: &VoxelWorldConfig) -> BlockId {
        let height = terrain_height_at(self, config.seed, world_pos.x, world_pos.z);
        let terrain_height = height.round() as i32;

        if world_pos.y > terrain_height {
            return if world_pos.y <= self.water_level {
                SHOWCASE_WATER
            } else {
                BlockId::AIR
            };
        }

        let cave = value3(
            config.seed ^ 0x0f0f_aaaa,
            Vec3::new(world_pos.x as f32, world_pos.y as f32, world_pos.z as f32)
                * self.cave_frequency,
        );
        if world_pos.y < terrain_height - 2 && cave > self.cave_threshold {
            return BlockId::AIR;
        }

        if world_pos.y == terrain_height {
            if terrain_height <= self.water_level + 1 {
                SHOWCASE_SAND
            } else {
                SHOWCASE_GRASS
            }
        } else if world_pos.y >= terrain_height - 3 {
            SHOWCASE_DIRT
        } else {
            SHOWCASE_STONE
        }
    }
}

#[derive(Clone, Debug)]
struct ShowcaseLampDecorator {
    threshold: f32,
}

impl Default for ShowcaseLampDecorator {
    fn default() -> Self {
        Self { threshold: 0.92 }
    }
}

impl VoxelDecorationHook for ShowcaseLampDecorator {
    fn decorate_block(
        &self,
        world_pos: IVec3,
        sampled: BlockId,
        config: &VoxelWorldConfig,
    ) -> Option<BlockId> {
        if sampled != SHOWCASE_GRASS {
            return None;
        }
        let lamp_noise = fbm2(
            config.seed ^ 0x44aa_9911,
            Vec2::new(world_pos.x as f32, world_pos.z as f32) * 0.07,
            1,
        );
        (lamp_noise > self.threshold).then_some(SHOWCASE_LAMP)
    }
}

#[derive(Clone, Debug)]
struct ShowcaseFoliageDecorator {
    foliage_chance: f32,
    water_level: i32,
}

impl Default for ShowcaseFoliageDecorator {
    fn default() -> Self {
        Self {
            foliage_chance: 0.08,
            water_level: 10,
        }
    }
}

impl VoxelDecorationHook for ShowcaseFoliageDecorator {
    fn decorate_block(
        &self,
        world_pos: IVec3,
        sampled: BlockId,
        config: &VoxelWorldConfig,
    ) -> Option<BlockId> {
        if sampled != BlockId::AIR {
            return None;
        }
        let terrain = ShowcaseTerrainSampler::default();
        let terrain_height =
            terrain_height_at(&terrain, config.seed, world_pos.x, world_pos.z).round() as i32;
        if world_pos.y != terrain_height + 1 || terrain_height <= self.water_level {
            return None;
        }
        let tall_grass_noise = fbm2(
            config.seed ^ 0x7171_1818,
            Vec2::new(world_pos.x as f32, world_pos.z as f32) * 0.24,
            2,
        );
        (tall_grass_noise > 1.0 - self.foliage_chance * 2.0).then_some(SHOWCASE_TALL_GRASS)
    }
}

#[derive(Clone, Debug)]
struct ShowcaseTreeDecorator {
    foliage_chance: f32,
    water_level: i32,
}

impl Default for ShowcaseTreeDecorator {
    fn default() -> Self {
        Self {
            foliage_chance: 0.08,
            water_level: 10,
        }
    }
}

impl VoxelDecorationHook for ShowcaseTreeDecorator {
    fn decorate_block(
        &self,
        world_pos: IVec3,
        sampled: BlockId,
        config: &VoxelWorldConfig,
    ) -> Option<BlockId> {
        if sampled != BlockId::AIR {
            return None;
        }
        let tree_chance = self.foliage_chance * 1.5;
        let terrain = ShowcaseTerrainSampler::default();

        for dz in -3..=3 {
            for dx in -3..=3 {
                let trunk_x = world_pos.x + dx;
                let trunk_z = world_pos.z + dz;
                let hash = tree_hash(config.seed, trunk_x, trunk_z);
                if hash > tree_chance {
                    continue;
                }

                let ground =
                    terrain_height_at(&terrain, config.seed, trunk_x, trunk_z).round() as i32;
                if ground <= self.water_level + 1 {
                    continue;
                }

                let tree_height = 4 + ((hash * 1000.0) as i32 % 3);
                let trunk_top = ground + tree_height;
                let canopy_center = trunk_top;
                let canopy_radius = 2;

                if world_pos.x == trunk_x
                    && world_pos.z == trunk_z
                    && world_pos.y > ground
                    && world_pos.y <= trunk_top
                {
                    return Some(SHOWCASE_WOOD);
                }

                let rel_x = world_pos.x - trunk_x;
                let rel_y = world_pos.y - canopy_center;
                let rel_z = world_pos.z - trunk_z;
                let dist_sq = rel_x * rel_x + rel_y * rel_y + rel_z * rel_z;
                if dist_sq <= canopy_radius * canopy_radius + 1
                    && world_pos.y >= canopy_center - 1
                    && world_pos.y <= canopy_center + canopy_radius
                    && !(world_pos.x == trunk_x
                        && world_pos.z == trunk_z
                        && world_pos.y <= trunk_top)
                {
                    return Some(SHOWCASE_LEAVES);
                }
            }
        }

        None
    }
}

fn terrain_height_at(terrain: &ShowcaseTerrainSampler, seed: u64, x: i32, z: i32) -> f32 {
    terrain.base_height as f32
        + fbm2(
            seed,
            Vec2::new(x as f32, z as f32) * terrain.height_frequency,
            terrain.hill_octaves,
        ) * terrain.height_amplitude as f32
}

fn hash(mut x: u64) -> u64 {
    x ^= x >> 30;
    x = x.wrapping_mul(0xbf58_476d_1ce4_e5b9);
    x ^= x >> 27;
    x = x.wrapping_mul(0x94d0_49bb_1331_11eb);
    x ^ (x >> 31)
}

fn hash3(seed: u64, x: i32, y: i32, z: i32) -> u64 {
    hash(
        seed ^ (x as u64).wrapping_mul(0x9e37_79b9)
            ^ (y as u64).wrapping_mul(0x517c_c1b7)
            ^ (z as u64).wrapping_mul(0x94d0_49bb),
    )
}

fn unit_from_hash(value: u64) -> f32 {
    ((value & 0xffff_ffff) as f32 / u32::MAX as f32) * 2.0 - 1.0
}

fn smoothstep(t: f32) -> f32 {
    t * t * (3.0 - 2.0 * t)
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

fn value2(seed: u64, point: Vec2) -> f32 {
    let cell = point.floor().as_ivec2();
    let frac = point.fract();
    let tx = smoothstep(frac.x);
    let ty = smoothstep(frac.y);
    let c00 = unit_from_hash(hash3(seed, cell.x, 0, cell.y));
    let c10 = unit_from_hash(hash3(seed, cell.x + 1, 0, cell.y));
    let c01 = unit_from_hash(hash3(seed, cell.x, 0, cell.y + 1));
    let c11 = unit_from_hash(hash3(seed, cell.x + 1, 0, cell.y + 1));
    lerp(lerp(c00, c10, tx), lerp(c01, c11, tx), ty)
}

fn value3(seed: u64, point: Vec3) -> f32 {
    let cell = point.floor().as_ivec3();
    let frac = point.fract();
    let tx = smoothstep(frac.x);
    let ty = smoothstep(frac.y);
    let tz = smoothstep(frac.z);

    let sample = |dx: i32, dy: i32, dz: i32| -> f32 {
        unit_from_hash(hash3(seed, cell.x + dx, cell.y + dy, cell.z + dz))
    };

    let c000 = sample(0, 0, 0);
    let c100 = sample(1, 0, 0);
    let c010 = sample(0, 1, 0);
    let c110 = sample(1, 1, 0);
    let c001 = sample(0, 0, 1);
    let c101 = sample(1, 0, 1);
    let c011 = sample(0, 1, 1);
    let c111 = sample(1, 1, 1);

    let a = lerp(lerp(c000, c100, tx), lerp(c010, c110, tx), ty);
    let b = lerp(lerp(c001, c101, tx), lerp(c011, c111, tx), ty);
    lerp(a, b, tz)
}

fn fbm2(seed: u64, point: Vec2, octaves: u8) -> f32 {
    let mut sum = 0.0;
    let mut amplitude = 1.0;
    let mut frequency = 1.0;
    let mut normalizer = 0.0;

    for octave in 0..octaves {
        sum += value2(seed.wrapping_add(octave as u64), point * frequency) * amplitude;
        normalizer += amplitude;
        amplitude *= 0.5;
        frequency *= 2.0;
    }

    if normalizer == 0.0 {
        0.0
    } else {
        sum / normalizer
    }
}

fn tree_hash(seed: u64, x: i32, z: i32) -> f32 {
    let hash = hash(
        seed.wrapping_add(0xdead_beef)
            ^ (x as u64).wrapping_mul(0x9e37_79b9)
            ^ (z as u64).wrapping_mul(0x94d0_49bb),
    );
    (hash & 0xffff_ffff) as f32 / u32::MAX as f32
}
