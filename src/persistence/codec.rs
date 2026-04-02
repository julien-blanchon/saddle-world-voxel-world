use crate::{block::BlockId, chunk::ChunkData};

pub fn encode_rle_blocks(chunk: &ChunkData) -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&(chunk.blocks().len() as u32).to_le_bytes());
    let mut iter = chunk.blocks().iter().copied();
    let Some(mut current) = iter.next() else {
        return bytes;
    };
    let mut run = 1_u32;
    for block in iter {
        if block == current && run < u32::MAX {
            run += 1;
        } else {
            bytes.extend_from_slice(&run.to_le_bytes());
            bytes.extend_from_slice(&current.0.to_le_bytes());
            current = block;
            run = 1;
        }
    }
    bytes.extend_from_slice(&run.to_le_bytes());
    bytes.extend_from_slice(&current.0.to_le_bytes());
    bytes
}

pub fn decode_rle_blocks(dims: bevy::prelude::UVec3, bytes: &[u8]) -> Result<ChunkData, String> {
    if bytes.len() < 4 {
        return Err("rle payload missing length header".into());
    }
    let expected = u32::from_le_bytes(bytes[0..4].try_into().unwrap()) as usize;
    let mut blocks = Vec::with_capacity(expected);
    let mut cursor = 4;
    while cursor + 6 <= bytes.len() {
        let run = u32::from_le_bytes(bytes[cursor..cursor + 4].try_into().unwrap());
        cursor += 4;
        let block = BlockId(u16::from_le_bytes(
            bytes[cursor..cursor + 2].try_into().unwrap(),
        ));
        cursor += 2;
        blocks.extend(std::iter::repeat_n(block, run as usize));
    }
    if blocks.len() != expected || expected != (dims.x * dims.y * dims.z) as usize {
        return Err("rle payload length mismatch".into());
    }
    Ok(ChunkData::new_filled(dims, BlockId::AIR).tap_mut(|chunk| {
        chunk.blocks_mut().copy_from_slice(&blocks);
    }))
}

trait TapMut: Sized {
    fn tap_mut(self, f: impl FnOnce(&mut Self)) -> Self;
}

impl<T> TapMut for T {
    fn tap_mut(mut self, f: impl FnOnce(&mut Self)) -> Self {
        f(&mut self);
        self
    }
}
