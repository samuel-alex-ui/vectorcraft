# Aetherys World Map — Phase 1

This is an incremental feature branch, not a replacement for VectorCraft.

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

## Next implementation steps

1. Add a user-visible **New World Map** preset/command, keeping regular
   VectorCraft documents unchanged. The new document should open with a
   1000 × 500 artboard and explicit world-map metadata.
2. Add editable map objects and layer conventions without altering existing
   general-purpose vector objects.
3. Preserve geographic mapping metadata when saving/reopening documents.
4. Add an explicit **3D Preview** action. First render an image of the 2D
   map as an equirectangular sphere texture; allow rotation and basic zoom.
5. Eventually replace the simple preview texture with resolution-aware
   vector data/tiles and deeper geographic navigation.

## Validation

From the repository root:

```bash
cargo test -p vectorcraft-worldmap
cargo fmt --all --check
```

These commands are provided for local validation; the initial changes
were committed via the GitHub connection and have **not** been compiled
or tested in this environment.

## Upstream compatibility

Work on `feature/world-map`. Do not modify upstream program behavior until
a focused integration is ready and verified. Keep the MIT / Apache-2.0
license notices on reused VectorCraft code.
