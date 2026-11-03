//! Geographic metadata through the real native envelope, including legacy versions.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use vectorcraft_doc::{Document, WorldMap};
use vectorcraft_format::{SaveOptions, load, save_with};

#[test]
fn world_map_survives_native_versions_and_compression() {
    let mut doc = Document::new(1000.0, 500.0);
    doc.world_map = Some(WorldMap::standard().with_planet_radius_m(6_371_000.0).unwrap());
    for version in vectorcraft_format::MIN_VERSION..=vectorcraft_format::VERSION {
        for compress in [false, true] {
            if compress && version < vectorcraft_format::COMPRESSED_SINCE {
                continue;
            }
            let bytes = save_with(&doc, &SaveOptions { version, compress, ..Default::default() }).unwrap();
            let reopened = load(&bytes).unwrap();
            assert_eq!(reopened.world_map, doc.world_map);
            assert_eq!(reopened.artboards, doc.artboards);
            assert_eq!(reopened.world_map.unwrap().planet_radius_m(), Some(6_371_000.0));
        }
    }
}

#[test]
fn ordinary_documents_are_not_inferred_from_title_or_size() {
    let mut doc = Document::new(1000.0, 500.0);
    doc.title = "Aetherys World Map".into();
    let value = serde_json::to_value(&doc).unwrap();
    assert!(value.get("world_map").is_none());
    assert!(serde_json::from_value::<Document>(value).unwrap().world_map.is_none());
    let bytes = save_with(&doc, &SaveOptions::default()).unwrap();
    assert!(load(&bytes).unwrap().world_map.is_none());
}

#[test]
fn damaged_geography_is_rejected_by_native_loader() {
    let mut doc = Document::new(1000.0, 500.0);
    doc.world_map = Some(WorldMap::standard());
    let bytes = save_with(&doc, &SaveOptions::default()).unwrap();
    let original: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    for bad in [serde_json::json!(0), serde_json::json!(-1), serde_json::json!("NaN"), serde_json::Value::Null] {
        let mut value = original.clone();
        value["document"]["world_map"]["width"] = bad;
        assert!(load(&serde_json::to_vec(&value).unwrap()).is_err());
    }
    let mut value = original;
    value["document"]["world_map"]["projection"] = serde_json::json!("unknown");
    assert!(load(&serde_json::to_vec(&value).unwrap()).is_err());
}
