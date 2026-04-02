use std::path::PathBuf;

use bevy::prelude::*;

use super::{load_chunk_delta, save_chunk_delta};
use crate::{BlockId, SaveMode, SavePolicy};

#[test]
fn region_roundtrip_persists_chunk_deltas() {
    let root = tempfile_dir("voxel_world_region_roundtrip");
    let policy = SavePolicy {
        mode: SaveMode::DeltaRegions,
        root: root.clone(),
        ..SavePolicy::default()
    };
    let mut overrides = std::collections::BTreeMap::new();
    overrides.insert(1, BlockId::DIRT);
    overrides.insert(7, BlockId::STONE);
    save_chunk_delta(&policy, IVec3::new(2, 0, -1), 42, 7, 3, &overrides).unwrap();
    let loaded = load_chunk_delta(&policy, IVec3::new(2, 0, -1), 42, 7)
        .unwrap()
        .unwrap();
    assert_eq!(loaded.len(), 2);
    assert_eq!(loaded[0].block, BlockId::DIRT);
}

#[test]
fn version_mismatch_is_ignored() {
    let root = tempfile_dir("voxel_world_region_version_mismatch");
    let policy = SavePolicy {
        mode: SaveMode::DeltaRegions,
        root: root.clone(),
        ..SavePolicy::default()
    };
    let mut overrides = std::collections::BTreeMap::new();
    overrides.insert(2, BlockId::LAMP);
    save_chunk_delta(&policy, IVec3::ZERO, 42, 7, 1, &overrides).unwrap();

    assert!(
        load_chunk_delta(&policy, IVec3::ZERO, 42, 8)
            .unwrap()
            .is_none()
    );
    assert!(
        load_chunk_delta(&policy, IVec3::ZERO, 99, 7)
            .unwrap()
            .is_none()
    );
}

#[test]
fn corrupt_entry_is_skipped_without_poisoning_other_chunks() {
    let root = tempfile_dir("voxel_world_region_corruption");
    let policy = SavePolicy {
        mode: SaveMode::DeltaRegions,
        root: root.clone(),
        ..SavePolicy::default()
    };

    let mut first = std::collections::BTreeMap::new();
    first.insert(1, BlockId::DIRT);
    save_chunk_delta(&policy, IVec3::new(0, 0, 0), 7, 1, 1, &first).unwrap();

    let mut second = std::collections::BTreeMap::new();
    second.insert(9, BlockId::STONE);
    save_chunk_delta(&policy, IVec3::new(1, 0, 0), 7, 1, 1, &second).unwrap();

    let path = region_path(&root, IVec3::ZERO);
    let mut bytes = std::fs::read(&path).unwrap();
    corrupt_first_entry_checksum(&mut bytes);
    std::fs::write(&path, bytes).unwrap();

    assert!(
        load_chunk_delta(&policy, IVec3::new(0, 0, 0), 7, 1)
            .unwrap()
            .is_none()
    );
    let loaded_second = load_chunk_delta(&policy, IVec3::new(1, 0, 0), 7, 1)
        .unwrap()
        .unwrap();
    assert_eq!(loaded_second.len(), 1);
    assert_eq!(loaded_second[0].block, BlockId::STONE);
}

#[test]
fn empty_overrides_remove_region_file_entry() {
    let root = tempfile_dir("voxel_world_region_empty_entry");
    let policy = SavePolicy {
        mode: SaveMode::DeltaRegions,
        root: root.clone(),
        ..SavePolicy::default()
    };
    let mut overrides = std::collections::BTreeMap::new();
    overrides.insert(3, BlockId::DIRT);
    save_chunk_delta(&policy, IVec3::ZERO, 2, 1, 1, &overrides).unwrap();
    save_chunk_delta(
        &policy,
        IVec3::ZERO,
        2,
        1,
        2,
        &std::collections::BTreeMap::new(),
    )
    .unwrap();

    assert!(
        load_chunk_delta(&policy, IVec3::ZERO, 2, 1)
            .unwrap()
            .is_none()
    );
    assert!(!region_path(&root, IVec3::ZERO).exists());
}

fn corrupt_first_entry_checksum(bytes: &mut [u8]) {
    let header = 24;
    let payload_len = u32::from_le_bytes(bytes[header..header + 4].try_into().unwrap()) as usize;
    let checksum_offset = header + 4 + payload_len;
    bytes[checksum_offset] ^= 0xff;
}

fn region_path(root: &str, region: IVec3) -> PathBuf {
    PathBuf::from(root).join(format!("region.{}.{}.{}.vwr", region.x, region.y, region.z))
}

fn tempfile_dir(name: &str) -> String {
    let mut path = std::env::temp_dir();
    path.push(name);
    let _ = std::fs::remove_dir_all(&path);
    std::fs::create_dir_all(&path).unwrap();
    path.to_string_lossy().to_string()
}
