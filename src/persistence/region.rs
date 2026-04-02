use std::{
    collections::{BTreeMap, HashMap},
    fs,
    io::{Cursor, Read, Write},
    path::{Path, PathBuf},
};

use bevy::prelude::*;

use crate::{
    block::BlockId,
    chunk::{ChunkEditDelta, delta_from_overrides},
    config::{SaveMode, SavePolicy},
};

const MAGIC_V1: &[u8; 8] = b"VXREGN01";
const MAGIC_V2: &[u8; 8] = b"VXREGN02";
const HEADER_BYTES: usize = 24;

pub fn save_chunk_delta(
    policy: &SavePolicy,
    chunk: IVec3,
    seed: u64,
    generator_version: u32,
    chunk_version: u64,
    overrides: &BTreeMap<u32, BlockId>,
) -> Result<(), String> {
    if policy.mode != SaveMode::DeltaRegions {
        return Ok(());
    }

    fs::create_dir_all(&policy.root).map_err(|error| error.to_string())?;
    let path = region_path(
        Path::new(&policy.root),
        region_for_chunk(chunk, policy.region_dims),
    );
    let mut entries = read_region_entries(&path, seed, generator_version)?;
    if overrides.is_empty() {
        entries.remove(&chunk);
    } else {
        entries.insert(
            chunk,
            RegionEntry {
                chunk_version,
                edits: delta_from_overrides(overrides),
            },
        );
    }

    if entries.is_empty() {
        match fs::remove_file(&path) {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(error) => Err(error.to_string()),
        }
    } else {
        write_region_entries(&path, seed, generator_version, &entries)
    }
}

pub fn load_chunk_delta(
    policy: &SavePolicy,
    chunk: IVec3,
    seed: u64,
    generator_version: u32,
) -> Result<Option<Vec<ChunkEditDelta>>, String> {
    if policy.mode != SaveMode::DeltaRegions {
        return Ok(None);
    }

    let path = region_path(
        Path::new(&policy.root),
        region_for_chunk(chunk, policy.region_dims),
    );
    let entries = read_region_entries(&path, seed, generator_version)?;
    Ok(entries.get(&chunk).map(|entry| entry.edits.clone()))
}

#[derive(Clone, Debug)]
struct RegionEntry {
    chunk_version: u64,
    edits: Vec<ChunkEditDelta>,
}

fn region_for_chunk(chunk: IVec3, region_dims: IVec3) -> IVec3 {
    IVec3::new(
        chunk.x.div_euclid(region_dims.x.max(1)),
        chunk.y.div_euclid(region_dims.y.max(1)),
        chunk.z.div_euclid(region_dims.z.max(1)),
    )
}

fn region_path(root: &Path, region: IVec3) -> PathBuf {
    root.join(format!("region.{}.{}.{}.vwr", region.x, region.y, region.z))
}

fn read_region_entries(
    path: &Path,
    seed: u64,
    generator_version: u32,
) -> Result<HashMap<IVec3, RegionEntry>, String> {
    let bytes = match fs::read(path) {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(HashMap::new()),
        Err(error) => return Err(error.to_string()),
    };
    if bytes.len() < HEADER_BYTES {
        return Err("region file header is truncated".into());
    }

    let mut cursor = Cursor::new(bytes.as_slice());
    let magic = read_magic(&mut cursor)?;
    match magic {
        magic if magic == *MAGIC_V2 => read_region_entries_v2(&mut cursor, seed, generator_version),
        magic if magic == *MAGIC_V1 => read_region_entries_v1(&mut cursor, seed, generator_version),
        _ => Err("region magic mismatch".into()),
    }
}

