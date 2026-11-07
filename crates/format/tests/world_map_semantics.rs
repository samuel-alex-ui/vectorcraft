//! Native semantic metadata, format guards and adversarial schema inputs.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
use proptest::prelude::*;
use serde_json::json;
use vectorcraft_doc::{Document, GeographicMetadata, MapSemantics, WorldMap};
use vectorcraft_format::{COMPRESSED_SINCE, MIN_VERSION, SaveOptions, VERSION, load, save_with};

fn document() -> Document {
    let mut d = Document::new(1000.0, 500.0);
    d.world_map = Some(WorldMap::standard().with_content_schema(Some(1)).unwrap());
    let mut schema = MapSemantics::default();
    schema.categories.insert("user:6e3a99d5-e361-4bd0-a286-597691803c33".into(), "Handelsrouten".into());
    d.map_semantics = Some(schema);
    let id = d.layers[0].id;
    d.node_mut(id).unwrap().extras_mut().geography = Some(Box::new(
        serde_json::from_value::<GeographicMetadata>(
            json!({"categories": ["core:land", "user:6e3a99d5-e361-4bd0-a286-597691803c33"], "tags": ["Nordregion"], "properties": {"name": "Island"}, "geographic_entity_id": "7a7eb4cc-b286-44dd-9949-4542254e5ee0"}),
        )
        .unwrap(),
    ));
    d
}

#[test]
fn semantics_survive_all_native_versions_and_compression() {
    let d = document();
    for version in MIN_VERSION..=VERSION {
        for compress in [false, true] {
            if compress && version < COMPRESSED_SINCE {
                continue;
            }
            let bytes = save_with(&d, &SaveOptions { version, compress, ..Default::default() }).unwrap();
            let back = load(&bytes).unwrap();
            assert_eq!(back.layers, d.layers);
            assert_eq!(back.map_semantics, d.map_semantics);
            assert_eq!(back.world_map, d.world_map);
        }
    }
}

#[test]
fn unsupported_inconsistent_and_invalid_metadata_is_rejected() {
    let d = document();
    let bytes = save_with(&d, &Default::default()).unwrap();
    let value: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    for (key, bad) in [("schema_version", json!(2)), ("schema_version", json!(-1)), ("categories", json!({"core:land": "Land"}))] {
        let mut v = value.clone();
        v["document"]["map_semantics"][key] = bad;
        assert!(load(&serde_json::to_vec(&v).unwrap()).is_err());
    }
    for bad in [json!(null), json!(2), json!("1")] {
        let mut v = value.clone();
        v["document"]["world_map"]["content_schema"] = bad;
        assert!(load(&serde_json::to_vec(&v).unwrap()).is_err());
    }
    let mut v = value.clone();
    v["document"].as_object_mut().unwrap().remove("map_semantics");
    assert!(load(&serde_json::to_vec(&v).unwrap()).is_err());
    let mut v = value.clone();
    v["document"].as_object_mut().unwrap().remove("world_map");
    assert!(load(&serde_json::to_vec(&v).unwrap()).is_err());
    let mut v = value;
    v["document"]["layers"][0]["geography"]["properties"] = json!({"x": true});
    assert!(load(&serde_json::to_vec(&v).unwrap()).is_err());
    let mut invalid = d;
    invalid.world_map = None;
    assert!(save_with(&invalid, &Default::default()).is_err());
}

#[test]
fn duplicate_and_oversized_collections_fail_during_reading() {
    for raw in [r#"{"tags":["x","x"]}"#, r#"{"properties":{"x":"a","x":"b"}}"#] {
        assert!(serde_json::from_str::<GeographicMetadata>(raw).is_err());
    }
    for v in [
        json!({"categories": (0..33).map(|n| format!("user:00000000-0000-4000-8000-{n:012x}")).collect::<Vec<_>>()}),
        json!({"tags": ["x".repeat(257)]}),
        json!({"properties": {"x": "v".repeat(4097)}}),
    ] {
        assert!(serde_json::from_value::<GeographicMetadata>(v).is_err());
    }
}

proptest! {
    #![proptest_config(ProptestConfig { cases: 64, failure_persistence: None, ..ProptestConfig::default() })]
    #[test]
    fn hostile_geography_never_panics_and_successful_reads_validate(tag in ".{0,300}", version in any::<u32>()) {
        let mut value: serde_json::Value = serde_json::from_slice(&save_with(&document(), &Default::default()).unwrap()).unwrap();
        value["document"]["layers"][0]["geography"]["tags"] = json!([tag]);
        value["document"]["map_semantics"]["schema_version"] = json!(version);
        if let Ok(back) = load(&serde_json::to_vec(&value).unwrap()) { prop_assert!(back.validate_geography().is_ok()); }
    }
}

#[test]
fn empty_metadata_is_omitted_without_promoting_an_ordinary_document() {
    let mut d = Document::new(100.0, 100.0);
    d.node_mut(d.layers[0].id).unwrap().extras_mut().geography = Some(Box::new(GeographicMetadata::default()));
    let bytes = save_with(&d, &Default::default()).unwrap();
    let value: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert!(value["document"].get("map_semantics").is_none());
    assert!(value["document"]["layers"][0].get("geography").is_none());
    let back = load(&bytes).unwrap();
    assert!(back.world_map.is_none());
    assert!(back.layers[0].extras().geography.is_none());
}
