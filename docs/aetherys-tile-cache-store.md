# Aetherys derived tile cache – experimental storage protocol (Phase 1.7)

This describes the **optional, standalone cache** implemented in
`vectorcraft-worldmap`, not the canonical `.vectorcraft` save format.
The editor does not use this disk store automatically yet.

## Data ownership and invariants

- The editable `Document.layers` tree remains the **only authoritative geometry**.
  All preview cache blobs can be deleted and recreated without losing a
  coastline, city, house or edit history.
- `TileCacheManifest` describes **derived preview bytes**, not vector nodes.
  Do not put a tile's editable Node tree exclusively in this cache.
- Each caller supplies an independent document cache directory and a revision
  label. Revisions identify that document's exact state; sharing a root across
  unrelated documents is not permitted. The editor has not been wired to
  generate these labels and roots.
- Missing/corrupt referenced preview bytes return typed errors. Callers must
  regenerate them from canonical geometry, not draw an empty tile.
- All file names are calculated from bounded numeric geographic tile keys.
  Manifest JSON cannot supply an arbitrary relative path.
- This cache is platform-gated: the filesystem API is absent on wasm32;
  the pure manifest and geographic indexing remain portable.

## Manifest schema v1

`TileCacheManifest::new(world, source_revision)` produces a manifest with:

- Fixed magic `aetherys-derived-tiles` and schema `1`
- The map projection/extent from the existing validated `WorldMap` metadata
- A document revision supplied by the application
- At most 16,384 unique entries, each carrying a validated `WorldTile`,
  positive payload size up to 16 MiB, and its CRC32 checksum

Manifest JSON is limited to **4 MiB** before deserialization.
Unknown JSON fields, invalid map metadata, future schemas, duplicates,
invalid tiles, stale revisions and invalid sizes are all rejected.
The relative path of a blob is derived as:

```text
rev-<revision>/z<level>/x<column>/y<row>-<length>-<crc32>.bin
```

CRC32 is for **accidental corruption detection only**, not adversarial
tamper resistance or cryptographic authenticity. If untrusted uploads are
later accepted, choose a cryptographic digest and a stricter trust model
before storing any canonical state.

## Two-phase publication and recovery behavior (native only)

`TileCacheStore::new(cache_root)` is a library component. The root must be
trusted application-controlled cache storage, independent of the real
`.vectorcraft` file and the user's document directory.

1. Compute derived preview payloads from the current canonical document,
   and describe them using `TileCacheManifest::add_preview`.
2. Call `write_preview(revision, meta, bytes)` for **every** entry. The API
   checks length and CRC32 before writing. Files are written to unique
   temporary names with `create_new`, synced, then renamed within the same
   directory. Existing immutable blobs are verified instead of overwritten.
3. Call `publish_manifest(&manifest)` only once all referenced blobs are
   present and verified. It writes a temporary JSON manifest, flushes it,
   then atomically replaces `manifest.json` in the cache directory.
4. During an open, `load_manifest(expected_revision)` validates the bounded
   manifest. `read_preview(manifest, tile)` checks the referenced bytes
   before allowing the caller to use them. A tile absent from the *manifest*
   returns `Ok(None)`; a file referenced by it that is missing/corrupt
   returns an error.

On an interrupted write **before the manifest rename**, the previous
manifest remains active, and incomplete new blobs are unreferenced. On
supported Unix filesystems the writer also fsyncs parent directories.
This mechanism protects **only derived cache publication**; it is not the
application's canonical save/undo/crash-recovery transaction. Filesystem
durability and directory operations vary by OS/volume.

Old revisions and failed/orphaned blobs may remain on disk until a future
garbage collector removes them. Do not interpret the cache's RAM budget as a
disk-space quota. The native APIs are not yet called by UI or renderer code.

## Remaining milestones before real vector streaming

1. Define a **separate**, versioned canonical geometry chunk format with
   stable node IDs, layer/effect/clip dependency handling, unbounded object
   and filled-polygon ownership rules, and an explicit node-to-chunk index.
2. Design an atomic transaction joining a document revision, its geometry
   manifest, loaded/dirty/pinned editable chunks, and undo journal. Do not
   evict unsaved or referenced geometry.
3. Make export, save, select, hit testing, copy/paste and 3D preview request
   complete geometry when required. Retain old-format compatibility.
4. Add asynchronous viewport-loading and cancellation, bounded decoded
   geometry memory and explicit error/loading states.
5. Run crash-injection, corruption, round-trip, undo/redo, macOS and wasm
   integration tests before enabling streaming in the editor.

## Validation commands

```sh
cargo fmt --all
cargo fmt --all --check
cargo test -p vectorcraft-worldmap
cargo test -p vectorcraft-worldmap --test chunk_manifest
cargo test -p vectorcraft-worldmap --test native_tile_store
cargo clippy -p vectorcraft-worldmap --all-targets -- -D warnings
cargo check -p vectorcraft-worldmap --target wasm32-unknown-unknown
```

The wasm target must be installed for the last command.
None of these tests establishes editor-level document streaming.
