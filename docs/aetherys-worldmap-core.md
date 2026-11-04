# Aetherys World Map Core – phased implementation (1.6–1.8)

This work builds on Phase 1.5's independent, zoom-faded geographic layers. Keep all
regular VectorCraft document behavior unchanged. **The three phases are one overall
project, not an excuse to deploy unfinished disk streaming or filled-polygon wrapping.**

## Step 1 implemented on this branch: spatial culling (Phase 1.6 foundation)

### Existing engine behavior

The renderer already calculates per-object painted bounds, culls offscreen
objects and caches geometry by immutable `Arc<Node>` identity. However, even
when each individual object was cheaply culled, `draw_children` visited
every child of a large layer on every pan frame.

### New behavior

For world-map layers and simple groups containing **128 or more direct members**:

1. Compute a conservative bounding-box tree once using the existing renderer's
   authoritative painted bounds. Retain the tree in the parent's geometry-cache
   entry (not in the document or native-file format).
2. Each frame queries the visible document rectangle with the same two-pixel
   safety margin as ordinary culling.
3. Visit candidates in **original painter order** and then run the exact
   existing `draw_arc` pipeline with its normal checks. Nodes with missing,
   invalid or unbounded rectangles always participate.
4. On structural edits, normal immutable-node replacement gives the layer a
   new identity, which cannot reuse its old index. Existing symbol/plugin
   invalidation logic drops the associated geometry-cache entry.
5. Knockout, clipping, isolated selection/dimming, trace previews and unusual
   transient drawing paths retain the prior rendering behavior.
6. For ordinary VectorCraft documents the index is disabled.

Index construction costs O(N log N) and memory O(N) per indexed container;
subsequent viewport queries are generally substantially cheaper than visiting
all N children. **Derived spatial indices share a 64 MiB retained-cache
budget:** the least recently used index is discarded as needed both during
rendering and at frame end. Its geometry and document contents are never
discarded. A single indexed group is limited to 262,144 direct children;
oversized groups use the original conservative renderer. These are cache
limits, not caps on the number of objects a document can hold.

A spatially giant individual path *still needs its full geometry drawn*;
this step does not subdivide a single path or automatically split filled
coastlines.

Renderer statistics include `spatial_indices_built` (index/cache misses)
and `spatial_nodes_skipped` (members pruned by the spatial query).

### Test and benchmark plan

Run on macOS from the repository branch:

```sh
cargo fmt --all
cargo fmt --all --check
cargo test -p vectorcraft-render --test world_map_spatial
cargo test -p vectorcraft-render --lib spatial_budget
cargo test -p vectorcraft-worldmap --test seam_polyline
cargo test -p vectorcraft-worldmap --test tiles
cargo test -p vectorcraft-worldmap --test tile_requests
cargo test -p vectorcraft-render --test world_map_fade
cargo test -p vectorcraft-worldmap
cargo clippy -p vectorcraft-render -p vectorcraft-worldmap --all-targets -- -D warnings
# Optional diagnostic benchmarks (not performance pass/fail assertions):
cargo test -p vectorcraft-render --test world_map_spatial -- --ignored --nocapture
# The new 32,000-vertex *single filled coastline* diagnostic isolates a
# path-level bottleneck: the group index is not expected to make this faster.
cargo test -p vectorcraft-render --test world_map_spatial benchmark_one_huge_filled_coastline -- --ignored --nocapture
```

Pixel-for-pixel comparisons with ordinary rendering on four viewports must
pass. After the first frame, repeated pan frames should reuse the index;
after inserting a new object, the index must rebuild and display that object.
Time a warmed renderer on multiple viewports with representative workloads;
compare wall times on the *same machine* and report the number of objects
pruned. The first reported local Mac benchmark on 2026-11-04, with 8,192 simple
rectangles and four warmed viewports, measured 1.004459 ms indexed versus
1.443375 ms on the legacy traversal (approximately 30.4% lower elapsed time)
and 7,664 skipped nodes on the last view. This is **one diagnostic run**,
not a reliable general-purpose speedup claim. Larger coastlines, fills,
effects, memory and other zoom levels still need profiling.

### Explicit next 1.6 work

- A manual diagnostic benchmark with **one 32,000-vertex filled coastline**
  has now been added. It checks pixel-equivalence against an ordinary
  document and reports the timed renders without asserting an unearned
  speedup. Run on macOS and record measurements before changing path
  segmentation or the rendering of filled shapes.
