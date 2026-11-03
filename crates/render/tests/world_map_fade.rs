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
