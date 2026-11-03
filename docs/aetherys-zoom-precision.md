# Aetherys — Zoom and coordinate precision (Phase 1.3)

**Status:** Phase 1.4 enables world-map navigation to 1,000,000×. The Phase 1.3 audit below is historical baseline. Updated 2026-11-03.
Product goals remain those of [the project guide](aetherys-project-guide.md).

## Phase 1.3 baseline limits (world-map changes: Phase 1.4 below)

| Path | Current behaviour / limit |
|---|---|
| UI camera | `state::View` stores centre, rotation and zoom in `f64`. Zoom is screen points per document point. |
| Navigation | `canvas/view.rs`, marquee zoom, Set Zoom and saved views clamp to 0.0313–640×. Default percentage presets/Navigator span 3.13–64,000%. **Existing exception:** with Display Print Size on (96/72 baseline), Zoom In presets can reach 853⅓× (64,000 displayed percent); Set Zoom and restore still clamp at 640×. This inconsistency is tested and retained, not silently normalized. |
| Coordinate conversion | `canvas::Xf` subtracts the camera centre before projecting to the screen; rotation and local offsets use `f64`. egui pointer positions and final screen positions use `f32`. |
| Tool input | `Xf::to_doc` supplies document-space `f64` points. `ToolContext::tol` and engine selection tolerance divide screen tolerances by zoom. This alone does not verify every tool at extreme scales. |
| Model / persistence | Document points and world-map projection use `f64`; document coordinate limit is ±4,000,000 points (`doc/limits.rs`). Native save defaults to full floating-point precision; optional geometry rounding is lossy (`format::SaveOptions::precision`, up to 9 decimal places). |
| Rendering | `canvas::frame_view` builds an `f64` affine transform; `Renderer` derives the visible document rectangle through its inverse. CPU vector rasterization uses the existing vello_cpu backend; GPU presentation does not imply a new GPU vector engine. |
| Culling | `renderer/nodes.rs` skips objects outside cached visible bounds and leaves smaller than a quarter screen pixel in both dimensions. This is existing culling, not a new geographic LOD/index/streaming system. |
| Raster size | Renderer contexts cap each side at 32,768 pixels (`render/options.rs`); UI texture/device limits can be smaller. This is an image-size limit, not a geographic world-size limit. |

For the guide's illustrative **40,000 km circumference** and 1000-point world
width, one document point represents 40 km at the equator. With a 1000-screen-
point viewport and zoom 640×, about **62.5 km** is visible horizontally; one
screen point spans **62.5 m**, and a 10 m building is only **0.16 screen point**.
The Print Size preset exception reaches about 46.875 km across the same viewport,
still far from building-scale editing. The current UI therefore does **not**
provide building-scale editing.
Zoom factors here are absolute screen/document scale, not factors relative to
Fit Artboard (which also depends on viewport size and margins). Device pixels
add the display scale factor. Physical resolution varies with latitude and
planet radius; equirectangular coordinates do not preserve local distances.

## Small implementation change

`Xf::to_screen` now adds the screen centre to the rotated local offset in `f64`
and rounds once at the final egui boundary. Previously the offset was rounded
before addition, losing detail when it almost cancelled the screen centre.
`Xf::to_doc` now promotes both screen coordinates before subtraction, rather
than performing the subtraction in `f32`. Regression tests reproduce both
losses on the previous code.

No zoom cap, tool, renderer, native format or geographic reference is changed.
Normal and world-map documents use this same conversion. Camera navigation
must not rewrite vector geometry or world metadata.

## Targeted verification

`crates/ui-egui/tests/zoom_precision.rs` checks:

- both premature-rounding regressions;
- screen → document → screen at zoom 0.0313, 1, 640, 40,000, 400,000 and
  1,000,000, four rotations, map centre and near ±4,000,000 document points:
  less than 0.002 screen point round-trip error and less than 0.01 screen
  point difference against the combined render affine;
- a one-screen-point delta keeps its length after conversion and scaling;
- existing UI zoom limits, including the Print Size preset exception, and
  unchanged document state (with a small vector rectangle) for ordinary/world maps;
- the existing CPU renderer drawing an isolated 20-screen-pixel rectangle at
  640–1,000,000× into a 64×64 image, with foreground/background pixel checks.

`crates/worldmap/tests/precision.rs` checks geographic round trips near both
map edges and poles, and distinct canvas points separated by 0.00000025 point.
It requires less than 0.000001 screen point projection error at a synthetic
1,000,000× scale (less than 1e-12 document point).

These are bounded arithmetic and simple-vector checks. They do **not** enable
1,000,000× in the UI or certify dense maps, effects, text, snapping, Bézier
editing or every renderer operation at that scale. Execution results are
recorded below.

Executed on macOS, 2026-11-03:

- Before the fix, `cargo test -p vectorcraft-ui-egui --test zoom_precision`
  reproduced both rounding regressions (2 failed, 3 passed).
- After the fix, the same target passed all 5 tests, including the Print Size
  navigation exception and unchanged ordinary/world-map documents with artwork.
- `cargo test -p vectorcraft-worldmap`: 6 existing unit tests and 2 new precision
  integration tests passed.
- `cargo check -p vectorcraft-ui-egui --lib --target wasm32-unknown-unknown`:
  passed (an existing renderer dead-code warning remains).
