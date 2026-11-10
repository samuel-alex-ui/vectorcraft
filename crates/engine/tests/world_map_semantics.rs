//! AETH-01: metadata is undoable, explicitly linked, independent of geometry and map modes.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
use serde_json::json;
use vectorcraft_engine::Session;
use vectorcraft_testkit::fixtures::{exec, rect, select};

const ENTITY: &str = "7a7eb4cc-b286-44dd-9949-4542254e5ee0";
const CATEGORY: &str = "user:6e3a99d5-e361-4bd0-a286-597691803c33";
fn world() -> Session {
    let mut s = Session::new();
    exec(&mut s, "file.newWorldMap", json!({"created": 1234567890}));
    s
}

#[test]
fn linked_representations_keep_independent_geometry_and_support_history_and_replay() {
    let mut s = world();
    let coarse = rect(&mut s, 10.0, 10.0, 40.0, 20.0);
    let fine = rect(&mut s, 10.0, 10.0, 41.0, 21.0);
    let old = s.doc().unwrap().doc.clone();
    let steps = s.doc().unwrap().history.undo.len();
    let params = json!({"ids": [coarse, fine], "geography": {"categories": ["core:river", "core:border"], "tags": ["North"], "properties": {"name": "Silver"}, "geographic_entity_id": ENTITY}});
    assert_eq!(exec(&mut s, "map.setGeography", params.clone())["changed"], 2);
    let tagged = s.doc().unwrap().doc.clone();
    assert_eq!(tagged.node(coarse).unwrap().kind, old.node(coarse).unwrap().kind);
    assert_eq!(tagged.node(fine).unwrap().kind, old.node(fine).unwrap().kind);
    assert_eq!(s.doc().unwrap().history.undo.len(), steps + 1);
    assert_eq!(exec(&mut s, "map.setGeography", params)["changed"], 0);
    assert_eq!(s.doc().unwrap().history.undo.len(), steps + 1);
    exec(&mut s, "edit.undo", json!({}));
    assert_eq!(*s.doc().unwrap().doc, *old);
    exec(&mut s, "edit.redo", json!({}));
    assert_eq!(*s.doc().unwrap().doc, *tagged);
    select(&mut s, &[fine]);
    exec(&mut s, "object.move", json!({"dx": 3.0, "dy": 2.0}));
    assert_eq!(s.doc().unwrap().doc.node(coarse).unwrap().kind, old.node(coarse).unwrap().kind);
    assert_ne!(s.doc().unwrap().doc.node(fine).unwrap().kind, tagged.node(fine).unwrap().kind);
    let before_query = s.journal.len();
    let info = exec(&mut s, "map.geographyInfo", json!({"ids": [coarse, fine]}));
    assert_eq!(info["objects"][0]["geography"]["geographic_entity_id"], ENTITY);
    assert_eq!(s.journal.len(), before_query);
    let mut replay = Session::new();
    for (id, p) in &s.journal {
        replay.execute(id, p).unwrap();
    }
    assert_eq!(*replay.doc().unwrap().doc, *s.doc().unwrap().doc);
    let bytes = vectorcraft_format::save_with(&s.doc().unwrap().doc, &Default::default()).unwrap();
    let back = vectorcraft_format::load(&bytes).unwrap();
    assert_eq!(back.layers, s.doc().unwrap().doc.layers);
    assert_eq!(back.map_semantics, s.doc().unwrap().doc.map_semantics);
    let before_clear = s.doc().unwrap().doc.clone();
    exec(&mut s, "map.setGeography", json!({"ids": [coarse, fine], "geography": null}));
    assert!(s.doc().unwrap().doc.node(coarse).unwrap().extras().geography.is_none());
    assert!(s.doc().unwrap().doc.node(fine).unwrap().extras().geography.is_none());
    assert!(s.doc().unwrap().doc.map_semantics.is_some());
    exec(&mut s, "edit.undo", json!({}));
    assert_eq!(*s.doc().unwrap().doc, *before_clear);
    exec(&mut s, "edit.redo", json!({}));
    assert!(s.doc().unwrap().doc.node(fine).unwrap().extras().geography.is_none());
}

