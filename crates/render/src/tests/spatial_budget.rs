//! Spatial-index memory limits must never discard document contents or rendered art.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use super::*;
use vectorcraft_doc::WorldMap;

fn indexed_map(offset: f64) -> Document {
    let mut doc = Document::new(1000.0, 500.0);
    doc.world_map = Some(WorldMap::standard());
    let layer = doc.layers[0].id;
    for i in 0..256 {
        let x = offset + (i % 32) as f64 * 25.0;
        let y = 10.0 + (i / 32) as f64 * 25.0;
        let shape = shapes::rectangle(Rect::new(x, y, x + 15.0, y + 15.0));
        let fill = Paint::solid(Color::rgb(0.15, 0.7, 0.25));
        let node = Node::path(doc.alloc_id(), shape, Appearance::basic(fill, Paint::None, 0.0));
        doc.insert(Some(layer), 0, node).unwrap();
    }
    doc
}

fn render_map(renderer: &mut Renderer, doc: &Document) -> Vec<u8> {
    let opts = RenderOptions { background: Some([255, 255, 255, 255]), ..Default::default() };
    renderer.render(doc, 128, 128, Affine::IDENTITY, &opts).pixels
}

fn cached_index_bytes(renderer: &Renderer) -> usize {
    renderer.geom.values().filter_map(|entry| entry.spatial.as_ref()).map(|index| index.memory_bytes()).sum()
}

#[test]
fn evicted_index_rebuilds_without_pixel_changes_or_document_mutation() {
    let doc = indexed_map(0.0);
    let before = doc.clone();
    let mut renderer = Renderer::new();
    let pixels = render_map(&mut renderer, &doc);
    assert_eq!(renderer.stats.spatial_indices_built, 1);
    assert!(cached_index_bytes(&renderer) > 0);
    renderer.trim_spatial_cache_to(0);
    assert_eq!(cached_index_bytes(&renderer), 0);
    let again = render_map(&mut renderer, &doc);
    assert_eq!(pixels, again, "evicting a derived index changed output");
    assert_eq!(renderer.stats.spatial_indices_built, 1);
    assert_eq!(doc, before, "index eviction must never modify saved document data");
}

#[test]
fn older_indexes_are_evicted_before_a_recently_used_map() {
    let old = indexed_map(0.0);
    let recent = indexed_map(40.0);
    let old_key = Arc::as_ptr(&old.layers[0]) as usize;
    let recent_key = Arc::as_ptr(&recent.layers[0]) as usize;
    let mut renderer = Renderer::new();
    render_map(&mut renderer, &old);
    let old_size = cached_index_bytes(&renderer);
    assert!(old_size > 0);
    render_map(&mut renderer, &recent);
    let new_size = cached_index_bytes(&renderer) - old_size;
    assert!(new_size > 0);
    renderer.trim_spatial_cache_to(new_size);
    assert!(renderer.geom.get(&old_key).and_then(|entry| entry.spatial.as_ref()).is_none());
    assert!(renderer.geom.get(&recent_key).and_then(|entry| entry.spatial.as_ref()).is_some());
    // The old map still draws correctly after its index is recreated.
    let mut baseline = Renderer::new();
    assert_eq!(render_map(&mut renderer, &old), render_map(&mut baseline, &old));
    assert_eq!(renderer.stats.spatial_indices_built, 1);
}
