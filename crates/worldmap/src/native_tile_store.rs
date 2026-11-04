//! Native filesystem store for *derived preview cache* blobs.
//!
//! This never opens or writes the editable .vectorcraft file. Blobs are
//! verified before the active manifest is atomically replaced. A crash may
//! leave an unreferenced temporary blob, but must not publish a manifest
//! pointing to an unfinished preview.
//!
//! Path segments originate only from validated numeric tile IDs; the root
//! directory itself must be selected by the trusted application, not by
//! untrusted document metadata.

use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use crate::{ChunkManifestError, MAX_MANIFEST_BYTES, PreviewChunkMeta, TileCacheManifest, WorldTile};

static TEMP_SEQUENCE: AtomicU64 = AtomicU64::new(0);
const ACTIVE_MANIFEST: &str = "manifest.json";

#[derive(Debug)]
pub enum TileStoreError {
    Io(std::io::Error),
    InvalidManifest(ChunkManifestError),
    MissingBlob,
    ExistingBlobMismatch,
}

impl std::fmt::Display for TileStoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(e) => write!(f, "tile cache IO: {e}"),
            Self::InvalidManifest(e) => write!(f, "invalid tile cache manifest or chunk: {e}"),
            Self::MissingBlob => write!(f, "referenced tile preview is missing"),
            Self::ExistingBlobMismatch => write!(f, "existing tile preview does not match its manifest"),
        }
    }
}

impl std::error::Error for TileStoreError {}

impl From<std::io::Error> for TileStoreError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<ChunkManifestError> for TileStoreError {
    fn from(error: ChunkManifestError) -> Self {
        Self::InvalidManifest(error)
    }
}

/// The root directory belongs to the application cache, never the document.
/// Disk garbage collection and automatic background loading are later work.
pub struct TileCacheStore {
    root: PathBuf,
}

impl TileCacheStore {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    /// Write one immutable derived preview at a safe, deterministic path.
    /// If that path already exists, verify it instead of overwriting it.
    pub fn write_preview(&self, source_revision: u64, meta: &PreviewChunkMeta, bytes: &[u8]) -> Result<(), TileStoreError> {
        meta.verify(bytes)?;
        let dest = self.root.join(meta.relative_path(source_revision));
        if dest.exists() {
            let saved = read_exact_preview(&dest, meta)?;
            if saved != bytes {
                return Err(TileStoreError::ExistingBlobMismatch);
            }
            return Ok(());
        }
        write_atomic(&dest, bytes)?;
        Ok(())
    }

    /// Load an optional preview only after verifying the complete manifest
    /// and the exact bytes of the referenced file. A referenced-but-missing
    /// blob is an error, not an empty tile.
    pub fn read_preview(&self, manifest: &TileCacheManifest, tile: WorldTile) -> Result<Option<Vec<u8>>, TileStoreError> {
        manifest.validate(manifest.source_revision)?;
        let Some(meta) = manifest.get(tile) else { return Ok(None) };
        let path = self.root.join(meta.relative_path(manifest.source_revision));
        Ok(Some(read_exact_preview(&path, meta)?))
    }

    /// Commit the active manifest only once every referenced preview is
    /// present and validated. The old manifest remains intact on any error
    /// before atomic rename. Documents themselves are never modified.
    pub fn publish_manifest(&self, manifest: &TileCacheManifest) -> Result<(), TileStoreError> {
        manifest.validate(manifest.source_revision)?;
        let json = manifest.to_json()?;
        for meta in &manifest.entries {
            let path = self.root.join(meta.relative_path(manifest.source_revision));
            read_exact_preview(&path, meta)?;
        }
        write_atomic(&self.root.join(ACTIVE_MANIFEST), &json)?;
        Ok(())
    }

    /// On a cache miss the caller must regenerate previews from the full
    /// canonical document. Corruption/stale revisions return a typed error;
    /// they are never interpreted as blank geography.
    pub fn load_manifest(&self, expected_revision: u64) -> Result<Option<TileCacheManifest>, TileStoreError> {
        let path = self.root.join(ACTIVE_MANIFEST);
        let mut file = match File::open(path) {
            Ok(file) => file,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(error) => return Err(TileStoreError::Io(error)),
        };
        // Read at most one byte over the limit, even for a maliciously large file.
        let mut json = Vec::new();
        Read::by_ref(&mut file).take((MAX_MANIFEST_BYTES + 1) as u64).read_to_end(&mut json)?;
        Ok(Some(TileCacheManifest::from_json(&json, expected_revision)?))
    }
}

fn read_exact_preview(path: &Path, meta: &PreviewChunkMeta) -> Result<Vec<u8>, TileStoreError> {
    let mut file = match File::open(path) {
        Ok(file) => file,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Err(TileStoreError::MissingBlob),
        Err(error) => return Err(TileStoreError::Io(error)),
    };
    // `bytes` has already been validated by its manifest or verify call.
    // Never allocate based on the actual on-disk length.
    let count = usize::try_from(meta.bytes).map_err(|_| ChunkManifestError::InvalidChunkSize)?;
    let mut out = vec![0u8; count];
    match file.read_exact(&mut out) {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::UnexpectedEof => {
            return Err(TileStoreError::InvalidManifest(ChunkManifestError::LengthMismatch));
        }
        Err(error) => return Err(TileStoreError::Io(error)),
    }
    let mut extra = [0u8; 1];
    if file.read(&mut extra)? != 0 {
        return Err(TileStoreError::InvalidManifest(ChunkManifestError::LengthMismatch));
    }
    meta.verify(&out)?;
    Ok(out)
}

fn write_atomic(dest: &Path, bytes: &[u8]) -> Result<(), TileStoreError> {
    let parent = dest.parent().ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidInput, "tile cache path has no parent"))?;
    fs::create_dir_all(parent)?;
    // create_new() avoids two writers ever sharing the same temporary file.
    let mut temp_file = None;
    for _ in 0..16 {
        let seq = TEMP_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let temp = parent.join(format!(".aetherys-{}-{seq}.tmp", std::process::id()));
        match OpenOptions::new().write(true).create_new(true).open(&temp) {
            Ok(file) => {
                temp_file = Some((temp, file));
                break;
            }
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(TileStoreError::Io(error)),
        }
    }
    let Some((temp, mut file)) = temp_file else {
        return Err(TileStoreError::Io(std::io::Error::new(std::io::ErrorKind::AlreadyExists, "no unique temporary cache path")));
    };
    let outcome = (|| -> Result<(), TileStoreError> {
        file.write_all(bytes)?;
        file.sync_all()?;
        drop(file);
        fs::rename(&temp, dest)?;
        // On Unix persist the directory entry after the atomic rename.
        #[cfg(unix)]
        File::open(parent)?.sync_all()?;
        Ok(())
    })();
    if outcome.is_err() {
        // Failure cleanup affects only our unique temporary cache file.
        let _ = fs::remove_file(temp);
    }
    outcome
}