#[test]
fn invalid_multi_edits_are_atomic_and_ordinary_documents_are_unchanged() {
    let mut s = world();
    let id = rect(&mut s, 0.0, 0.0, 20.0, 10.0);
    let before = s.doc().unwrap().doc.clone();
    let steps = s.doc().unwrap().history.undo.len();
    for g in [
        json!({"categories": ["core:unknown"]}),
        json!({"tags": ["x", "x"]}),
        json!({"properties": {"count": 2}}),
        json!({"geographic_entity_id": "not-uuid"}),
        json!({"tags": (0..65).map(|n| format!("tag-{n}")).collect::<Vec<_>>()}),
        json!({"properties": {"name": "x".repeat(4097)}}),
    ] {
        assert!(s.execute("map.setGeography", &json!({"id": id, "geography": g})).is_err());
    }
    assert!(s.execute("map.setGeography", &json!({"ids": [id, 999999], "geography": {"categories": ["core:land"]}})).is_err());
    assert_eq!(*s.doc().unwrap().doc, *before);
    assert_eq!(s.doc().unwrap().history.undo.len(), steps);
    let mut normal = vectorcraft_testkit::fixtures::session();
    let old = normal.doc().unwrap().doc.clone();
    assert!(normal.execute("map.setGeography", &json!({"id": 1, "geography": {"tags": ["x"]}})).is_err());
    assert_eq!(*normal.doc().unwrap().doc, *old);
    assert!(serde_json::to_value(&old).unwrap().get("map_semantics").is_none());
}

#[test]
fn categories_have_stable_ids_and_duplicate_breaks_entity_links() {
    let mut s = world();
    let id = rect(&mut s, 0.0, 0.0, 20.0, 10.0);
    exec(&mut s, "map.setCategory", json!({"category": CATEGORY, "label": "Routes"}));
    exec(&mut s, "map.setGeography", json!({"id": id, "geography": {"categories": [CATEGORY], "geographic_entity_id": ENTITY}}));
    exec(&mut s, "map.setCategory", json!({"category": CATEGORY, "label": "Trade"}));
    select(&mut s, &[id]);
    exec(&mut s, "edit.duplicate", json!({}));
    let d = &s.doc().unwrap().doc;
    let mut matches = Vec::new();
    d.walk(|n| {
        if n.extras().geography.is_some() {
            matches.push(n);
        }
    });
    assert_eq!(matches.len(), 2);
    let copy = matches.iter().find(|n| n.id != id).unwrap();
    assert_eq!(copy.extras().geography.as_ref().unwrap().categories, vec![CATEGORY]);
    assert!(copy.extras().geography.as_ref().unwrap().geographic_entity_id.is_none());
    assert_eq!(d.node(id).unwrap().extras().geography.as_ref().unwrap().geographic_entity_id.as_deref(), Some(ENTITY));
    assert_eq!(d.map_semantics.as_ref().unwrap().categories[CATEGORY], "Trade");
    d.validate_geography().unwrap();
    exec(&mut s, "edit.undo", json!({}));
    assert_eq!(s.doc().unwrap().doc.node_count(), 2);
    exec(&mut s, "edit.redo", json!({}));
    assert_eq!(s.doc().unwrap().doc.node_count(), 3);
}

proptest::proptest! {
    #![proptest_config(proptest::test_runner::Config { cases: 32, failure_persistence: None, ..Default::default() })]
    #[test]
    fn geographic_commands_with_hostile_text_are_atomic(tag in ".{0,280}", id in 0u64..5) {
        let mut s = world();
        let before = s.doc().unwrap().doc.clone();
        let steps = s.doc().unwrap().history.undo.len();
        let result = s.execute("map.setGeography", &json!({"id": id, "geography": {"tags": [tag]}}));
        if result.is_err() {
            proptest::prop_assert_eq!(&*s.doc().unwrap().doc, &*before);
            proptest::prop_assert_eq!(s.doc().unwrap().history.undo.len(), steps);
        } else {
            proptest::prop_assert!(s.doc().unwrap().doc.validate_geography().is_ok());
        }
    }
}

