//! AETH-01 design evidence: today's reader is not a lossless carrier for future node metadata.
//! These probes characterize the pre-feature reader; they do not implement geographic semantics.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use serde_json::json;
use vectorcraft_doc::{Document, WorldMap};
use vectorcraft_format::{COMPRESSED_SINCE, MIN_VERSION, SaveOptions, VERSION, load, save_with};

#[test]
fn future_document_metadata_survives_but_future_node_metadata_is_discarded() {
    let mut doc = Document::new(1000.0, 500.0);
    doc.world_map = Some(WorldMap::standard());
    let mut value = serde_json::to_value(&doc).unwrap();
    // A synthetic probe, deliberately not the proposed schema or a real user's map.
    let probe = json!({"schema": 1, "marker": "aeth-01-compatibility-probe"});
    value["aetherys_semantics_probe"] = probe.clone();
    value["layers"][0]["aetherys_semantics_probe"] = probe.clone();
    let before: Document = serde_json::from_value(value).unwrap();
    assert_eq!(before.extra.get("aetherys_semantics_probe"), Some(&probe));

    for version in MIN_VERSION..=VERSION {
        for compress in [false, true] {
            if compress && version < COMPRESSED_SINCE {
                continue;
            }
            let options = SaveOptions { version, compress, ..Default::default() };
            let back = load(&save_with(&before, &options).unwrap()).unwrap();
            let roundtrip = serde_json::to_value(&back).unwrap();
            assert_eq!(back.world_map, doc.world_map);
            assert_eq!(back.layers, doc.layers);
            assert_eq!(roundtrip["aetherys_semantics_probe"], probe);
            assert!(roundtrip["layers"][0].get("aetherys_semantics_probe").is_none());
        }
    }
}

#[test]
fn unknown_world_map_fields_are_rejected_instead_of_preserved() {
    let mut doc = Document::new(1000.0, 500.0);
    doc.world_map = Some(WorldMap::standard());
    let bytes = save_with(&doc, &SaveOptions::default()).unwrap();
    let mut value: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    value["document"]["world_map"]["aetherys_semantics_probe"] = json!(1);
    assert!(load(&serde_json::to_vec(&value).unwrap()).is_err());
    // The original file still opens: no implicit inference or migration is involved.
    assert_eq!(load(&bytes).unwrap().world_map, doc.world_map);
}