fn read_region_entries_v1(
    cursor: &mut Cursor<&[u8]>,
    seed: u64,
    generator_version: u32,
) -> Result<HashMap<IVec3, RegionEntry>, String> {
    let file_seed = read_u64(cursor)?;
    let file_generator_version = read_u32(cursor)?;
    let entry_count = read_u32(cursor)?;
    if file_seed != seed || file_generator_version != generator_version {
        return Ok(HashMap::new());
    }

    let mut entries = HashMap::new();
    for _ in 0..entry_count {
        let chunk = IVec3::new(read_i32(cursor)?, read_i32(cursor)?, read_i32(cursor)?);
        let chunk_version = read_u64(cursor)?;
        let edit_count = read_u32(cursor)?;
        let mut edits = Vec::with_capacity(edit_count as usize);
        for _ in 0..edit_count {
            let local_index = read_u32(cursor)?;
            let block = BlockId(read_u16(cursor)?);
            edits.push(ChunkEditDelta { local_index, block });
        }
        entries.insert(
            chunk,
            RegionEntry {
                chunk_version,
                edits,
            },
        );
    }
    Ok(entries)
}

fn read_region_entries_v2(
    cursor: &mut Cursor<&[u8]>,
    seed: u64,
    generator_version: u32,
) -> Result<HashMap<IVec3, RegionEntry>, String> {
    let file_seed = read_u64(cursor)?;
    let file_generator_version = read_u32(cursor)?;
    let entry_count = read_u32(cursor)?;
    if file_seed != seed || file_generator_version != generator_version {
        return Ok(HashMap::new());
    }

    let mut entries = HashMap::new();
    for _ in 0..entry_count {
        match read_v2_entry(cursor)? {
            EntryRead::Parsed(chunk, entry) => {
                entries.insert(chunk, entry);
            }
            EntryRead::Skip => {}
            EntryRead::Truncated => break,
        }
    }
    Ok(entries)
}

enum EntryRead {
    Parsed(IVec3, RegionEntry),
    Skip,
    Truncated,
}

fn read_v2_entry(cursor: &mut Cursor<&[u8]>) -> Result<EntryRead, String> {
    let start = cursor.position() as usize;
    let bytes = cursor.get_ref();
    if start + 4 > bytes.len() {
        return Ok(EntryRead::Truncated);
    }

    let payload_len =
        u32::from_le_bytes(bytes[start..start + 4].try_into().expect("payload len")) as usize;
    let payload_start = start + 4;
    let checksum_start = payload_start + payload_len;
    let next_entry = checksum_start + 4;
    if next_entry > bytes.len() {
        cursor.set_position(bytes.len() as u64);
        return Ok(EntryRead::Truncated);
    }

    let payload = &bytes[payload_start..checksum_start];
    let expected_checksum = u32::from_le_bytes(
        bytes[checksum_start..next_entry]
            .try_into()
            .expect("checksum"),
    );
    cursor.set_position(next_entry as u64);

    if checksum(payload) != expected_checksum {
        return Ok(EntryRead::Skip);
    }

    match parse_v2_entry_payload(payload) {
        Ok(parsed) => Ok(EntryRead::Parsed(parsed.0, parsed.1)),
        Err(_) => Ok(EntryRead::Skip),
    }
}

fn parse_v2_entry_payload(payload: &[u8]) -> Result<(IVec3, RegionEntry), String> {
    let mut cursor = Cursor::new(payload);
    let chunk = IVec3::new(
        read_i32_slice(&mut cursor)?,
        read_i32_slice(&mut cursor)?,
        read_i32_slice(&mut cursor)?,
    );
    let chunk_version = read_u64_slice(&mut cursor)?;
    let edit_count = read_u32_slice(&mut cursor)?;
    let mut edits = Vec::with_capacity(edit_count as usize);
    for _ in 0..edit_count {
        let local_index = read_u32_slice(&mut cursor)?;
        let block = BlockId(read_u16_slice(&mut cursor)?);
        edits.push(ChunkEditDelta { local_index, block });
    }
    if cursor.position() as usize != payload.len() {
        return Err("entry payload length mismatch".into());
    }
    Ok((
        chunk,
        RegionEntry {
            chunk_version,
            edits,
        },
    ))
}

