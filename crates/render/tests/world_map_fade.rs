//! Zoom-dependent geographic layer opacity is a view-only composition.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use vectorcraft_color::{Color, Paint};
use vectorcraft_doc::{Appearance, Document, Node, WorldMap, ZoomFade};
use vectorcraft_geom::{Affine, Rect, shapes};
use vectorcraft_render::{RenderOptions, Renderer};

fn picture(doc: &Document, zoom: Option<f64>) -> [u8; 4] {
    let opts = RenderOptions { background: Some([255, 255, 255, 255]), world_map_zoom: zoom, ..Default::default() };
    Renderer::new().render(doc, 100, 100, Affine::IDENTITY, &opts).pixel(50, 50)
}

#[test]
fn world_map_layer_fades_without_mutating_stored_opacity() {
    let mut doc = Document::new(1000.0, 500.0);
    doc.world_map = Some(WorldMap::standard());
    let layer = doc.layers[0].id;
    doc.node_mut(layer).unwrap().extras_mut().zoom_fade = Some(ZoomFade::new(None, Some([200.0, 500.0])).unwrap());
    let path = Node::path(
        doc.alloc_id(),
        shapes::rectangle(Rect::new(20.0, 20.0, 80.0, 80.0)),
        Appearance::basic(Paint::solid(Color::rgb(1.0, 0.0, 0.0)), Paint::None, 0.0),
    );
    doc.insert(Some(layer), 0, path).unwrap();

    assert_eq!(picture(&doc, Some(100.0)), [255, 0, 0, 255]);
    assert_eq!(picture(&doc, Some(500.0)), [255, 255, 255, 255]);
    let halfway = picture(&doc, Some(350.0));
    assert_eq!(halfway[0], 255);
    assert!(halfway[1] > 80 && halfway[1] < 200, "{halfway:?}");
    assert_eq!(picture(&doc, None), [255, 0, 0, 255]); // Export or thumbnail.
    assert_eq!(doc.node(layer).unwrap().opacity, 1.0);
    doc.world_map = None;
    assert_eq!(picture(&doc, Some(350.0)), [255, 0, 0, 255]); // Ordinary document.
}

#[test]
fn nested_group_overrides_a_fully_faded_parent_and_preserves_manual_visibility() {
    let mut doc = Document::new(1000.0, 500.0);
    doc.world_map = Some(WorldMap::standard());
    let layer = doc.layers[0].id;
    doc.node_mut(layer).unwrap().extras_mut().zoom_fade = Some(ZoomFade::new(None, Some([200.0, 500.0])).unwrap());
    let group = doc.alloc_id();
    let path = Node::path(
        doc.alloc_id(),
        shapes::rectangle(Rect::new(20.0, 20.0, 80.0, 80.0)),
        Appearance::basic(Paint::solid(Color::rgb(1.0, 0.0, 0.0)), Paint::None, 0.0),
    );
    doc.insert(Some(layer), 0, Node::group(group, vec![std::sync::Arc::new(path)])).unwrap();
    let inherited = picture(&doc, Some(350.0));
    assert_eq!(picture(&doc, Some(500.0)), [255, 255, 255, 255]);
    doc.node_mut(group).unwrap().extras_mut().zoom_fade = Some(ZoomFade::new(Some([200.0, 500.0]), None).unwrap());
    assert_eq!(picture(&doc, Some(350.0)), inherited);
    assert_eq!(picture(&doc, Some(500.0)), [255, 0, 0, 255]);
    doc.node_mut(layer).unwrap().visible = false;
    assert_eq!(picture(&doc, Some(500.0)), [255, 255, 255, 255]);
}

