//! Bounded, revision-aware cache for *derived* geographic tile previews.
//!
//! Tile previews may be raster bytes or other immutable presentation data
//! computed from the canonical Document; they are never source vector art.
//! The caller supplies the document revision: stale asynchronous results are
//! rejected, and changing revision invalidates the entire cache.
//!
//! The byte budget covers payloads held by this cache, not clones of Arc
//! buffers already handed to callers or allocator overhead. Never use the
//! cache's contents as the sole copy of an edit or saved document geometry.

use std::collections::{BTreeMap, HashSet};
use std::sync::Arc;

use crate::{MAX_QUERY_TILES, MAX_TILE_LEVEL, WorldTile};

/// Conservative default; separate from the renderer's spatial-index budget.
pub const DEFAULT_TILE_PREVIEW_BUDGET: usize = 32 << 20;
/// Hard ceiling prevents a caller from accidentally configuring unbounded RAM.
pub const MAX_TILE_PREVIEW_BUDGET: usize = 128 << 20;

struct Entry {
    data: Arc<[u8]>,
    last_used: u64,
}

/// Cache of immutable previews for one revision of the authoritative document.
///
/// Each instance is local to its caller: no shared global state, no disk IO,
/// and no changes to document serialization or undo history.
pub struct TilePreviewCache {
    revision: u64,
    budget: usize,
    bytes: usize,
    clock: u64,
    entries: BTreeMap<WorldTile, Entry>,
}

fn valid_tile(tile: WorldTile) -> bool {
    if tile.level > MAX_TILE_LEVEL {
        return false;
    }
    let cols = 1u32 << (tile.level + 1);
    let rows = 1u32 << tile.level;
    tile.column < cols && tile.row < rows
}

impl TilePreviewCache {
    /// `budget` is clamped to the 128 MiB safety ceiling; zero disables storage.
    pub fn new(revision: u64, budget: usize) -> Self {
        Self { revision, budget: budget.min(MAX_TILE_PREVIEW_BUDGET), bytes: 0, clock: 0, entries: BTreeMap::new() }
    }

    pub fn budget_bytes(&self) -> usize {
        self.budget
    }

    pub fn stored_bytes(&self) -> usize {
        self.bytes
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn revision(&self) -> u64 {
        self.revision
    }

    /// Only the owner may activate a new document revision. Clearing a cache
    /// drops *derived* previews; the canonical artwork is unaffected.
    pub fn switch_revision(&mut self, revision: u64) {
        if revision != self.revision {
            self.entries.clear();
            self.bytes = 0;
            self.clock = 0;
            self.revision = revision;
        }
    }

    /// Get an immutable shared preview. Old asynchronous consumers can hold a
    /// clone safely after eviction; it is never returned for a new revision.
    pub fn get(&mut self, revision: u64, tile: WorldTile) -> Option<Arc<[u8]>> {
        if revision != self.revision {
            return None;
        }
        let entry = self.entries.get_mut(&tile)?;
        self.clock = self.clock.saturating_add(1);
        entry.last_used = self.clock;
        Some(entry.data.clone())
    }

    /// Install a completed preview if it belongs to the active revision and
    /// can fit without evicting `visible` tiles. Prefetched results should
    /// pass the current viewport's tile keys here and may be discarded if
    /// visible previews already consume the budget.
    ///
    /// Returns false without changing existing entries if the new preview
    /// cannot fit even after evicting every unprotected entry. No geometry
    /// or editing state is ever pinned or stored by this cache.
    pub fn insert(&mut self, revision: u64, tile: WorldTile, bytes: Vec<u8>, visible: &[WorldTile]) -> bool {
        let size = bytes.len();
        if revision != self.revision || !valid_tile(tile) || size == 0 || size > self.budget {
            return false;
        }
        let old_size = self.entries.get(&tile).map_or(0, |entry| entry.data.len());
        let base = self.bytes.saturating_sub(old_size);
        let protected: HashSet<WorldTile> = visible.iter().copied().collect();
        let reclaimable = self.entries.iter().filter(|(k, _)| **k != tile && !protected.contains(k)).map(|(_, entry)| entry.data.len()).sum::<usize>();
        if base.saturating_add(size) > self.budget.saturating_add(reclaimable) {
            return false;
        }
        // Avoid keeping millions of tiny (but nonempty) previews.
        let replacing = self.entries.contains_key(&tile);
        while self.bytes.saturating_sub(old_size).saturating_add(size) > self.budget
            || (!replacing && self.entries.len() >= MAX_QUERY_TILES)
        {
            let victim = self.entries.iter()
                .filter(|(k, _)| **k != tile && !protected.contains(k))
                .min_by_key(|(key, entry)| (entry.last_used, **key))
                .map(|(key, _)| *key);
            let Some(victim) = victim else { return false };
            if let Some(evicted) = self.entries.remove(&victim) {
                self.bytes = self.bytes.saturating_sub(evicted.data.len());
            }
        }
        if let Some(previous) = self.entries.remove(&tile) {
            self.bytes = self.bytes.saturating_sub(previous.data.len());
        }
        self.clock = self.clock.saturating_add(1);
        self.entries.insert(tile, Entry { data: Arc::from(bytes), last_used: self.clock });
        self.bytes = self.bytes.saturating_add(size);
        true
    }
}