- Profile one single extremely long coastline path and filled coast polygons.
  A group-level index does **not** reduce their segment processing.
- Decide on topology-preserving tiled geometry or path/subpath acceleration
  without changing fills, stroke caps, winding rules, effects or editing.
- The 64 MiB retained-cache limit and oversized-group fallback are now
  implemented and have targeted tests. Validate on macOS.
- Rotated and 1,000,000× viewport comparisons and heavily overlapping
  painter-order comparisons were added; run them before release. Still profile
  nested groups, varied blend modes and large real-world maps.

## Phase 1.7: tile addressing and memory safety – first foundations

**Implemented in this step:** the 64 MiB world-map spatial-index cache
budget and `WorldMap::visible_tiles` / `WorldMap::tile_bounds` provide a
stable 2:1 geographic tile grid. Level 0 contains two 500×500 tiles on
the standard 1000×500 map; each level doubles each axis, up to level 18.
Viewport queries conservatively include tiles touching exact boundaries,
clip off-world views, reject invalid coordinates and cap responses at
4,096 tile keys. Tile addresses **do not store or copy vector geometry**.
The open-line antimeridian is not silently wrapped by the viewport query.

**Not implemented:** disk loading, tile manifests, dirty/pinned editing
chunks, prefetching, cache-backed document materialization and crash recovery.
The viewport tiles are currently a read-only addressing utility, not a
streaming engine.

**Next foundation implemented:** `WorldMap::plan_tile_requests` supplies a
`TileRequestPlan` containing strictly required `visible` tiles and optional
nearest-first `prefetch` tiles. A configurable margin of 0–2 neighboring
tiles never wraps across the map seam. It preserves every visible tile and
caps the combined request list at 4,096. Queries reject invalid margins and
nonfinite or oversized viewports. A later asynchronous loader can prioritize
`visible`, cancel obsolete prefetches, and reuse the returned stable
`WorldTile` keys. Neither method creates or persists map geometry.
These are deterministic, pure-data plans; **no asynchronous I/O or disk
chunk loading has been added**.

**Do not lazy-load the existing mutable `Document.layers` graph without
designing document consistency and undo first.** Current documents hold nodes
directly, and edits, serialization, export, clipboard and history assume those
nodes are available.

Proposed safe approach:

1. Define a versioned optional chunk manifest keyed by geographic tile and
   detail band, with stable node IDs, explicit bounds and per-blob checksums.
   Chunk files must have an atomic commit protocol and be independently
   validated; hashes must be checked before exposing decoded geometry.
2. Define a read-only cache of decoded chunks with an explicit byte budget and
   an LRU eviction policy. Dirty or actively edited chunks stay pinned until
   durably saved. Failures render an error state, never an empty coastline.
3. Keep the canonical saved document and its revision/journal authoritative.
   An interrupted save or undo must not lose unloaded geometry. Clipboard,
   export, search and selection need complete-materialization policies.
4. Prefetch neighboring visible tiles; cancel stale asynchronous requests.
   Avoid blocking the render thread or writing caches to the document format.
5. Preserve compatibility with old `.vcr` documents. Introduce streaming only
   behind an opt-in world-map mode, after round-trip, undo, crash-recovery and
   memory tests.

The existing geometry/frame caches and newly indexed group traversal are a
safe foundation; they are **not** an implementation of disk-backed streaming.

## Phase 1.8: geographic seam – first utility, not complete editor integration

`WorldMap::project_seamless_polyline` now splits **open geographic polylines**
on the antimeridian at x=0/x=width, interpolating latitude at the seam.
Longitude changes greater than 180° take the shorter path. Input points are
validated through the existing projection API. Exact +180° to −180° pairs do
not create a spurious line across the flat map. This is a derived display
operation: it does not mutate the original geographic source points.

**Not yet supported:** holes, closed/filled polygons, Bezier control-point
wrapping, clipping masks, interactive seam snapping, multiworld projection,
duplicated labels, export wrapping and 3D mapping. Filling a split polygon
naively changes its topology and must not be done. Build polygon clipping and
editor integration separately, with tests for winding and border ownership.

## Merge and validation gates

- This branch is deliberately stacked onto Phase 1.5's branch until PR #5
  is merged into main; do not land it directly on an older main or duplicate
  Phase 1.5 in an unrelated patch.
- A GitHub commit alone does not prove a native build. Run the tests and
  Clippy above and a local application smoke test before any merge.
- The 3D globe can follow once streaming, editor seam semantics and large-map
  performance are proven, not merely because the module names exist.
