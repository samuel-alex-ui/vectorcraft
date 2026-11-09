//! AETH-02: explicit levels, history, replay and native persistence.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
use serde_json::json;
use vectorcraft_engine::Session;
use vectorcraft_testkit::fixtures::exec;

fn world() -> Session {
    let mut s = Session::new();
    exec(&mut s, "file.newWorldMap", json!({"created":1234567890}));
    s
}
#[test]
fn levels_are_unbounded_by_five_explicit_and_preserve_legacy_and_active_layer() {
    let mut s = world();
    let old = s.doc().unwrap().doc.clone();
    let active = s.doc().unwrap().current_layer();
    assert_eq!(exec(&mut s, "map.detailLevel.list", json!({}))["levels"], json!([]));
    for i in 1..=8 {
        let id = exec(&mut s, "map.detailLevel.create", json!({}))["id"].as_u64().map(vectorcraft_doc::NodeId).unwrap();
        let d = &s.doc().unwrap().doc;
        let n = d.node(id).unwrap();
        assert_eq!(n.name.as_deref(), Some(format!("Ansicht {i}").as_str()));
        assert!(n.extras().detail_level);
        let labels: Vec<_> = n.children().unwrap().iter().map(|n| n.display_name()).collect();
        assert_eq!(labels, ["Konturen & Meere", "Geografie", "Politik", "Wirtschaft", "Infrastruktur"]);
        assert_eq!(s.doc().unwrap().current_layer(), active);
    }
    assert_eq!(s.doc().unwrap().doc.layers.first(), old.layers.first());
    assert_eq!(exec(&mut s, "map.detailLevel.list", json!({}))["levels"].as_array().unwrap().len(), 8);
    let before = s.journal.len();
    exec(&mut s, "map.detailLevel.list", json!({}));
    assert_eq!(s.journal.len(), before);
    let mut replay = Session::new();
    for (c, p) in &s.journal {
        replay.execute(c, p).unwrap();
    }
    assert_eq!(*replay.doc().unwrap().doc, *s.doc().unwrap().doc);
}
#[test]
fn lifecycle_history_and_roundtrip() {
    let mut s = world();
    let initial = s.doc().unwrap().doc.clone();
    let id = exec(&mut s, "map.detailLevel.create", json!({}))["id"].clone();
    let created = s.doc().unwrap().doc.clone();
    exec(&mut s, "edit.undo", json!({}));
    assert_eq!(*s.doc().unwrap().doc, *initial);
    exec(&mut s, "edit.redo", json!({}));
    assert_eq!(*s.doc().unwrap().doc, *created);
    exec(&mut s, "map.detailLevel.rename", json!({"id":id,"name":"Stadt"}));
    let renamed = s.doc().unwrap().doc.clone();
    exec(&mut s, "edit.undo", json!({}));
    assert_eq!(*s.doc().unwrap().doc, *created);
    exec(&mut s, "edit.redo", json!({}));
    assert_eq!(*s.doc().unwrap().doc, *renamed);
    let steps = s.doc().unwrap().history.undo.len();
    assert_eq!(exec(&mut s, "map.detailLevel.rename", json!({"id":id,"name":"Stadt"}))["changed"], 0);
    assert_eq!(s.doc().unwrap().history.undo.len(), steps);
    exec(&mut s, "layer.setCurrent", json!({"id": id}));
    let art = exec(&mut s, "shape.rectangle", json!({"x":10,"y":20,"width":30,"height":40}))["id"].clone();
    let entity = "7a7eb4cc-b286-44dd-9949-4542254e5ee0";
    exec(&mut s, "map.setGeography", json!({"id":art,"geography":{"categories":["core:building"],"geographic_entity_id":entity}}));
    exec(&mut s, "map.setGeography", json!({"id":id,"geography":{"categories":["core:land"],"geographic_entity_id":entity}}));
    let before_copy = s.doc().unwrap().doc.clone();
    let copy = exec(&mut s, "map.detailLevel.duplicate", json!({"id":id}))["ids"][0].as_u64().map(vectorcraft_doc::NodeId).unwrap();
    let copied = s.doc().unwrap().doc.clone();
    let n = copied.node(copy).unwrap();
    assert!(n.extras().detail_level);
    let copied_art = n.children().unwrap().last().unwrap();
    let source_art = before_copy.node(vectorcraft_doc::NodeId(art.as_u64().unwrap())).unwrap();
    assert_ne!(copied_art.id, source_art.id);
    assert_eq!(copied_art.kind, source_art.kind);
    assert_eq!(copied_art.extras().geography.as_ref().unwrap().categories, vec!["core:building"]);
    assert!(copied_art.extras().geography.as_ref().unwrap().geographic_entity_id.is_none());
    assert!(n.extras().geography.as_ref().unwrap().geographic_entity_id.is_none());
    assert_ne!(n.children().unwrap()[0].id, before_copy.node(vectorcraft_doc::NodeId(id.as_u64().unwrap())).unwrap().children().unwrap()[0].id);
    exec(&mut s, "edit.undo", json!({}));
    assert_eq!(*s.doc().unwrap().doc, *before_copy);
    exec(&mut s, "edit.redo", json!({}));
    assert_eq!(*s.doc().unwrap().doc, *copied);
    exec(&mut s, "map.detailLevel.delete", json!({"id":id}));
    exec(&mut s, "edit.undo", json!({}));
    assert_eq!(*s.doc().unwrap().doc, *copied);
    for version in vectorcraft_format::MIN_VERSION..=vectorcraft_format::VERSION {
        for compress in [false, true] {
            if compress && version < vectorcraft_format::COMPRESSED_SINCE {
                continue;
            }
            let bytes = vectorcraft_format::save_with(&copied, &vectorcraft_format::SaveOptions { version, compress, ..Default::default() }).unwrap();
            let back = vectorcraft_format::load(&bytes).unwrap();
            assert_eq!(back.layers, copied.layers);
            assert_eq!(back.map_semantics, copied.map_semantics);
        }
    }
}
#[test]
fn invalid_inputs_and_ordinary_docs_are_unchanged() {
    let mut s = world();
    let id = s.doc().unwrap().doc.layers[0].id;
    let before = s.doc().unwrap().doc.clone();
    let steps = s.doc().unwrap().history.undo.len();
    for p in [json!({"name":""}), json!({"name":false}), json!({"name":"x\n"}), json!({"name":"x".repeat(257)})] {
        assert!(s.execute("map.detailLevel.create", &p).is_err());
    }
    for c in ["rename", "duplicate", "delete"] {
        for id in [json!(id), json!(999999), json!(-1), json!("1")] {
            assert!(s.execute(&format!("map.detailLevel.{c}"), &json!({"id":id,"name":"New"})).is_err());
        }
    }
    assert_eq!(*s.doc().unwrap().doc, *before);
    assert_eq!(s.doc().unwrap().history.undo.len(), steps);
    exec(&mut s, "file.new", json!({}));
    let ordinary = s.doc().unwrap().doc.clone();
    for c in ["create", "rename", "duplicate", "delete", "list"] {
        assert!(s.execute(&format!("map.detailLevel.{c}"), &json!({"id":1,"name":"New"})).is_err());
    }
    assert_eq!(*s.doc().unwrap().doc, *ordinary);
}

