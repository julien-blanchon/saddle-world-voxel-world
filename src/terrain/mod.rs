mod generator;
mod noise;

pub use generator::{generate_chunk, sample_generated_block};

#[cfg(test)]
#[path = "terrain_tests.rs"]
mod tests;
