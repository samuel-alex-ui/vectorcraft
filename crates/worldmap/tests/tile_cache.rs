//! Derived tile previews are disposable, revision-scoped and RAM bounded.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use vectorcraft_worldmap::{MAX_TILE_PREVIEW_BUDGET, TilePreviewCache, WorldTile};

fn tile(column: u32) -> WorldTile {
    WorldTile { level: 2, column, row: 0 }
}

#[test]
fn least_recently_used_unprotected_preview_is_evicted() {
    let mut cache = TilePreviewCache::new(42, 8);
    assert!(cache.insert(42, tile(0), vec![1; 3], &[]));
    assert!(cache.insert(42, tile(1), vec![2; 3], &[]));
    assert_eq!(cache.get(42, tile(0)).unwrap().as_ref(), &[1, 1, 1]);
    assert!(cache.insert(42, tile(2), vec![3; 3], &[]));
    assert_eq!(cache.stored_bytes(), 6);
    assert_eq!(cache.len(), 2);
    assert!(cache.get(42, tile(1)).is_none(), "least recent preview should be evicted");
    assert_eq!(cache.get(42, tile(0)).unwrap().as_ref(), &[1, 1, 1]);
    assert_eq!(cache.get(42, tile(2)).unwrap().as_ref(), &[3, 3, 3]);
}

#[test]
fn visible_previews_are_preserved_and_optional_prefetch_can_be_rejected() {
    let mut cache = TilePreviewCache::new(1, 6);
    assert!(cache.insert(1, tile(0), vec![10; 3], &[]));
    assert!(cache.insert(1, tile(1), vec![11; 3], &[]));
    assert!(!cache.insert(1, tile(2), vec![12; 3], &[tile(0), tile(1)]));
    assert_eq!(cache.len(), 2);
    assert_eq!(cache.stored_bytes(), 6);
    assert!(cache.get(1, tile(0)).is_some());
    assert!(cache.get(1, tile(1)).is_some());

    assert!(cache.insert(1, tile(2), vec![12; 3], &[tile(0)]));
    assert!(cache.get(1, tile(0)).is_some());
    assert!(cache.get(1, tile(1)).is_none());
    assert!(cache.get(1, tile(2)).is_some());
}

#[test]
fn rejected_replacement_does_not_evict_or_modify_existing_entries() {
    let mut cache = TilePreviewCache::new(7, 6);
    assert!(cache.insert(7, tile(0), vec![1; 2], &[]));
    assert!(cache.insert(7, tile(1), vec![2; 3], &[]));
    assert!(!cache.insert(7, tile(0), vec![9; 5], &[tile(1)]));
    assert_eq!(cache.stored_bytes(), 5);
    assert_eq!(cache.get(7, tile(0)).unwrap().as_ref(), &[1, 1]);
    assert_eq!(cache.get(7, tile(1)).unwrap().as_ref(), &[2, 2, 2]);
}

#[test]
fn stale_async_results_are_rejected_after_document_revision_change() {
    let mut cache = TilePreviewCache::new(10, 20);
    assert!(cache.insert(10, tile(0), vec![3; 4], &[]));
    let old_handle = cache.get(10, tile(0)).unwrap();
    cache.switch_revision(11);
    assert_eq!(cache.revision(), 11);
    assert_eq!(cache.stored_bytes(), 0);
    assert!(cache.is_empty());
    assert!(cache.get(11, tile(0)).is_none());
    assert!(cache.get(10, tile(0)).is_none());
    assert!(!cache.insert(10, tile(0), vec![4; 4], &[]));
    assert!(cache.insert(11, tile(0), vec![5; 4], &[]));
    assert_eq!(old_handle.as_ref(), &[3, 3, 3, 3], "eviction never modifies an Arc held by a caller");
    assert_eq!(cache.get(11, tile(0)).unwrap().as_ref(), &[5, 5, 5, 5]);
    cache.switch_revision(11);
    assert_eq!(cache.len(), 1, "same revision should not discard a warm cache");
}

#[test]
fn invalid_tiles_empty_payloads_and_excessive_budgets_are_safe() {
    let mut cache = TilePreviewCache::new(0, usize::MAX);
    assert_eq!(cache.budget_bytes(), MAX_TILE_PREVIEW_BUDGET);
    assert!(!cache.insert(0, WorldTile { level: 2, column: 8, row: 0 }, vec![1], &[]));
    assert!(!cache.insert(0, WorldTile { level: 19, column: 0, row: 0 }, vec![1], &[]));
    assert!(!cache.insert(0, tile(0), Vec::new(), &[]));
    assert!(cache.is_empty());
    let mut disabled = TilePreviewCache::new(0, 0);
    assert!(!disabled.insert(0, tile(0), vec![1], &[]));
    assert_eq!(disabled.stored_bytes(), 0);
}
