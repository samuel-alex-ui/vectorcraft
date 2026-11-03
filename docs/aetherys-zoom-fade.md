# Aetherys 1.5 – World-map layer zoom fading

**Implementation branch:** `feature/aetherys-zoom-fade`  
**Scope:** Existing VectorCraft layers; no new map geometry, rendering engine or layer hierarchy.

## User workflow

1. Create a world map with **File → New World Map…**.
2. Create two ordinary layers, e.g. **Eldoria – Grob** and **Eldoria – Fein**. Draw an independent coastline/land outline on each.
3. Select the coarse layer in the Layers panel and open **Layer Options** (the Layers panel menu → *Options for Selection…*, or double-click a row away from the name).
4. Under **Aetherys – Zoom visibility**, enable *Fade layer by zoom* and **Fade out**, e.g. from **200×** to **500×**.
5. Select the fine layer, enable **Fade in** from **200×** to **500×**.
6. Zoom: at 200× the coarse layer is fully visible; through the transition both blend; at 500× only the fine layer remains. Fading does **not** move, simplify or regenerate either path.

Use additional layers for more transitions, e.g. a medium coast fades in 200–500× and fades out 2,000–5,000×. The values are **camera factors** (screen points per document point), not the UI's percent labels. A document may contain arbitrarily many independent detail layers.

## Design and technical integration

- `vectorcraft_doc::ZoomFade` is a validated, optional value with independent `fade_in: [start,end]?` and `fade_out: [start,end]?` intervals. Intervals must be finite, positive and strictly increasing.
- The option is stored in each layer's `NodeExtras::zoom_fade` and serialized with the normal document node. It follows the layer when moved or copied, without a parallel layer/object registry.
- `layer.setProps`, `layer.new` and `layer.newSublayer` accept optional `zoomFade` only in world maps. `null` removes it. Invalid ranges and non-layer targets are rejected.
- A smoothstep curve gives continuous opacity: `t*t*(3-2*t)`. Fade-in opacity increases from 0 to 1; fade-out decreases from 1 to 0. If both are configured, their multipliers are combined.
- The canvas supplies `RenderOptions::world_map_zoom`; the existing renderer multiplies the **whole layer group's** effective opacity. Saved object/layer opacity and geometry do not change. Fully transparent layers are skipped.
- Selection, Select Behind and marquee selection skip fully faded layers at the current zoom. The Layers panel remains a direct way to locate/edit an otherwise invisible layer.
- Ordinary VectorCraft documents, and rendering without a world-map zoom option, keep existing appearance and selection behaviour.

## Boundaries and known limitations

- **No automatic geometric synchronization.** When a large-scale coastline changes, adjust the broad representation first and refine the finer one manually.
- **Cross-fade is not path morphing.** At intermediate zoom levels two different silhouettes may overlap visibly. Particularly with filled land masses, transparent edges may show ghosting/colour changes. For exact filled-land transitions, a future single-background/coverage strategy may be needed.
- **View-dependent only for now.** Existing export/thumbnail code does not set `world_map_zoom`, so exports render the document without zoom-based fading, rather than selecting the current detail level automatically.
- **Selection tool scope:** Standard Selection/Direct/Group Selection paths were updated. Other tools that independently sample underlying geometry may still see layers at zero fade; those paths need separate validation.
- A layer intentionally faded to zero can still be edited after choosing it in the Layers panel; it will not appear on canvas until its zoom or fade range changes. A future edit-mode override may improve this.
- Performance and correctness in very large maps, every live effect, special blend/knockout group, 3D projection, and seam handling remain separate development tasks.
- Changes have been committed via GitHub. Native compilation, formatting, Clippy and runtime behaviour must be **verified on a Rust build environment** before merging.

## Useful commands

```sh
cargo test -p vectorcraft-doc geographic_fade_tests
cargo test -p vectorcraft-doc zoom_fade
cargo test -p vectorcraft-format --test world_map_fade
cargo test -p vectorcraft-render --test world_map_fade
cargo test -p vectorcraft-engine --test world_map_fade
cargo test -p vectorcraft-ui-egui world_map_layer_dialog
cargo fmt --all --check
```

From a locally checked-out feature branch, an optional application build:

```sh
bash packaging/macos/aetherys-app.sh --build
```
