//! Geographic tile addressing for future lazy viewport caches.
//!
//! A tile is only an index over the *same* equirectangular document coordinates.
//! It does not own artwork and must not become a second geometry store. At level 0
//! the planet has two square tiles across its 2:1 map; each level doubles both axes.

use crate::{CanvasPoint, WorldMap};

/// Highest level supported by the stable u32 tile coordinates (2^19 × 2^18).
pub const MAX_TILE_LEVEL: u8 = 18;
/// Limit an individual viewport query so an accidentally huge zoomed-out
/// request cannot allocate millions of tile keys.
pub const MAX_QUERY_TILES: usize = 4096;
/// Maximum prefetch distance in whole geographic tiles. Keeps planning bounded.
pub const MAX_PREFETCH_MARGIN: u8 = 2;

/// Tiles required immediately, followed by optional nearby speculative work.
///
/// This is a read-only request plan, not a geometry store, asynchronous loader
/// or mutation of the canonical vector document. Consumers must always load
/// `visible` first and may cancel or ignore `prefetch` during a fast pan.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TileRequestPlan {
    pub visible: Vec<WorldTile>,
    /// Sorted nearest-first (Manhattan distance from the visible tile rectangle)
    /// with row/column tiebreakers for reproducible load order.
    pub prefetch: Vec<WorldTile>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct WorldTile {
    pub level: u8,
    pub column: u32,
    pub row: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TileError {
    InvalidLevel,
    InvalidViewport,
    InvalidPrefetchMargin,
    TooManyTiles,
}

impl std::fmt::Display for TileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for TileError {}

/// The inclusive tile range intersecting a visible interval. Both sides of an
/// exact tile boundary are included to be conservative for edge strokes.
fn tile_range(a: f64, b: f64, extent: f64, count: u32) -> (u32, u32) {
    let start = a.clamp(0.0, extent) / extent * f64::from(count);
    let end = b.clamp(0.0, extent) / extent * f64::from(count);
    let last = count - 1;
    let start_floor = start.floor() as u32;
    let first = if start > 0.0 && start.fract() == 0.0 { start_floor.saturating_sub(1) } else { start_floor };
    (first.min(last), (end.floor() as u32).min(last))
}

impl WorldMap {
    /// Plan which tiles a viewport needs now and which nearby tiles could
    /// be prefetched without delaying it. The total number of requested keys
    /// never exceeds `MAX_QUERY_TILES`. Viewport tiles are never dropped to
    /// make room for speculative prefetch work.
    ///
    /// This deliberately does not wrap across the ±180° map edges: the
    /// 2D editor needs an explicit seam policy before scheduling both sides.
    pub fn plan_tile_requests(
        &self,
        top_left: CanvasPoint,
        bottom_right: CanvasPoint,
        level: u8,
        prefetch_margin: u8,
    ) -> Result<TileRequestPlan, TileError> {
        if prefetch_margin > MAX_PREFETCH_MARGIN {
            return Err(TileError::InvalidPrefetchMargin);
        }
        let visible = self.visible_tiles(top_left, bottom_right, level)?;
        let Some(first) = visible.first().copied() else {
            return Ok(TileRequestPlan { visible, prefetch: Vec::new() });
        };
        if prefetch_margin == 0 || visible.len() >= MAX_QUERY_TILES {
            return Ok(TileRequestPlan { visible, prefetch: Vec::new() });
        }

        let mut min_column = first.column;
        let mut max_column = first.column;
        let mut min_row = first.row;
        let mut max_row = first.row;
        for tile in &visible {
            min_column = min_column.min(tile.column);
            max_column = max_column.max(tile.column);
            min_row = min_row.min(tile.row);
            max_row = max_row.max(tile.row);
        }

        let columns = 1u32 << (level + 1);
        let rows = 1u32 << level;
        let margin = u32::from(prefetch_margin);
        let min_x = min_column.saturating_sub(margin);
        let max_x = max_column.saturating_add(margin).min(columns - 1);
        let min_y = min_row.saturating_sub(margin);
        let max_y = max_row.saturating_add(margin).min(rows - 1);

        // A rectangular tile query covers every cell between these bounds:
        // no hash set or extra geometry ownership is needed to exclude them.
        let mut neighbors = Vec::new();
        for row in min_y..=max_y {
            for column in min_x..=max_x {
                if (min_column..=max_column).contains(&column) && (min_row..=max_row).contains(&row) {
                    continue;
                }
                let dx = min_column.saturating_sub(column).max(column.saturating_sub(max_column));
                let dy = min_row.saturating_sub(row).max(row.saturating_sub(max_row));
                neighbors.push((dx + dy, WorldTile { level, column, row }));
            }
        }
        neighbors.sort_unstable_by_key(|(distance, tile)| (*distance, tile.row, tile.column));
        let remaining = MAX_QUERY_TILES - visible.len();
        let prefetch = neighbors.into_iter().take(remaining).map(|(_, tile)| tile).collect();
        Ok(TileRequestPlan { visible, prefetch })
    }

    /// Returns the map-space bounding box for a validated tile key. A tile
    /// exists independently of zoom, display size, platform and renderer.
    pub fn tile_bounds(&self, tile: WorldTile) -> Option<(CanvasPoint, CanvasPoint)> {
        if tile.level > MAX_TILE_LEVEL {
            return None;
        }
        let columns = 1u32 << (tile.level + 1);
        let rows = 1u32 << tile.level;
        if tile.column >= columns || tile.row >= rows {
            return None;
        }
        let (width, height) = self.size();
        Some((
            CanvasPoint { x: width * f64::from(tile.column) / f64::from(columns), y: height * f64::from(tile.row) / f64::from(rows) },
            CanvasPoint { x: width * f64::from(tile.column + 1) / f64::from(columns), y: height * f64::from(tile.row + 1) / f64::from(rows) },
        ))
    }

    /// Query tiles intersecting the rectangular viewport, including boundary
    /// contacts and clipped off-world viewports. Results are row-major and
    /// contain no duplicates.
    ///
    /// Deliberately does *not* wrap a view across the antimeridian: both ends
    /// are separate 2D map edges, and world-wrap presentation needs an explicit
    /// view policy. The query is read-only and never loads or modifies geometry.
    pub fn visible_tiles(&self, top_left: CanvasPoint, bottom_right: CanvasPoint, level: u8) -> Result<Vec<WorldTile>, TileError> {
        if level > MAX_TILE_LEVEL {
            return Err(TileError::InvalidLevel);
        }
        if ![top_left.x, top_left.y, bottom_right.x, bottom_right.y].iter().all(|value| value.is_finite())
            || top_left.x > bottom_right.x
            || top_left.y > bottom_right.y
        {
            return Err(TileError::InvalidViewport);
        }
        let (width, height) = self.size();
        if top_left.x > width || top_left.y > height || bottom_right.x < 0.0 || bottom_right.y < 0.0 {
            return Ok(Vec::new());
        }
        let columns = 1u32 << (level + 1);
        let rows = 1u32 << level;
        let (x0, x1) = tile_range(top_left.x, bottom_right.x, width, columns);
        let (y0, y1) = tile_range(top_left.y, bottom_right.y, height, rows);
        // u64 avoids overflow on wasm32 for deliberately huge viewport requests.
        let count = u64::from(x1 - x0 + 1) * u64::from(y1 - y0 + 1);
        if count > MAX_QUERY_TILES as u64 {
            return Err(TileError::TooManyTiles);
        }
        let mut out = Vec::with_capacity(count as usize);
        for row in y0..=y1 {
            for column in x0..=x1 {
                out.push(WorldTile { level, column, row });
            }
        }
        Ok(out)
    }
}
