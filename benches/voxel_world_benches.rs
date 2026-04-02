use std::collections::HashMap;

use bevy::prelude::*;
use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use saddle_world_voxel_world::{
    BlockId, BlockRegistry, BlockSampler, ChunkData, VoxelWorldConfig,
    benchmark_support::mesh_chunk_with_unknown_neighbors, decode_rle_blocks, encode_rle_blocks,
    generate_chunk, raycast_blocks, sample_generated_block,
};

fn criterion_config() -> Criterion {
    Criterion::default().sample_size(10)
}

fn bench_meshing(c: &mut Criterion) {
    let config = VoxelWorldConfig::default();
    let registry = BlockRegistry::default();
    let dims = config.chunk_dims;
    let terrain_like = generate_chunk(IVec3::ZERO, &config);
    let cases = [
        ("empty", ChunkData::new_filled(dims, BlockId::AIR)),
        ("full_solid", ChunkData::new_filled(dims, BlockId::STONE)),
        ("checkerboard", checkerboard_chunk(dims)),
        ("terrain_like", terrain_like),
        ("edit_heavy", edit_heavy_chunk(dims)),
    ];

    let mut group = c.benchmark_group("meshing");
    for (name, chunk) in cases {
        let summary = mesh_chunk_with_unknown_neighbors(IVec3::ZERO, &chunk, &registry, &config);
        group.bench_with_input(BenchmarkId::new("chunk", name), &chunk, |b, chunk| {
            b.iter(|| {
                black_box(mesh_chunk_with_unknown_neighbors(
                    IVec3::ZERO,
                    black_box(chunk),
                    black_box(&registry),
                    black_box(&config),
                ))
            });
        });
        group.throughput(criterion::Throughput::Elements(
            summary.opaque_indices as u64,
        ));
    }
    group.finish();
}

fn bench_compression(c: &mut Criterion) {
    let dims = UVec3::splat(16);
    let terrain = generate_chunk(IVec3::new(1, 0, -1), &VoxelWorldConfig::default());
    let cases = [
        ("empty", ChunkData::new_filled(dims, BlockId::AIR)),
        ("uniform", ChunkData::new_filled(dims, BlockId::STONE)),
        ("terrain_like", terrain),
    ];

    let mut encode_group = c.benchmark_group("compression_encode");
    for (name, chunk) in cases.iter() {
        let encoded = encode_rle_blocks(chunk);
        let _compression_ratio = encoded.len() as f32 / (chunk.blocks().len() * 2) as f32;
        encode_group.bench_with_input(BenchmarkId::new("chunk", name), chunk, |b, chunk| {
            b.iter(|| black_box(encode_rle_blocks(black_box(chunk))));
        });
    }
    encode_group.finish();

    let mut decode_group = c.benchmark_group("compression_decode");
    for (name, chunk) in cases.iter() {
        let encoded = encode_rle_blocks(chunk);
        decode_group.bench_with_input(BenchmarkId::new("chunk", name), &encoded, |b, encoded| {
            b.iter(|| black_box(decode_rle_blocks(dims, black_box(encoded)).unwrap()));
        });
    }
    decode_group.finish();
}

fn bench_raycast(c: &mut Criterion) {
    let registry = BlockRegistry::default();
    let sparse = Sampler {
        blocks: HashMap::from([
            (IVec3::new(8, 8, 8), BlockId::STONE),
            (IVec3::new(24, 8, 8), BlockId::STONE),
        ]),
    };
    let dense = Sampler {
        blocks: dense_raycast_map(),
    };

    let mut group = c.benchmark_group("raycast");
    group.bench_function("sparse", |b| {
        b.iter(|| {
            black_box(raycast_blocks(
                black_box(&sparse),
                black_box(&registry),
                Vec3::new(-10.0, 8.5, 8.5),
                Vec3::X,
                64.0,
            ))
        });
    });
    group.bench_function("dense", |b| {
        b.iter(|| {
            black_box(raycast_blocks(
                black_box(&dense),
                black_box(&registry),
                Vec3::new(-10.0, 8.5, 8.5),
                Vec3::X,
                64.0,
            ))
        });
    });
    group.finish();
}

fn bench_terrain_generation(c: &mut Criterion) {
    let config = VoxelWorldConfig::default();
    c.bench_function("terrain_generation_chunk", |b| {
        b.iter(|| {
            black_box(generate_chunk(
                black_box(IVec3::new(2, 0, -3)),
                black_box(&config),
            ))
        })
    });
    c.bench_function("terrain_sampling_hotspot", |b| {
        b.iter(|| {
            for x in -16..16 {
                for z in -16..16 {
                    black_box(sample_generated_block(IVec3::new(x, 12, z), &config));
                }
            }
        })
    });
}

fn checkerboard_chunk(dims: UVec3) -> ChunkData {
    let mut chunk = ChunkData::new_filled(dims, BlockId::AIR);
    for z in 0..dims.z {
        for y in 0..dims.y {
            for x in 0..dims.x {
                let block = if (x + y + z) % 2 == 0 {
                    BlockId::STONE
                } else {
                    BlockId::DIRT
                };
                chunk.set(UVec3::new(x, y, z), block);
            }
        }
    }
    chunk
}

fn edit_heavy_chunk(dims: UVec3) -> ChunkData {
    let mut chunk = ChunkData::new_filled(dims, BlockId::STONE);
    for z in 1..(dims.z - 1) {
        for y in 1..(dims.y - 1) {
            for x in 1..(dims.x - 1) {
                if (x + y + z) % 5 == 0 {
                    chunk.set(UVec3::new(x, y, z), BlockId::AIR);
                }
            }
        }
    }
    chunk
}

fn dense_raycast_map() -> HashMap<IVec3, BlockId> {
    let mut blocks = HashMap::new();
    for z in 0..16 {
        for y in 0..16 {
            for x in 0..16 {
                if (x + y + z) % 3 != 0 {
                    blocks.insert(IVec3::new(x, y, z), BlockId::STONE);
                }
            }
        }
    }
    blocks
}

struct Sampler {
    blocks: HashMap<IVec3, BlockId>,
}

impl BlockSampler for Sampler {
    fn sample_block(&self, world_pos: IVec3) -> Option<BlockId> {
        self.blocks.get(&world_pos).copied()
    }
}

criterion_group! {
    name = benches;
    config = criterion_config();
    targets = bench_meshing, bench_compression, bench_raycast, bench_terrain_generation
}
criterion_main!(benches);
