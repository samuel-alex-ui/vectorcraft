//! World-map modes are document metadata with independent identity and history.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
use serde_json::json;
use vectorcraft_engine::Session;

#[test]
fn modes_roundtrip_without_touching_art_and_support_undo_redo() {
    let mut s = Session::new();
    s.execute("file.newWorldMap", &json!({})).unwrap();
    let detail = s.execute("map.detailLevel.create", &json!({"name":"Detail"})).unwrap()["id"].clone();
    s.execute("layer.setCurrent", &json!({"id":detail})).unwrap();
    s.execute("shape.rectangle", &json!({"x":10,"y":20,"width":30,"height":40})).unwrap();
    let original = s.doc().unwrap().doc.clone();
    let layer = s.doc().unwrap().current_layer();
    let defaults = s.execute("map.mode.list", &json!({})).unwrap();
    assert_eq!(defaults["active"], 1);
    assert_eq!(defaults["modes"].as_array().unwrap().len(), 6);
    assert_eq!(*s.doc().unwrap().doc, *original);
    let custom = s.execute("map.mode.edit", &json!({"action":"create","name":"Handel"})).unwrap()["id"].clone();
    let duplicate = s.execute("map.mode.edit", &json!({"action":"duplicate","id":custom,"name":"Handel II"})).unwrap()["id"].clone();
    assert_ne!(custom, duplicate);
    s.execute("map.mode.edit", &json!({"action":"rename","id":custom,"name":"Handelsrouten"})).unwrap();
    s.execute("map.mode.select", &json!({"id":custom})).unwrap();
    let selected = s.doc().unwrap().doc.clone();
    assert_eq!(selected.layers, original.layers);
    assert_eq!(s.doc().unwrap().current_layer(), layer);
    s.execute("map.mode.edit", &json!({"action":"delete","id":custom})).unwrap();
    assert_eq!(s.execute("map.mode.list", &json!({})).unwrap()["active"], 1);
    s.execute("edit.undo", &json!({})).unwrap();
    assert_eq!(*s.doc().unwrap().doc, *selected);
    s.execute("edit.redo", &json!({})).unwrap();
    assert_eq!(s.execute("map.mode.list", &json!({})).unwrap()["active"], 1);
    s.execute("edit.undo", &json!({})).unwrap();
    let bytes = vectorcraft_format::save_with(&selected, &Default::default()).unwrap();
    let reopened = vectorcraft_format::load(&bytes).unwrap();
    assert_eq!(reopened.layers, selected.layers);
    assert_eq!(reopened.map_semantics, selected.map_semantics);
    let mut invalid: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    invalid["document"]["map_semantics"]["modes"]["active"] = json!(999);
    assert!(vectorcraft_format::load(&serde_json::to_vec(&invalid).unwrap()).is_err());
}

#[test]
fn bad_edits_are_atomic_and_ordinary_documents_have_no_modes() {
    let mut s = Session::new();
    s.execute("file.new", &json!({})).unwrap();
    for (c, p) in [("map.mode.list", json!({})), ("map.mode.select", json!({"id":1})), ("map.mode.edit", json!({"action":"create","name":"Custom"}))]
    {
        assert!(s.execute(c, &p).is_err());
    }
    s.execute("file.newWorldMap", &json!({})).unwrap();
    let original = s.doc().unwrap().doc.clone();
    assert!(s.execute("map.mode.select", &json!({"id":999})).is_err());
    for p in [
        json!({"action":"create","name":""}),
        json!({"action":"create","name":"a".repeat(257)}),
        json!({"action":"duplicate","id":999,"name":"Copy"}),
    ] {
        assert!(s.execute("map.mode.edit", &p).is_err());
    }
    assert_eq!(*s.doc().unwrap().doc, *original);
    for id in 2..=6 {
        s.execute("map.mode.edit", &json!({"action":"delete","id":id})).unwrap();
    }
    let last = s.doc().unwrap().doc.clone();
    assert!(s.execute("map.mode.edit", &json!({"action":"delete","id":1})).is_err());
    assert_eq!(*s.doc().unwrap().doc, *last);
}
