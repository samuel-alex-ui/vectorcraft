//! Filesystem tests for optional, derived tile previews (never editable art).
#![cfg(not(target_arch = "wasm32"))]
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use vectorcraft_worldmap::{ChunkManifestError, MAX_MANIFEST_BYTES, TileCacheManifest, TileCacheStore, TileStoreError, WorldMap, WorldTile};

static UNIQUE: AtomicU64 = AtomicU64::new(0);

struct TestDirectory(PathBuf);

impl TestDirectory {
    fn new() -> Self {
        let nanos = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
        let sequence = UNIQUE.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!("aetherys-tile-cache-test-{}-{nanos}-{sequence}", std::process::id()));
        fs::create_dir(&path).unwrap();
        Self(path)
    }

    fn store(&self) -> TileCacheStore {
        TileCacheStore::new(self.0.clone())
    }
}

impl Drop for TestDirectory {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).ok();
    }
}

fn tile(col: u32) -> WorldTile {
    WorldTile { level: 3, column: col, row: 1 }
}

#[test]
fn cache_writes_and_reads_only_validated_derived_previews() {
    let directory = TestDirectory::new();
    let store = directory.store();
    assert!(store.load_manifest(12).unwrap().is_none());
    let mut manifest = TileCacheManifest::new(WorldMap::standard(), 12);
    let bytes_a = b"immutable coast preview A";
    let bytes_b = b"immutable coast preview B";
    let a = manifest.add_preview(tile(4), bytes_a).unwrap();
    let b = manifest.add_preview(tile(5), bytes_b).unwrap();
    store.write_preview(12, &a, bytes_a).unwrap();
    store.write_preview(12, &b, bytes_b).unwrap();
    // Existing immutable blobs are verified, not replaced.
    store.write_preview(12, &a, bytes_a).unwrap();
    store.publish_manifest(&manifest).unwrap();
    let loaded = store.load_manifest(12).unwrap().unwrap();
    assert_eq!(loaded, manifest);
    assert_eq!(store.read_preview(&loaded, tile(4)).unwrap(), Some(bytes_a.to_vec()));
    assert_eq!(store.read_preview(&loaded, tile(5)).unwrap(), Some(bytes_b.to_vec()));
    assert_eq!(store.read_preview(&loaded, tile(0)).unwrap(), None);
    assert!(matches!(store.load_manifest(13), Err(TileStoreError::InvalidManifest(ChunkManifestError::StaleRevision))));
}

#[test]
fn corrupted_and_missing_chunks_never_appear_as_blank_geography() {
    let directory = TestDirectory::new();
    let store = directory.store();
    let mut manifest = TileCacheManifest::new(WorldMap::standard(), 1);
    let original = b"valid derived preview";
    let meta = manifest.add_preview(tile(2), original).unwrap();
    assert!(matches!(store.write_preview(1, &meta, b"bogus"), Err(TileStoreError::InvalidManifest(ChunkManifestError::LengthMismatch))));
    store.write_preview(1, &meta, original).unwrap();
    store.publish_manifest(&manifest).unwrap();
    let path = directory.0.join(meta.relative_path(1));
    fs::write(&path, b"tiny").unwrap();
    assert!(matches!(store.read_preview(&manifest, tile(2)), Err(TileStoreError::InvalidManifest(ChunkManifestError::LengthMismatch))));
    fs::write(&path, vec![b'x'; original.len()]).unwrap();
    assert!(matches!(store.read_preview(&manifest, tile(2)), Err(TileStoreError::InvalidManifest(ChunkManifestError::ChecksumMismatch))));
    fs::remove_file(&path).unwrap();
    assert!(matches!(store.read_preview(&manifest, tile(2)), Err(TileStoreError::MissingBlob)));
}

#[test]
fn failed_publication_preserves_the_previously_committed_manifest() {
    let directory = TestDirectory::new();
    let store = directory.store();
    let mut old = TileCacheManifest::new(WorldMap::standard(), 7);
    let a = old.add_preview(tile(0), b"old preview").unwrap();
    store.write_preview(7, &a, b"old preview").unwrap();
    store.publish_manifest(&old).unwrap();

    let mut newer = TileCacheManifest::new(WorldMap::standard(), 8);
    let b = newer.add_preview(tile(1), b"new preview").unwrap();
    newer.add_preview(tile(2), b"unwritten preview").unwrap();
    store.write_preview(8, &b, b"new preview").unwrap();

    assert!(matches!(store.publish_manifest(&newer), Err(TileStoreError::MissingBlob)));
    // The new blob exists but is orphaned; the old committed manifest stays.
    assert_eq!(store.load_manifest(7).unwrap().unwrap(), old);
    assert_eq!(store.read_preview(&old, tile(0)).unwrap(), Some(b"old preview".to_vec()));
    assert!(matches!(store.load_manifest(8), Err(TileStoreError::InvalidManifest(ChunkManifestError::StaleRevision))));

    // After the last blob is written, the new complete manifest may replace it.
    let c = newer.get(tile(2)).unwrap();
    store.write_preview(8, c, b"unwritten preview").unwrap();
    store.publish_manifest(&newer).unwrap();
    assert_eq!(store.load_manifest(8).unwrap().unwrap(), newer);
}

#[test]
fn oversized_manifest_does_not_cause_unbounded_read_allocation() {
    let directory = TestDirectory::new();
    fs::write(directory.0.join("manifest.json"), vec![b' '; MAX_MANIFEST_BYTES + 1]).unwrap();
    let store = directory.store();
    assert!(matches!(store.load_manifest(1), Err(TileStoreError::InvalidManifest(ChunkManifestError::TooManyEntries))));
}