#[test]
fn entity_commands_preserve_metadata_and_find_variants_across_levels() {
    let mut s = world();
    let legacy = vectorcraft_format::save_with(&s.doc().unwrap().doc, &Default::default()).unwrap();
    let legacy = vectorcraft_format::load(&legacy).unwrap();
    assert!(legacy.map_semantics.is_none());
    let lookup = |s: &mut Session| exec(s, "map.entityRepresentations", json!({"entity": ENTITY}))["objects"].clone();
    assert_eq!(lookup(&mut s), json!([]));
    let mut variants = Vec::new();
    for x in [10.0, 15.0] {
        let level = exec(&mut s, "map.detailLevel.create", json!({}))["id"].clone();
        exec(&mut s, "layer.setCurrent", json!({"id": level}));
        variants.push((level, rect(&mut s, x, 20.0, 40.0, 30.0)));
    }
    let [(coarse_level, coarse), (fine_level, fine)] = &variants[..] else { panic!("two variants") };
    exec(&mut s, "map.setGeography", json!({"id": coarse, "geography":{"tags":["keep"]}}));
    let before = s.doc().unwrap().doc.clone();
    for params in [json!({"id": coarse,"entity":"bad"}), json!({"ids":[coarse, 999999],"entity":ENTITY}), json!({"id": coarse})] {
        assert!(s.execute("map.setEntity", &params).is_err());
        assert_eq!(s.doc().unwrap().doc, before);
    }
    let link = json!({"ids":[coarse, fine],"entity":ENTITY});
    assert_eq!(exec(&mut s, "map.setEntity", link.clone())["changed"], 2);
    assert_eq!(exec(&mut s, "map.setEntity", link)["changed"], 0);
    let expected = json!([{"id":coarse,"detailLevel":coarse_level},{"id":fine,"detailLevel":fine_level}]);
    assert_eq!(lookup(&mut s), expected);
    let linked = s.doc().unwrap().doc.clone();
    assert_eq!(linked.node(*coarse).unwrap().extras().geography.as_ref().unwrap().tags, ["keep"]);
    let bytes = vectorcraft_format::save_with(&linked, &Default::default()).unwrap();
    assert_eq!(vectorcraft_format::load(&bytes).unwrap().layers, linked.layers);
    exec(&mut s, "edit.undo", json!({}));
    assert_eq!(s.doc().unwrap().doc, before);
    exec(&mut s, "edit.redo", json!({}));
    select(&mut s, &[*fine]);
    exec(&mut s, "object.move", json!({"dx":3,"dy":4}));
    assert_eq!(s.doc().unwrap().doc.node(*coarse), linked.node(*coarse));
    assert_ne!(s.doc().unwrap().doc.node(*fine), linked.node(*fine));
    exec(&mut s, "edit.clear", json!({"ids":[fine]}));
    assert_eq!(lookup(&mut s), json!([{"id":coarse,"detailLevel":coarse_level}]));
    exec(&mut s, "edit.undo", json!({}));
    assert_eq!(lookup(&mut s), expected);
    exec(&mut s, "map.setEntity", json!({"id":coarse,"entity":null}));
    assert_eq!(lookup(&mut s), json!([{"id":fine,"detailLevel":fine_level}]));
    assert_eq!(s.doc().unwrap().doc.node(*coarse).unwrap().extras().geography.as_ref().unwrap().tags, ["keep"]);
    let mut normal = vectorcraft_testkit::fixtures::session();
    assert!(normal.execute("map.setEntity", &json!({"id":1,"entity":ENTITY})).is_err());
    assert!(normal.execute("map.entityRepresentations", &json!({"entity":ENTITY})).is_err());
}
