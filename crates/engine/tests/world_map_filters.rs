//! AETH-05: persisted filters drive the canvas without changing stored artwork.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
use serde_json::{Value, json};
use vectorcraft_engine::Session;
use vectorcraft_geom::{Affine, Point};
use vectorcraft_render::{RenderOptions, Renderer};

fn run(s: &mut Session, command: &str, params: Value) -> Value {
    s.execute(command, &params).unwrap()
}

#[test]
fn mode_filters_apply_across_levels_and_compose_with_manual_visibility_and_zoom() {
    let mut s = Session::new();
    run(&mut s, "file.newWorldMap", json!({}));
    let mut levels = Vec::new();
    for x in [10, 40] {
        let layer = run(&mut s, "map.detailLevel.create", json!({}))["id"].clone();
        run(&mut s, "layer.setCurrent", json!({"id":layer}));
        let art = run(&mut s, "shape.rectangle", json!({"x":x,"y":10,"width":20,"height":20}))["id"].clone();
        run(&mut s, "paint.setFill", json!({"id":art,"color":"#ff0000"}));
        run(&mut s, "map.setGeography", json!({"id":art,"geography":{"categories":["core:river"],"tags":["major"],"properties":{"basin":"West"}}}));
        levels.push((layer, art));
    }
    let layer = s.doc().unwrap().current_layer();
    let original_art = s.doc().unwrap().doc.layers.clone();
    let pixel = |s: &Session, x, zoom| {
        let opts = RenderOptions { background: Some([255, 255, 255, 255]), world_map_zoom: zoom, ..Default::default() };
        Renderer::new().render(&s.doc().unwrap().doc, 80, 40, Affine::IDENTITY, &opts).pixel(x, 20)
    };
    let white = [255, 255, 255, 255];
    let red = [255, 0, 0, 255];
    let rules = json!([{"kind":"category","id":"core:river"}]);
    run(&mut s, "map.mode.setFilter", json!({"id":2,"rules":rules}));
    assert_eq!(pixel(&s, 20, Some(1.0)), red);
    run(&mut s, "map.mode.select", json!({"id":2}));
    for x in [20, 50] {
        assert_eq!(pixel(&s, x, Some(1.0)), white);
        assert!(
            vectorcraft_engine::doc::hit::hit_test_at_zoom(&s.doc().unwrap().doc, Point::new(f64::from(x), 20.0), Default::default(), 1.0).is_none()
        );
    }
    assert_eq!(pixel(&s, 20, None), red);
    assert_eq!(s.doc().unwrap().doc.layers, original_art);
    assert_eq!(s.doc().unwrap().current_layer(), layer);
    let filtered = s.doc().unwrap().doc.clone();
    let bytes = vectorcraft_format::save_with(&filtered, &Default::default()).unwrap();
    assert_eq!(vectorcraft_format::load(&bytes).unwrap().map_semantics, filtered.map_semantics);
    assert!(s.execute("map.mode.setFilter", &json!({"id":2,"rules":[{"kind":"group","id":levels[0].1}]})).is_err());
    assert_eq!(*s.doc().unwrap().doc, *filtered);
    run(&mut s, "edit.undo", json!({}));
    assert_eq!(pixel(&s, 20, Some(1.0)), red);
    run(&mut s, "edit.redo", json!({}));
    assert_eq!(pixel(&s, 20, Some(1.0)), white);

    for rule in
        [json!({"kind":"group","id":levels[0].0}), json!({"kind":"tag","tag":"major"}), json!({"kind":"property","key":"basin","value":"West"})]
    {
        run(&mut s, "map.mode.setFilter", json!({"id":2,"rules":[rule]}));
        assert_eq!(pixel(&s, 20, Some(1.0)), white);
    }
    run(&mut s, "map.mode.setFilter", json!({"id":2,"rules":[]}));
    run(&mut s, "layer.setProps", json!({"id":levels[0].0,"visible":false}));
    run(&mut s, "layer.setProps", json!({"id":levels[1].0,"zoomFade":{"fade_in":[2,3]}}));
    assert_eq!(pixel(&s, 20, Some(3.0)), white);
    assert_eq!(pixel(&s, 50, Some(1.0)), white);
    assert_eq!(pixel(&s, 50, Some(3.0)), red);
    run(&mut s, "file.new", json!({}));
    assert!(s.execute("map.mode.setFilter", &json!({"id":2,"rules":[]})).is_err());
}
