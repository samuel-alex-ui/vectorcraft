//! Stable read-only geographic tiles for future viewport streaming.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use vectorcraft_worldmap::{CanvasPoint, MAX_TILE_LEVEL, TileError, WorldMap, WorldTile};

fn point(x: f64, y: f64) -> CanvasPoint {
    CanvasPoint { x, y }
}

#[test]
fn level_zero_covers_the_planet_with_two_square_tiles() {
    let map = WorldMap::standard();
    let tiles = map.visible_tiles(point(0.0, 0.0), point(1000.0, 500.0), 0).unwrap();
    assert_eq!(tiles, vec![
        WorldTile { level: 0, column: 0, row: 0 },
        WorldTile { level: 0, column: 1, row: 0 },
    ]);
    assert_eq!(map.tile_bounds(tiles[0]), Some((point(0.0, 0.0), point(500.0, 500.0))));
    assert_eq!(map.tile_bounds(tiles[1]), Some((point(500.0, 0.0), point(1000.0, 500.0))));
}

#[test]
fn exact_tile_boundaries_include_neighbors_without_duplicates() {
    let map = WorldMap::standard();
    let tiles = map.visible_tiles(point(500.0, 250.0), point(500.0, 250.0), 2).unwrap();
    assert_eq!(tiles, vec![
        WorldTile { level: 2, column: 3, row: 1 },
        WorldTile { level: 2, column: 4, row: 1 },
        WorldTile { level: 2, column: 3, row: 2 },
        WorldTile { level: 2, column: 4, row: 2 },
    ]);
}

#[test]
fn off_world_views_are_clipped_without_implicit_antimeridian_wrap() {
    let map = WorldMap::standard();
    let west = map.visible_tiles(point(-100.0, 220.0), point(5.0, 225.0), 3).unwrap();
    let east = map.visible_tiles(point(995.0, 220.0), point(1100.0, 225.0), 3).unwrap();
    assert!(west.iter().all(|tile| tile.column == 0));
    assert!(east.iter().all(|tile| tile.column == 15));
    assert!(map.visible_tiles(point(-300.0, 20.0), point(-100.0, 60.0), 3).unwrap().is_empty());
    assert!(map.visible_tiles(point(1001.0, 20.0), point(1200.0, 60.0), 3).unwrap().is_empty());
}

#[test]
fn oversized_or_nonfinite_requests_are_rejected_before_allocation() {
    let map = WorldMap::standard();
    assert_eq!(map.visible_tiles(point(0.0, 0.0), point(1000.0, 500.0), 10), Err(TileError::TooManyTiles));
    assert_eq!(map.visible_tiles(point(0.0, 0.0), point(1000.0, 500.0), MAX_TILE_LEVEL + 1), Err(TileError::InvalidLevel));
    assert_eq!(map.visible_tiles(point(f64::NAN, 0.0), point(1.0, 1.0), 2), Err(TileError::InvalidViewport));
    assert_eq!(map.visible_tiles(point(3.0, 0.0), point(2.0, 5.0), 2), Err(TileError::InvalidViewport));
    assert_eq!(map.visible_tiles(point(0.0, f64::INFINITY), point(2.0, 5.0), 2), Err(TileError::InvalidViewport));
}

#[test]
fn building_scale_queries_preserve_geographic_tile_location() {
    let map = WorldMap::standard();
    let p = point(500.00000025, 250.00000025);
    let tiles = map.visible_tiles(p, p, MAX_TILE_LEVEL).unwrap();
    assert!(!tiles.is_empty());
    assert!(tiles.len() <= 4);
    for tile in tiles {
        let (lo, hi) = map.tile_bounds(tile).unwrap();
        assert!(lo.x <= p.x && p.x <= hi.x && lo.y <= p.y && p.y <= hi.y, "point outside tile {tile:?}");
    }
    assert!(map.tile_bounds(WorldTile { level: MAX_TILE_LEVEL, column: 1u32 << 19, row: 0 }).is_none());
    assert!(map.tile_bounds(WorldTile { level: MAX_TILE_LEVEL + 1, column: 0, row: 0 }).is_none());
}
