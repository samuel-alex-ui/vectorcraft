# Aetherys – dynamic metric scale bar (2D world map)

The 2D world-map editor displays a Google Maps-style scale bar near the
bottom-left of the map canvas. It is an **egui overlay**, not a vector node,
so editing, printing, export and serialization of the map geometry are
unchanged. Ordinary VectorCraft documents and presentation mode do not show it.

## Defining physical size

A fresh Aetherys world document deliberately has no planet radius:
the physical size of a fictional planet must never be assumed to be Earth's.

1. Open **File → Document Setup → World**.
2. Choose **Set radius**, then edit the planet's radius in **metres**.
   The initial 6,371,000 m shown after activation is an editable Earth-size
   example, **not** a default value stored in every world map.
3. Click OK. The radius is persisted in the existing `WorldMap` metadata
   and can be undone/redone in one normal Document Setup operation.
4. Until a radius is set, the canvas shows a short setup hint in place of
   any potentially misleading kilometre scale. **Clear** resets it to
   unknown. Ordinary document setup does not gain any world-only fields.

The `document.setup` engine command also reports/accepts `planetRadiusM`
for world maps, with `null` meaning unspecified. Invalid, nonfinite,
zero or negative radius values are rejected before mutating the document.

## Calculation and constraints

- The map is equirectangular, covering 360° longitude and 180° latitude,
  with canvas width twice its height. For a spherical radius `R` metres,
  a horizontal document-point delta at latitude `φ` corresponds locally
  to `(2πR / width) × cos(φ)` metres.
- The visible view's zoom is in **egui screen points per document point**
  (not device pixels). Screen-horizontal direction also accounts for any
  Rotate View angle `θ`:

  `metres_per_screen_point = (2πR / (width × zoom)) × sqrt((cosφ × cosθ)² + sin²θ)`

- The chosen reference latitude is at the centre of the current canvas.
  At very high latitudes the map projection strongly distorts distances.
  An ≈ prefix indicates that the result is a **local approximation**,
  not an exact long-distance geodesic or the surface area of a polygon.
- The bar chooses one of 1, 2, or 5 × 10ⁿ metres and adjusts its screen
  length continuously with zoom and pan. Units switch between km, m,
  cm and mm as appropriate. A pathological/undefined polar scale is
  hidden instead of displaying zero or infinity.
- A flat 2D map at 1,000,000× still uses the same radius and high-precision
  calculation. No canvas geometry is rescaled, quantized or duplicated.

This is an **on-screen linear scale** only. Measuring arbitrary paths,
territory areas in m²/km², altitude, relief and a physical 3D globe are
separate future tools/features.

## macOS validation

```sh
cargo fmt --all
cargo fmt --all --check
cargo test -p vectorcraft-worldmap --test scale
cargo test -p vectorcraft-worldmap
cargo test -p vectorcraft-engine --lib world_scale_tests
cargo test -p vectorcraft-ui-egui --lib world_tab_sets_physical_radius_as_one_undoable_document_edit
cargo test -p vectorcraft-ui-egui --lib metric_labels_are_human_readable_at_all_zoom_levels
cargo clippy -p vectorcraft-worldmap -p vectorcraft-engine -p vectorcraft-ui-egui --all-targets -- -D warnings
bash packaging/macos/aetherys-app.sh --build
open "dist/Aetherys World Editor.app"
```

After configuring a radius, pan the map from equator to high latitude,
rotate the view and zoom in to 1,000,000×. The bar should respond
immediately and never appear in an ordinary document or exported file.
