# Aetherys World Map — Phase 1

This is an incremental feature branch, not a replacement for VectorCraft.

> Status: Phase 1.2 implemented, validation results below. Updated 2026-11-03.

## Purpose

Keep the existing 2D vector editor as the authoring surface. A future
read-only 3D globe preview will display the same world map. Editing on the
globe is **out of scope**.

## First implementation (committed)

`crates/worldmap` is an independent Rust workspace crate defining:

- an **equirectangular**, full-world 2:1 map (initial canvas 1000 × 500
  VectorCraft points);
- forward conversion from geographic longitude/latitude to canvas X/Y;
- inverse conversion from canvas X/Y to geographic longitude/latitude;
- validation of sizes, coordinates and non-finite values;
- unit tests for corners, the origin, round trips and antimeridian edges.

A 2:1 map ensures the world can later be projected onto a smooth sphere.
The editor's legacy drawing coordinates and artboards are deliberately not
changed yet.

## Geographic conventions

- Longitude: -180° west to +180° east
- Latitude: -90° south to +90° north
- Top-left canvas corner: (-180°, +90°)
- Bottom-right canvas corner: (+180°, -90°)
- X increases east; Y increases south
- Left and right map edges refer to the same meridian on a globe.

**Important:** This projection distorts area and distances near the poles.
The seam, pole editing, and geometry that crosses the seam require special
handling before shipping a world-map editing mode.

## Phase 1.2: persistent geographic document model

`Document::world_map: Option<WorldMap>` explicitly identifies a world-map
document. `None` is the default for ordinary and pre-existing documents and is
omitted on save. Neither title nor artboard proportions imply a world map.
The existing `file.newWorldMap` command is now registered in the engine,
creating one 1000 × 500 point artboard and a standard mapping. The menu, CLI
and MCP all use this same command; the former UI-only alias is removed.

`worldmap` remains independent of the document and UI at layer 0. `doc` at
layer 1 depends on it and re-exports `WorldMap`. Serialized metadata contains
`projection: "equirectangular"`, `width`, `height`, and optional
`planet_radius_m` (positive finite metres, no assumed planet size).
The projection defines full-world angular bounds [-180°, 180°] × [-90°, 90°].
Missing or unknown projections, malformed dimensions, invalid radii and unknown
metadata fields are rejected on deserialize rather than silently changing the
coordinate system. Future projections/schema extensions require reader support.

The geographic reference rectangle is fixed at document origin (0, 0), with its
saved width and height in document points. It is independent of artboard crops,
ruler origins and view zoom. Editing an artboard does not rescale the planet or
move artwork. Changing the geographic reference rectangle is not exposed yet.
Existing vector objects and their stable `NodeId`s remain the sole geometry
store: geographic coordinates are derived through `project`/`unproject`.
No second set of geographic objects or Chronica records is maintained.

### Persistence and alternatives

The existing native serializer stores this optional field losslessly, including
compressed documents and legacy save versions; no envelope version bump is
required. The pre-feature document model preserves unknown document keys through
`Document::extra`; readers with that mechanism can retain the new field.
Versioned round-trip tests exercise this reader with older envelopes, not
historical application binaries. No heuristics migrate earlier maps that only
have a title and 2:1 artboard: those still open as ordinary editable documents.

A title/size heuristic was rejected because ordinary artwork can have either.
Storing untyped data in `unknown` was rejected because it bypasses geographic
validation. Making the map depend on an editable artboard was rejected because
crop, resize and artboard deletion would implicitly alter world coordinates.
The geographic model therefore owns the reference extent, with the artboard
initially matching it. Duplicating geographic/vector geometry was rejected
because keeping both synchronized would create competing sources of truth.

## Next implementation steps

1. Add world-map object/layer conventions using existing vector nodes and IDs.
2. Define seam-crossing and pole-editing behaviour before adding geographic tools.
3. Add a separate read-only 3D preview in a later phase.
4. Later add level-of-detail rendering and optional Chronica references.

## Validation

From the repository root:

```bash
cargo test -p vectorcraft-worldmap
cargo fmt --all --check
```

Phase 1.2 additionally covers native compressed/versioned round trips, ordinary
documents without metadata, malformed geographic metadata, engine creation and
the existing headless menu command test.

Verified locally on macOS, 2026-11-03:

- `cargo test -p vectorcraft-worldmap`: 6 passed.
- `cargo test -p vectorcraft-worldmap -p vectorcraft-format --test world_map`:
  3 native format tests passed, including supported versions and compression.
- Workspace Clippy with `--all-targets --keep-going -- -D warnings`: passed
  during `cargo xtask ci`.

- `RUST_TEST_THREADS=2 cargo xtask ci`: all 12 gates passed, including
  workspace Clippy, 13,042 unit/integration tests (14 ignored), doctests,
  no-bundled-fonts builds/tests, PDF dependencies, assets, brands, cursors,
  localization report, layers and WASM. This includes the engine creation,
  journal replay, headless menu and geographic metadata property tests.
- The first unrestricted parallel test run exceeded the existing 20-second
  budget for offsetting 50,000 objects. The unchanged test passed alone in
  8.70 seconds and in the complete two-thread run. No tests were skipped or
  time budgets relaxed for this verification.

The native tests validate supported older file envelopes using the current
reader; historical application binaries and 3D preview were not exercised.

## Upstream compatibility

Work on a separate feature branch (Phase 1.2: `codex/aetherys-phase-1-2`). Do not modify upstream program behavior until
a focused integration is ready and verified. Keep the MIT / Apache-2.0
license notices on reused VectorCraft code.

## Revision history

| Date | Change |
|---|---|
| 2026-11-03 | Phase 1.2: explicit validated geographic metadata, engine creation command and native persistence tests. |
