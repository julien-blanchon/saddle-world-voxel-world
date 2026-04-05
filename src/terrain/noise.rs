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

pub fn value2(seed: u64, point: Vec2) -> f32 {
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

pub fn value3(seed: u64, point: Vec3) -> f32 {
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

pub fn fbm2(seed: u64, point: Vec2, octaves: u8) -> f32 {
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

/// Deterministic hash for tree placement — returns 0.0..1.0.
pub fn tree_hash(seed: u64, x: i32, z: i32) -> f32 {
    let h = hash(
        seed.wrapping_add(0xdead_beef)
            ^ (x as u64).wrapping_mul(0x9e37_79b9)
            ^ (z as u64).wrapping_mul(0x94d0_49bb),
    );
    (h & 0xffff_ffff) as f32 / u32::MAX as f32
}

use bevy::prelude::*;
