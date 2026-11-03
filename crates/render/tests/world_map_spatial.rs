//! Integration tests for the indexed world-map rendering path.
//! These assert exact pixel equivalence with the existing ordinary-document path.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use vectorcraft_color::{Color, Paint};
use vectorcraft_doc::{Appearance, Document, Node, WorldMap, ZoomFade};
use vectorcraft_geom::{Affine, Rect, shapes};
use vectorcraft_render::{RenderOptions, Renderer};

fn grid(count: usize) -> Document {
    let mut doc = Document::new(1000.0, 500.0);
    doc.world_map = Some(WorldMap::standard());
    let layer = doc.layers[0].id;
    // Keep even the 8,192-object benchmark inside the geographic reference.
    let (columns, spacing, size) = if count > 1024 { (128, 7.0, 6.0) } else { (32, 28.0, 20.0) };
    for i in 0..count {
        let x = 7.0 + (i % columns) as f64 * spacing;
        let y = 7.0 + (i / columns) as f64 * spacing;
        let paint = if i % 2 == 0 { Color::rgb(1.0, 0.0, 0.0) } else { Color::rgb(0.0, 0.3, 1.0) };
        let appearance = Appearance::basic(Paint::solid(paint), Paint::None, 0.0);
        let n = Node::path(doc.alloc_id(), shapes::rectangle(Rect::new(x, y, x + size, y + size)), appearance);
        doc.insert(Some(layer), 0, n).unwrap();
    }
    doc
}

fn render(renderer: &mut Renderer, doc: &Document, dx: f64, dy: f64) -> Vec<u8> {
    let opts = RenderOptions { background: Some([255, 255, 255, 255]), ..Default::default() };
    renderer.render(doc, 128, 128, Affine::translate((-dx, -dy)), &opts).pixels
}

#[test]
fn indexed_map_matches_unindexed_renderer_at_multiple_viewports() {
    let mut indexed = grid(320);
    let mut ordinary = indexed.clone();
    ordinary.world_map = None;
    let mut fast = Renderer::new();
    let mut reference = Renderer::new();
    for (dx, dy) in [(0.0, 0.0), (220.0, 40.0), (600.0, 200.0), (850.0, 250.0)] {
        let actual = render(&mut fast, &indexed, dx, dy);
        let expected = render(&mut reference, &ordinary, dx, dy);
        assert_eq!(actual, expected, "different pixels at viewport {dx}, {dy}");
        assert!(fast.stats.spatial_nodes_skipped > 200, "spatial query did not prune offscreen nodes");
    }

    // Structural edits replace the parent's Arc; an old index must never hide new art.
    let layer = indexed.layers[0].id;
    let green = Appearance::basic(Paint::solid(Color::rgb(0.0, 1.0, 0.0)), Paint::None, 0.0);
    let inserted = Node::path(indexed.alloc_id(), shapes::rectangle(Rect::new(55.0, 55.0, 90.0, 90.0)), green);
    indexed.insert(Some(layer), 0, inserted).unwrap();
    ordinary = indexed.clone();
    ordinary.world_map = None;
    assert_eq!(render(&mut fast, &indexed, 0.0, 0.0), render(&mut reference, &ordinary, 0.0, 0.0));
    assert!(fast.stats.spatial_indices_built > 0, "edited layer index was not rebuilt");
}

#[test]
fn spatial_index_is_reused_between_pan_frames_without_changing_art() {
    let doc = grid(512);
    let mut renderer = Renderer::new();
    render(&mut renderer, &doc, 0.0, 0.0);
    assert!(renderer.stats.spatial_indices_built > 0);
    render(&mut renderer, &doc, 100.0, 0.0);
    assert_eq!(renderer.stats.spatial_indices_built, 0, "unchanged node rebuilt its index");
    assert!(renderer.stats.spatial_nodes_skipped > 0);
}

#[test]
fn indexed_world_map_still_respects_zoom_faded_layers() {
    let mut doc = grid(320);
    let layer = doc.layers[0].id;
    doc.node_mut(layer).unwrap().extras_mut().zoom_fade = Some(ZoomFade::new(None, Some([200.0, 500.0])).unwrap());
    let mut renderer = Renderer::new();
    let opts = |zoom| RenderOptions {
        background: Some([255, 255, 255, 255]),
        world_map_zoom: Some(zoom),
        ..Default::default()
    };
    let visible = renderer.render(&doc, 128, 128, Affine::IDENTITY, &opts(100.0));
    assert_eq!(visible.pixel(12, 12), [255, 0, 0, 255]);
    assert!(renderer.stats.spatial_indices_built > 0);
    let invisible = renderer.render(&doc, 128, 128, Affine::IDENTITY, &opts(500.0));
    assert_eq!(invisible.pixel(12, 12), [255, 255, 255, 255]);
    assert_eq!(renderer.stats.spatial_indices_built, 0, "fully faded layer should not be traversed");
}

#[test]
#[ignore = "manual performance measurement; timing depends on the Mac and build profile"]
fn benchmark_world_map_panning_against_reference() {
    use std::time::Instant;

    let doc = grid(8192);
    let mut ordinary = doc.clone();
    ordinary.world_map = None;
    let mut fast = Renderer::new();
    let mut reference = Renderer::new();
    let views = [(0.0, 0.0), (100.0, 50.0), (400.0, 120.0), (650.0, 200.0)];
    // Warm up both caches; compare the same camera positions.
    for (dx, dy) in views {
        let _ = render(&mut fast, &doc, dx, dy);
        let _ = render(&mut reference, &ordinary, dx, dy);
    }
    let start = Instant::now();
    for (dx, dy) in views {
        let _ = render(&mut fast, &doc, dx, dy);
    }
    let accelerated = start.elapsed();
    let start = Instant::now();
    for (dx, dy) in views {
        let _ = render(&mut reference, &ordinary, dx, dy);
    }
    let baseline = start.elapsed();
    eprintln!("world-map indexed: {accelerated:?}; legacy traversal: {baseline:?}; nodes skipped on last view: {}", fast.stats.spatial_nodes_skipped);
    // A benchmark is diagnostic, not a flaky timing-based pass/fail assertion.
}
