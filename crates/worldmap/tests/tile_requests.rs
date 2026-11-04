//! Viewport-first tile request planning is bounded, deterministic and read-only.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use vectorcraft_worldmap::{CanvasPoint, MAX_QUERY_TILES, TileError, WorldMap, WorldTile};

fn point(x: f64, y: f64) -> CanvasPoint {
    CanvasPoint { x, y }
}

#[test]
fn prefetch_never_replaces_visible_tiles_and_is_nearest_first() {
    let map = WorldMap::standard();
    let visible = map.visible_tiles(point(500.0, 250.0), point(500.0, 250.0), 2).unwrap();
    let plan = map.plan_tile_requests(point(500.0, 250.0), point(500.0, 250.0), 2, 1).unwrap();
    assert_eq!(plan.visible, visible);
    assert_eq!(plan.visible.len(), 4);
    assert_eq!(plan.prefetch.len(), 12);
    let expected_first_ring = [
        (3, 0), (4, 0),
        (2, 1), (5, 1),
        (2, 2), (5, 2),
        (3, 3), (4, 3),
    ];
    for (tile, (column, row)) in plan.prefetch.iter().zip(expected_first_ring) {
        assert_eq!((tile.column, tile.row), (column, row));
    }
    let mut unique = plan.visible.clone();
    unique.extend(&plan.prefetch);
    unique.sort_unstable();
    unique.dedup();
    assert_eq!(unique.len(), 16, "prefetch must not repeat visible or neighboring cells");
    assert_eq!(plan, map.plan_tile_requests(point(500.0, 250.0), point(500.0, 250.0), 2, 1).unwrap());
}

#[test]
fn planning_never_wraps_around_opposite_world_edges() {
    let map = WorldMap::standard();
    let west = map.plan_tile_requests(point(-50.0, 30.0), point(20.0, 60.0), 4, 2).unwrap();
    assert!(west.visible.iter().all(|tile| tile.column == 0));
    assert!(west.prefetch.iter().all(|tile| tile.column <= 2));
    let east = map.plan_tile_requests(point(990.0, 30.0), point(1060.0, 60.0), 4, 2).unwrap();
    assert!(east.visible.iter().all(|tile| tile.column == 31));
    assert!(east.prefetch.iter().all(|tile| tile.column >= 29));
}

#[test]
fn a_full_visible_budget_leaves_no_space_for_prefetch() {
    let map = WorldMap::standard();
    // At level 6, 64 columns by 64 rows are exactly the maximum tile budget.
    let top_left = point(0.0, 0.0);
    let bottom_right = point(1000.0 * 63.5 / 128.0, 500.0);
    let plan = map.plan_tile_requests(top_left, bottom_right, 6, 2).unwrap();
    assert_eq!(plan.visible.len(), MAX_QUERY_TILES);
    assert!(plan.prefetch.is_empty());
    assert_eq!(plan.visible.first(), Some(&WorldTile { level: 6, column: 0, row: 0 }));
    assert_eq!(plan.visible.last(), Some(&WorldTile { level: 6, column: 63, row: 63 }));
}

#[test]
fn speculative_requests_are_clipped_to_remaining_budget() {
    let map = WorldMap::standard();
    let plan = map.plan_tile_requests(point(0.0, 0.0), point(1000.0 * 62.5 / 128.0, 500.0), 6, 2).unwrap();
    assert_eq!(plan.visible.len(), 63 * 64);
    assert_eq!(plan.prefetch.len(), 64);
    assert!(plan.prefetch.iter().all(|tile| tile.column == 63));
    assert_eq!(plan.visible.len() + plan.prefetch.len(), MAX_QUERY_TILES);
}

#[test]
fn no_prefetch_and_invalid_inputs_have_explicit_results() {
    let map = WorldMap::standard();
    let without = map.plan_tile_requests(point(80.0, 80.0), point(100.0, 100.0), 3, 0).unwrap();
    assert_eq!(without.visible, map.visible_tiles(point(80.0, 80.0), point(100.0, 100.0), 3).unwrap());
    assert!(without.prefetch.is_empty());
    let outside = map.plan_tile_requests(point(1100.0, 100.0), point(1200.0, 200.0), 3, 2).unwrap();
    assert!(outside.visible.is_empty() && outside.prefetch.is_empty());
    assert_eq!(map.plan_tile_requests(point(20.0, 20.0), point(40.0, 40.0), 3, 3), Err(TileError::InvalidPrefetchMargin));
    assert_eq!(map.plan_tile_requests(point(f64::NAN, 0.0), point(10.0, 20.0), 3, 1), Err(TileError::InvalidViewport));
    assert_eq!(map.plan_tile_requests(point(0.0, 0.0), point(1000.0, 500.0), 10, 1), Err(TileError::TooManyTiles));
}