- `cargo fmt -p vectorcraft-ui-egui -p vectorcraft-worldmap -- --check`: passed.
- `cargo clippy -p vectorcraft-ui-egui --lib --test zoom_precision -- -D warnings`:
  passed.
- `cargo clippy -p vectorcraft-worldmap --all-targets -- -D warnings`: passed.

Full workspace CI was not repeated: this phase changes two arithmetic
expressions in the existing UI mapping and adds bounded tests/documentation;
focused regression checks and the WASM build address the changed behaviour.
No performance benchmark, full tool-matrix validation or interactive
building-scale zoom is claimed.

## Measures still required for the product goal

- Raise limits only after auditing every input path, zoom field/stop, saved-view
  restore and tool tolerance together, with normal-document compatibility.
- Keep camera-relative subtraction before narrowing to screen/raster types.
  Evaluate a local editing origin if necessary, keeping one authoritative
  vector geometry store and the unchanged geographic reference. Combined
  affine matrices can lose low bits when very large translations cancel.
- Audit absolute geometry tolerances (for example `geom/hit.rs` uses 1e-9),
  path cleanup, imports/exports and optional save rounding. `f64` alone is
  not an end-to-end accuracy guarantee; its spacing grows with coordinate size.
- Verify drag/anchor edits, selection, snapping, curves, strokes, gradients,
  images, text and effects under zoom, rotation, HiDPI and repeated navigation.
- Benchmark large scenes and visible-object queries before selecting spatial
  indices and later LOD/chunking. Existing culling does not establish performance
  for millions of objects or avoid all scene traversal.
- Define seam-crossing and pole-editing rules separately. Point round trips do
  not solve topology, pole ambiguity or geographic distortion.

No tiles, new LOD system, map layers or globe are introduced in this phase.

## Revision history

| Date | Change |
|---|---|
| 2026-11-03 | Phase 1.3: source audit, final-boundary rounding, targeted large-factor tests and explicit remaining limits. |

## Phase 1.4 — world-map navigation (2026-11-03)

World-map views now derive a 1,000,000× ceiling from `Document::world_map`.
The per-view policy is refreshed from document metadata, not inferred from title
or canvas size. Ordinary documents retain their old limits and Print Size
exception. Wheel/trackpad and animated zoom use `zoom_about`; click Zoom Tool,
marquee zoom, menu/keyboard steps, direct percentage entry, Navigator's slider
and saved-view restoration use the world-map ceiling. Above the old presets,
world-map steps double/halve the factor. Percent fields use the existing Print
Size baseline: 1,000,000× means 100,000,000% with the default baseline.

Fit Artboard/All/Selection still use the existing fit policy (including its
640× ceiling and one-document-point minimum target extent). They are not the
route to house-scale zoom in this core implementation. Use direct zoom, wheel,
Zoom Tool or Navigator instead. There is no new renderer or LOD system. Dense
maps, all effects, pen/direct-anchor editing, HiDPI and real gesture-device QA
still require broader validation; simple vector drawing/selection/movement
checks do not certify every tool at every scale.

### Local macOS application

Package the **existing** release binary without compiling:

```bash
bash packaging/macos/aetherys-app.sh
```

For future source updates, build once and repackage:

```bash
bash packaging/macos/aetherys-app.sh --build
open "dist/Aetherys World Editor.app"
```

The Finder app executes the copied native binary directly: no Cargo invocation,
Rust installation or compilation at launch. The script reuses the upstream
Info.plist, document associations, icon and entitlements. It ad-hoc signs and
verifies the local bundle; redistribution to other Macs still needs the
upstream signed/notarized release process. It does not rename all VectorCraft
UI strings or create an independent preference/document format. Packaging an
old release binary does not update its code: use `--build` for the new zoom.

### Relocated repository

The stale contributor paths came from `env!("CARGO_MANIFEST_DIR")` embedded in
the cached UI build-script executable. Read Cargo's current runtime manifest
directory instead; contributor inputs remain relative to the crate. No user
home path is embedded in source and no global `cargo clean` is necessary.


Phase 1.4 checks performed: seven `zoom_precision` integration tests passed,
including rectangle pointer drawing, selection, movement and Undo with a
1,000,000× tool context; full UI-target Clippy with `-D warnings` passed;
UI library WASM check passed (existing renderer warning). Normal-document
limits, Print Size behaviour, high-factor projection and simple rendering
remain covered. No full workspace CI or dense-scene benchmark was run.
The package-only path was Plist-linted, ad-hoc signature verified, launched via
LaunchServices (`open -n`) and visually inspected through a control screenshot.
`bash packaging/macos/aetherys-app.sh --build` completed successfully (one
release build). The updated app was launched through LaunchServices; control
inspection confirmed `world_map: true` and `zoom: 1000000.0`, and a screenshot
confirmed a small filled rectangle at that scale. Plist and signature checks
passed; linked libraries were macOS system libraries/frameworks.

Observed interaction limits: a default 1 pt stroke becomes enormous at this
scale, obscuring a small object's fill. Use None or an appropriately small
stroke before drawing houses; defaults are not silently rescaled. Properties
still rounds tiny point dimensions to `0 pt` visually even though f64 geometry
is retained. Detailed numeric-field presentation and complete anchor/pen-tool
coverage remain follow-up work. Basic pointer drawing/selection/movement and
Undo passed, with saved-view restoration also exercised through `view.goto1`.