#[test]
fn schema_guards_and_ordinary_copy_prevent_silent_metadata_loss() {
    let mut s = world();
    let id = exec(&mut s, "map.detailLevel.create", json!({}))["id"].as_u64().map(vectorcraft_doc::NodeId).unwrap();
    let d = s.doc().unwrap().doc.clone();
    let bytes = vectorcraft_format::save_with(&d, &Default::default()).unwrap();
    let raw: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    for value in [json!(0), json!(2), json!(-1), json!("1"), json!(null)] {
        let mut bad = raw.clone();
        bad["document"]["map_semantics"]["detail_levels_version"] = value;
        assert!(vectorcraft_format::load(&serde_json::to_vec(&bad).unwrap()).is_err());
    }
    let mut bad = raw.clone();
    bad["document"]["map_semantics"].as_object_mut().unwrap().remove("detail_levels_version");
    assert!(vectorcraft_format::load(&serde_json::to_vec(&bad).unwrap()).is_err());
    let mut ordinary = vectorcraft_doc::Document::new(100.0, 100.0);
    let copied = ordinary.reid(d.node(id).unwrap());
    assert!(!copied.extras().detail_level);
    assert!(ordinary.map_semantics.is_none());
    let mut masked = vectorcraft_doc::Node::group(vectorcraft_doc::NodeId(100), vec![]);
    masked.extras_mut().mask = Some(Box::new(vectorcraft_doc::OpacityMask::new(d.node(id).unwrap().clone(), true)));
    let copied_mask = ordinary.reid(&masked);
    assert!(!copied_mask.extras().mask.as_ref().unwrap().art.extras().detail_level);
    assert!(ordinary.map_semantics.is_none());
    // An older AETH-01 reader rejected unknown semantics fields through deny_unknown_fields.
    #[derive(serde::Deserialize)]
    #[serde(deny_unknown_fields)]
    struct OldSchema {
        #[serde(rename = "schema_version")]
        _version: u32,
        #[serde(default, rename = "categories")]
        _categories: std::collections::BTreeMap<String, String>,
    }
    assert!(serde_json::from_value::<OldSchema>(raw["document"]["map_semantics"].clone()).is_err());
    // Removing the last marked layer is fine while a legacy layer remains.
    exec(&mut s, "map.detailLevel.delete", json!({"id":id}));
    assert!(s.doc().unwrap().doc.validate_geography().is_ok());
}

#[test]
fn layer_operations_keep_levels_discoverable_and_last_layer_protected() {
    let mut s = world();
    let legacy = s.doc().unwrap().doc.layers[0].id;
    let id = exec(&mut s, "map.detailLevel.create", json!({}))["id"].clone();
    let parent = exec(&mut s, "map.detailLevel.create", json!({}))["id"].clone();
    exec(&mut s, "layer.move", json!({"id":id,"target":parent,"place":"inside"}));
    assert_eq!(exec(&mut s, "map.detailLevel.list", json!({}))["levels"].as_array().unwrap().len(), 2);
    exec(&mut s, "map.detailLevel.delete", json!({"id":id}));
    exec(&mut s, "layer.delete", json!({"id":legacy}));
    let before = s.doc().unwrap().doc.clone();
    assert!(s.execute("map.detailLevel.delete", &json!({"id":parent})).is_err());
    assert_eq!(*s.doc().unwrap().doc, *before);
    let bytes = vectorcraft_format::save_with(&before, &Default::default()).unwrap();
    assert!(vectorcraft_format::load(&bytes).is_ok());
}
