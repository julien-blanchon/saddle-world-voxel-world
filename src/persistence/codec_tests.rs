use bevy::prelude::*;

use super::{decode_rle_blocks, encode_rle_blocks};
use crate::{BlockId, ChunkData};

#[test]
fn empty_chunk_rle_roundtrip() {
    let dims = UVec3::splat(4);
    let chunk = ChunkData::new_filled(dims, BlockId::AIR);
    let bytes = encode_rle_blocks(&chunk);
    let decoded = decode_rle_blocks(dims, &bytes).unwrap();
    assert_eq!(chunk, decoded);
}

#[test]
fn random_like_chunk_rle_roundtrip() {
    let dims = UVec3::splat(4);
    let mut chunk = ChunkData::new_filled(dims, BlockId::AIR);
    for index in 0..(dims.x * dims.y * dims.z) {
        let local = chunk.local_from_index(index);
        let block = if index.is_multiple_of(3) {
            BlockId::SOLID
        } else {
            BlockId::SOLID_ALT
        };
        chunk.set(local, block);
    }
    let bytes = encode_rle_blocks(&chunk);
    let decoded = decode_rle_blocks(dims, &bytes).unwrap();
    assert_eq!(chunk, decoded);
}

#[test]
fn corrupt_rle_input_is_rejected() {
    assert!(decode_rle_blocks(UVec3::splat(4), &[1, 2, 3]).is_err());
}

#[test]
fn uniform_chunk_compresses_better_than_raw_payload() {
    let dims = UVec3::splat(16);
    let chunk = ChunkData::new_filled(dims, BlockId::SOLID);
    let encoded = encode_rle_blocks(&chunk);
    let raw_bytes = std::mem::size_of_val(chunk.blocks());
    assert!(encoded.len() < raw_bytes);
}