fn write_region_entries(
    path: &Path,
    seed: u64,
    generator_version: u32,
    entries: &HashMap<IVec3, RegionEntry>,
) -> Result<(), String> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(MAGIC_V2);
    bytes.extend_from_slice(&seed.to_le_bytes());
    bytes.extend_from_slice(&generator_version.to_le_bytes());
    bytes.extend_from_slice(&(entries.len() as u32).to_le_bytes());

    let mut sorted_entries: Vec<_> = entries.iter().collect();
    sorted_entries.sort_by_key(|(chunk, _)| (chunk.x, chunk.y, chunk.z));

    for (chunk, entry) in sorted_entries {
        let mut payload = Vec::new();
        payload.extend_from_slice(&chunk.x.to_le_bytes());
        payload.extend_from_slice(&chunk.y.to_le_bytes());
        payload.extend_from_slice(&chunk.z.to_le_bytes());
        payload.extend_from_slice(&entry.chunk_version.to_le_bytes());
        payload.extend_from_slice(&(entry.edits.len() as u32).to_le_bytes());
        for edit in &entry.edits {
            payload.extend_from_slice(&edit.local_index.to_le_bytes());
            payload.extend_from_slice(&edit.block.0.to_le_bytes());
        }

        bytes.extend_from_slice(&(payload.len() as u32).to_le_bytes());
        bytes.extend_from_slice(&payload);
        bytes.extend_from_slice(&checksum(&payload).to_le_bytes());
    }

    let mut file = fs::File::create(path).map_err(|error| error.to_string())?;
    file.write_all(&bytes).map_err(|error| error.to_string())
}

fn checksum(bytes: &[u8]) -> u32 {
    let mut hash = 0x811c_9dc5_u32;
    for byte in bytes {
        hash ^= u32::from(*byte);
        hash = hash.wrapping_mul(0x0100_0193);
    }
    hash
}

fn read_magic(cursor: &mut Cursor<&[u8]>) -> Result<[u8; 8], String> {
    let mut magic = [0_u8; 8];
    cursor
        .read_exact(&mut magic)
        .map_err(|error| error.to_string())?;
    Ok(magic)
}

fn read_u16(cursor: &mut Cursor<&[u8]>) -> Result<u16, String> {
    let mut bytes = [0_u8; 2];
    cursor
        .read_exact(&mut bytes)
        .map_err(|error| error.to_string())?;
    Ok(u16::from_le_bytes(bytes))
}

fn read_u32(cursor: &mut Cursor<&[u8]>) -> Result<u32, String> {
    let mut bytes = [0_u8; 4];
    cursor
        .read_exact(&mut bytes)
        .map_err(|error| error.to_string())?;
    Ok(u32::from_le_bytes(bytes))
}

fn read_i32(cursor: &mut Cursor<&[u8]>) -> Result<i32, String> {
    let mut bytes = [0_u8; 4];
    cursor
        .read_exact(&mut bytes)
        .map_err(|error| error.to_string())?;
    Ok(i32::from_le_bytes(bytes))
}

fn read_u64(cursor: &mut Cursor<&[u8]>) -> Result<u64, String> {
    let mut bytes = [0_u8; 8];
    cursor
        .read_exact(&mut bytes)
        .map_err(|error| error.to_string())?;
    Ok(u64::from_le_bytes(bytes))
}

fn read_u16_slice(cursor: &mut Cursor<&[u8]>) -> Result<u16, String> {
    read_u16(cursor)
}

fn read_u32_slice(cursor: &mut Cursor<&[u8]>) -> Result<u32, String> {
    read_u32(cursor)
}

fn read_i32_slice(cursor: &mut Cursor<&[u8]>) -> Result<i32, String> {
    read_i32(cursor)
}

fn read_u64_slice(cursor: &mut Cursor<&[u8]>) -> Result<u64, String> {
    read_u64(cursor)
}
