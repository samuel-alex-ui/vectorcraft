//! Versioned derived-preview manifests detect bad metadata before cache reads.
//! These files never replace the canonical editable VectorCraft document.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use vectorcraft_worldmap::{
    ChunkManifestError, MAX_MANIFEST_BYTES, MAX_PREVIEW_CHUNK_BYTES, PreviewChunkMeta, TILE_CACHE_SCHEMA, TileCacheManifest, WorldMap,
    WorldTile,
};

fn tile(column: u32, row: u32) -> WorldTile {
    WorldTile { level: 3, column, row }
}

#[test]
fn manifest_round_trips_and_verifies_a_preview_blob() {
    let map = WorldMap::standard();
    let mut manifest = TileCacheManifest::new(map, 42);
    let data = b"derived preview pixels, not canonical geography";
    let meta = manifest.add_preview(tile(5, 3), data).unwrap();
    assert_eq!(meta.bytes as usize, data.len());
    assert_eq!(meta.relative_path(42), format!("rev-42/z3/x5/y3-{}-{:08x}.bin", data.len(), meta.crc32));
    assert!(!meta.relative_path(42).contains(".."));
    let bytes = manifest.to_json().unwrap();
    let read = TileCacheManifest::from_json(&bytes, 42).unwrap();
    assert_eq!(read, manifest);
    assert_eq!(read.verify_tile(tile(5, 3), data), Ok(()));
    assert_eq!(read.verify_tile(tile(4, 3), data), Err(ChunkManifestError::InvalidTile));
    assert_eq!(read.verify_tile(tile(5, 3), b"wrong"), Err(ChunkManifestError::LengthMismatch));
    let mut corrupted = data.to_vec();
    corrupted[0] ^= 0x01;
    assert_eq!(read.verify_tile(tile(5, 3), &corrupted), Err(ChunkManifestError::ChecksumMismatch));
}

#[test]
fn invalid_dimensions_duplicate_tile_and_excessive_chunk_are_rejected() {
    let mut manifest = TileCacheManifest::new(WorldMap::standard(), 0);
    assert_eq!(manifest.add_preview(tile(5, 3), b""), Err(ChunkManifestError::InvalidChunkSize));
    assert_eq!(
        manifest.add_preview(WorldTile { level: 3, column: 16, row: 0 }, b"ok"),
        Err(ChunkManifestError::InvalidTile)
    );
    let valid = manifest.add_preview(tile(5, 3), b"ok").unwrap();
    assert_eq!(manifest.add_preview(tile(5, 3), b"again"), Err(ChunkManifestError::DuplicateTile));
    assert_eq!(manifest.entries.len(), 1);
    let overlong = vec![42u8; MAX_PREVIEW_CHUNK_BYTES + 1];
    assert_eq!(manifest.add_preview(tile(6, 3), &overlong), Err(ChunkManifestError::InvalidChunkSize));
    assert_eq!(valid.verify(&[]), Err(ChunkManifestError::LengthMismatch));
}

#[test]
fn stale_and_future_manifests_fail_closed() {
    let mut manifest = TileCacheManifest::new(WorldMap::standard(), 19);
    manifest.add_preview(tile(3, 2), b"tile").unwrap();
    let encoded = manifest.to_json().unwrap();
    assert_eq!(TileCacheManifest::from_json(&encoded, 20), Err(ChunkManifestError::StaleRevision));
    let mut json: serde_json::Value = serde_json::from_slice(&encoded).unwrap();
    json["schema"] = serde_json::json!(TILE_CACHE_SCHEMA + 1);
    let future = serde_json::to_vec(&json).unwrap();
    assert_eq!(TileCacheManifest::from_json(&future, 19), Err(ChunkManifestError::UnsupportedFormat));
    json["schema"] = serde_json::json!(TILE_CACHE_SCHEMA);
    json["magic"] = serde_json::json!("different cache");
    assert_eq!(
        TileCacheManifest::from_json(&serde_json::to_vec(&json).unwrap(), 19),
        Err(ChunkManifestError::UnsupportedFormat)
    );
}

#[test]
fn corrupt_json_and_duplicate_or_unsafe_entries_are_rejected() {
    let mut manifest = TileCacheManifest::new(WorldMap::standard(), 1);
    manifest.add_preview(tile(2, 4), b"ok").unwrap();
    let bytes = manifest.to_json().unwrap();
    let mut value: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    let entry = value["entries"][0].clone();
    value["entries"].as_array_mut().unwrap().push(entry);
    assert_eq!(
        TileCacheManifest::from_json(&serde_json::to_vec(&value).unwrap(), 1),
        Err(ChunkManifestError::DuplicateTile)
    );
    value["entries"].as_array_mut().unwrap().pop();
    value["entries"][0]["bytes"] = serde_json::json!(0);
    assert_eq!(
        TileCacheManifest::from_json(&serde_json::to_vec(&value).unwrap(), 1),
        Err(ChunkManifestError::InvalidChunkSize)
    );
    value["entries"][0]["bytes"] = serde_json::json!(3);
    value["entries"][0]["tile"]["column"] = serde_json::json!(99);
    assert_eq!(
        TileCacheManifest::from_json(&serde_json::to_vec(&value).unwrap(), 1),
        Err(ChunkManifestError::InvalidTile)
    );
    value["entries"][0]["tile"]["column"] = serde_json::json!(2);
    value["entries"][0]["tile"]["unknown"] = serde_json::json!("../escape");
    assert_eq!(
        TileCacheManifest::from_json(&serde_json::to_vec(&value).unwrap(), 1),
        Err(ChunkManifestError::InvalidEncoding)
    );
    assert_eq!(TileCacheManifest::from_json(b"{not json", 1), Err(ChunkManifestError::InvalidEncoding));
    assert_eq!(
        TileCacheManifest::from_json(&vec![b' '; MAX_MANIFEST_BYTES + 1], 1),
        Err(ChunkManifestError::TooManyEntries)
    );
}

#[test]
fn chunks_are_sorted_and_malicious_size_metadata_does_not_allocate() {
    let mut manifest = TileCacheManifest::new(WorldMap::standard(), u64::MAX);
    manifest.add_preview(tile(8, 6), b"data").unwrap();
    manifest.add_preview(tile(2, 6), b"data").unwrap();
    manifest.add_preview(tile(3, 2), b"data").unwrap();
    assert_eq!(manifest.entries.iter().map(|entry| entry.tile).collect::<Vec<_>>(), vec![tile(2, 6), tile(3, 2), tile(8, 6)]);
    let oversized = PreviewChunkMeta { tile: tile(3, 2), bytes: MAX_PREVIEW_CHUNK_BYTES as u32 + 1, crc32: 0 };
    assert_eq!(oversized.verify(b"foo"), Err(ChunkManifestError::InvalidChunkSize));
    assert!(manifest.entries.iter().all(|entry| entry.relative_path(u64::MAX).bytes().all(|c| c.is_ascii_alphanumeric() || b"/-._".contains(&c))));
}
