//! World-map layer detail metadata roundtrips in native files.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use serde_json::json;
use vectorcraft_doc::{Document, WorldMap, ZoomFade};
use vectorcraft_format::{SaveOptions, load, save_with};

#[test]
fn zoom_fade_survives_native_save_and_load() {
    let mut doc = Document::new(1000.0, 500.0);
    doc.world_map = Some(WorldMap::standard());
    let layer = doc.layers[0].id;
    let expected = ZoomFade::new(Some([200.0, 500.0]), Some([2000.0, 5000.0])).unwrap();
    doc.node_mut(layer).unwrap().extras_mut().zoom_fade = Some(expected);
    let bytes = save_with(&doc, &SaveOptions::default()).unwrap();
    let restored = load(&bytes).unwrap();
    assert_eq!(restored.world_map, doc.world_map);
    assert_eq!(restored.node(layer).unwrap().extras().zoom_fade, Some(expected));
    assert_eq!(restored.node(layer).unwrap().opacity, 1.0);
}

#[test]
fn documents_without_zoom_fade_keep_original_serialization() {
    let doc = Document::new(1000.0, 500.0);
    let raw = serde_json::to_value(&doc).unwrap();
    assert!(raw["layers"][0].get("zoom_fade").is_none());
    let saved = save_with(&doc, &SaveOptions::default()).unwrap();
    assert!(load(&saved).unwrap().layers[0].extras().zoom_fade.is_none());
    let mut ordinary = doc;
    ordinary.node_mut(ordinary.layers[0].id).unwrap().extras_mut().zoom_fade =
        Some(ZoomFade::new(None, Some([200.0, 500.0])).unwrap());
    let raw = serde_json::to_value(&ordinary).unwrap();
    assert_eq!(raw["layers"][0]["zoom_fade"], json!({"fade_out":[200.0,500.0]}));
}