#[test]
fn complementary_detail_levels_overlap_in_layer_order() {
    let mut doc = Document::new(1000.0, 500.0);
    doc.world_map = Some(WorldMap::standard());
    let base = doc.layers[0].id;
    for (color, fade_in, fade_out) in
        [(Color::rgb(1.0, 0.0, 0.0), None, Some([200.0, 500.0])), (Color::rgb(0.0, 0.0, 1.0), Some([200.0, 500.0]), None)]
    {
        let group = doc.alloc_id();
        let mut node = Node::group(group, vec![]);
        node.extras_mut().zoom_fade = Some(ZoomFade::new(fade_in, fade_out).unwrap());
        doc.insert(Some(base), usize::MAX, node).unwrap();
        let path = Node::path(
            doc.alloc_id(),
            shapes::rectangle(Rect::new(20.0, 20.0, 80.0, 80.0)),
            Appearance::basic(Paint::solid(color), Paint::None, 0.0),
        );
        doc.insert(Some(group), 0, path).unwrap();
    }
    assert_eq!(picture(&doc, Some(200.0)), [255, 0, 0, 255]);
    let [r, g, b, a] = picture(&doc, Some(350.0));
    assert!(r.abs_diff(128) <= 2 && g.abs_diff(64) <= 2 && b.abs_diff(192) <= 2 && a == 255, "{r}, {g}, {b}, {a}");
    assert_eq!(picture(&doc, Some(500.0)), [0, 0, 255, 255]);
}

#[test]
fn paired_coast_fills_preserve_coverage_and_use_one_outline() {
    use vectorcraft_doc::GeographicMetadata;

    let mut doc = Document::new(100.0, 100.0);
    doc.world_map = Some(WorldMap::standard());
    for (left, color, fade) in [
        (20.0, Color::rgb(1.0, 0.0, 0.0), ZoomFade::new(None, Some([2.0, 4.0])).unwrap()),
        (30.0, Color::rgb(0.0, 0.0, 1.0), ZoomFade::new(Some([2.0, 4.0]), None).unwrap()),
    ] {
        let mut level = Node::layer(doc.alloc_id(), "Coast", vectorcraft_doc::LayerColor::Preset(0));
        level.extras_mut().detail_level = true;
        level.extras_mut().zoom_fade = Some(fade);
        let id = level.id;
        doc.insert(None, usize::MAX, level).unwrap();
        let mut coast = Node::path(
            doc.alloc_id(),
            shapes::rectangle(Rect::new(left, 20.0, 80.0, 80.0)),
            Appearance::basic(Paint::solid(color), Paint::solid(Color::BLACK), 2.0),
        );
        coast.extras_mut().geography = Some(Box::new(GeographicMetadata { categories: vec!["core:coastline".into()], ..Default::default() }));
        doc.insert(Some(id), 0, coast).unwrap();
    }
    let render = |doc: &Document, zoom| {
        let opts = RenderOptions { world_map_zoom: zoom, ..Default::default() };
        Renderer::new().render(doc, 100, 100, Affine::IDENTITY, &opts)
    };
    let midpoint = render(&doc, Some(3.0));
    let [r, g, b, a] = midpoint.pixel(50, 50);
    assert!(r.abs_diff(128) <= 1 && b.abs_diff(128) <= 1 && g == 0 && a == 255, "{r}, {g}, {b}, {a}");
    assert!(midpoint.pixel(20, 50)[0] > 100, "the coarse outline is absent");
    assert_eq!(midpoint.pixel(30, 50), [0, 0, 0, 255], "the fine outline is drawn once");
    assert!(midpoint.pixel(25, 50)[3].abs_diff(128) <= 1, "the differing silhouette fades");
    let before = render(&doc, Some(2.5));
    assert_eq!(before.pixel(20, 50), [0, 0, 0, 255]);
    assert_ne!(before.pixel(30, 50), [0, 0, 0, 255]);
    assert_eq!(render(&doc, Some(2.0)).pixel(50, 50), [255, 0, 0, 255]);
    assert_eq!(render(&doc, Some(4.0)).pixel(50, 50), [0, 0, 255, 255]);
    let exported = Renderer::new().render_region(&doc, Rect::new(0.0, 0.0, 100.0, 100.0), 1.0, false);
    assert_eq!(exported.pixel(20, 50), [0, 0, 0, 255]);
    assert_eq!(exported.pixel(30, 50), [0, 0, 0, 255]);
    let level = doc.layers.last().unwrap().id;
    doc.node_mut(level).unwrap().extras_mut().detail_level = false;
    assert!(render(&doc, Some(3.0)).pixel(50, 50)[3].abs_diff(192) <= 1, "unmarked layers keep the ordinary fade");
    doc.world_map = None;
    assert_eq!(render(&doc, Some(3.0)).pixels, render(&doc, None).pixels);
}
