mod codec;
mod region;

pub use codec::{decode_rle_blocks, encode_rle_blocks};
pub use region::{load_chunk_delta, save_chunk_delta};

#[cfg(test)]
#[path = "codec_tests.rs"]
mod codec_tests;
#[cfg(test)]
#[path = "region_tests.rs"]
mod region_tests;
