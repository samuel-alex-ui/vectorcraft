//! Undoable, validated Layer Options for geographic detail.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use serde_json::json;
use vectorcraft_engine::Session;

#[test]
fn layer_zoom_fade_is_editable_only_in_world_maps_and_can_be_cleared() {
    let mut s = Session::new();
    s.execute("file.newWorldMap", &json!({})).unwrap();
    let id = s.doc().unwrap().doc.layers[0].id;
    s.execute(
        "layer.setProps",
        &json!({
            "id": id.0,
            "zoomFade": {"fade_out":[200.0,500.0]}
        }),
    )
    .unwrap();
    let fade = s.doc().unwrap().doc.node(id).unwrap().extras().zoom_fade.unwrap();
    assert_eq!(fade.opacity(200.0), 1.0);
    assert_eq!(fade.opacity(500.0), 0.0);
    assert!(s.execute("layer.setProps", &json!({"id":id.0,"zoomFade":{"fade_in":[500.0,200.0]}})).is_err());
    assert_eq!(s.doc().unwrap().doc.node(id).unwrap().extras().zoom_fade, Some(fade));
    s.execute("layer.setProps", &json!({"id":id.0,"zoomFade":null})).unwrap();
    assert!(s.doc().unwrap().doc.node(id).unwrap().extras().zoom_fade.is_none());

    let child = s.execute("shape.rectangle", &json!({"x":0,"y":0,"width":20,"height":20})).unwrap()["id"].as_u64().unwrap();
    let child = vectorcraft_doc::NodeId(child);
    s.execute("layer.setProps", &json!({"id":child.0,"zoomFade":{}})).unwrap();
    assert!(s.doc().unwrap().doc.node(child).unwrap().extras().zoom_fade.is_some());
    s.execute("edit.undo", &json!({})).unwrap();
    assert!(s.doc().unwrap().doc.node(child).unwrap().extras().zoom_fade.is_none());
    s.execute("edit.redo", &json!({})).unwrap();
    assert!(s.doc().unwrap().doc.node(child).unwrap().extras().zoom_fade.is_some());

    s.execute("file.new", &json!({})).unwrap();
    let id = s.doc().unwrap().doc.layers[0].id;
    assert!(s.execute("layer.setProps", &json!({"id":id.0,"zoomFade":{"fade_out":[200.0,500.0]}})).is_err());
}
