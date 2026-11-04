//! Versioned, strictly validated manifest for **derived** geographic preview chunks.
//!
//! The manifest is not a replacement for the canonical .vectorcraft Document:
//! its blobs are disposable rendered previews. A missing/invalid preview is a
//! cache miss, never evidence that the world has no geometry in that tile.
//!
//! All relative paths are derived from validated numeric tile identities.
//! Neither JSON nor a caller-provided blob can select arbitrary file paths.
//! CRC32 detects accidental corruption; it is not an authenticity signature.

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::{WorldMap, WorldTile};

pub const TILE_CACHE_SCHEMA: u32 = 1;
pub const MAX_MANIFEST_BYTES: usize = 4 << 20;
pub const MAX_MANIFEST_ENTRIES: usize = 16_384;
pub const MAX_PREVIEW_CHUNK_BYTES: usize = 16 << 20;
const MAGIC: &str = "aetherys-derived-tiles";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChunkManifestError {
    InvalidEncoding,
    UnsupportedFormat,
    StaleRevision,
    TooManyEntries,
    InvalidTile,
    DuplicateTile,
    InvalidChunkSize,
    LengthMismatch,
    ChecksumMismatch,
}

impl std::fmt::Display for ChunkManifestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for ChunkManifestError {}

/// Contents are immutable after publication; the file name is calculated,
/// not supplied by potentially malicious or stale manifest JSON.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PreviewChunkMeta {
    pub tile: WorldTile,
    pub bytes: u32,
    pub crc32: u32,
}

impl PreviewChunkMeta {
    pub fn verify(&self, data: &[u8]) -> Result<(), ChunkManifestError> {
        if self.bytes == 0 || self.bytes as usize > MAX_PREVIEW_CHUNK_BYTES {
            return Err(ChunkManifestError::InvalidChunkSize);
        }
        if data.len() != self.bytes as usize {
            return Err(ChunkManifestError::LengthMismatch);
        }
        if crc32fast::hash(data) != self.crc32 {
            return Err(ChunkManifestError::ChecksumMismatch);
        }
        Ok(())
    }

    /// A content-checked path within the application's *cache* directory.
    /// Revisions separate successive source documents; it cannot traverse.
    pub fn relative_path(&self, source_revision: u64) -> String {
        format!("rev-{source_revision}/z{}/x{}/y{}-{}-{:08x}.bin", self.tile.level, self.tile.column, self.tile.row, self.bytes, self.crc32)
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TileCacheManifest {
    magic: String,
    schema: u32,
    pub source_revision: u64,
    pub world: WorldMap,
    pub entries: Vec<PreviewChunkMeta>,
}

impl TileCacheManifest {
    pub fn new(world: WorldMap, source_revision: u64) -> Self {
        Self { magic: MAGIC.to_owned(), schema: TILE_CACHE_SCHEMA, source_revision, world, entries: Vec::new() }
    }

    /// Add metadata for one derived preview; it does not copy its contents.
    /// The caller must write the actual preview before publishing a manifest.
    pub fn add_preview(&mut self, tile: WorldTile, bytes: &[u8]) -> Result<PreviewChunkMeta, ChunkManifestError> {
        if self.world.tile_bounds(tile).is_none() {
            return Err(ChunkManifestError::InvalidTile);
        }
        if bytes.is_empty() || bytes.len() > MAX_PREVIEW_CHUNK_BYTES {
            return Err(ChunkManifestError::InvalidChunkSize);
        }
        if self.entries.len() >= MAX_MANIFEST_ENTRIES {
            return Err(ChunkManifestError::TooManyEntries);
        }
        if self.entries.iter().any(|entry| entry.tile == tile) {
            return Err(ChunkManifestError::DuplicateTile);
        }
        let entry = PreviewChunkMeta { tile, bytes: bytes.len() as u32, crc32: crc32fast::hash(bytes) };
        self.entries.push(entry.clone());
        self.entries.sort_unstable_by_key(|entry| entry.tile);
        Ok(entry)
    }

    pub fn get(&self, tile: WorldTile) -> Option<&PreviewChunkMeta> {
        self.entries.iter().find(|entry| entry.tile == tile)
    }

    /// Validate all manifest metadata before any cache file is opened.
    pub fn validate(&self, expected_revision: u64) -> Result<(), ChunkManifestError> {
        if self.magic != MAGIC || self.schema != TILE_CACHE_SCHEMA {
            return Err(ChunkManifestError::UnsupportedFormat);
        }
        if self.source_revision != expected_revision {
            return Err(ChunkManifestError::StaleRevision);
        }
        if self.entries.len() > MAX_MANIFEST_ENTRIES {
            return Err(ChunkManifestError::TooManyEntries);
        }
        let mut seen = BTreeSet::new();
        for entry in &self.entries {
            if self.world.tile_bounds(entry.tile).is_none() {
                return Err(ChunkManifestError::InvalidTile);
            }
            if entry.bytes == 0 || entry.bytes as usize > MAX_PREVIEW_CHUNK_BYTES {
                return Err(ChunkManifestError::InvalidChunkSize);
            }
            if !seen.insert(entry.tile) {
                return Err(ChunkManifestError::DuplicateTile);
            }
        }
        Ok(())
    }

    pub fn to_json(&self) -> Result<Vec<u8>, ChunkManifestError> {
        self.validate(self.source_revision)?;
        let json = serde_json::to_vec(self).map_err(|_| ChunkManifestError::InvalidEncoding)?;
        if json.len() > MAX_MANIFEST_BYTES {
            return Err(ChunkManifestError::TooManyEntries);
        }
        Ok(json)
    }

    /// Parse only a bounded JSON input. Unknown fields, unsupported schema,
    /// duplicate tiles, invalid coordinates and stale revisions fail closed.
    pub fn from_json(input: &[u8], expected_revision: u64) -> Result<Self, ChunkManifestError> {
        if input.len() > MAX_MANIFEST_BYTES {
            return Err(ChunkManifestError::TooManyEntries);
        }
        let manifest: Self = serde_json::from_slice(input).map_err(|_| ChunkManifestError::InvalidEncoding)?;
        manifest.validate(expected_revision)?;
        Ok(manifest)
    }

    pub fn verify_tile(&self, tile: WorldTile, blob: &[u8]) -> Result<(), ChunkManifestError> {
        let Some(meta) = self.get(tile) else { return Err(ChunkManifestError::InvalidTile) };
        meta.verify(blob)
    }
}
