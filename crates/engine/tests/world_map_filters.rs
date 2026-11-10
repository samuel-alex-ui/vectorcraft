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

#[test]
fn ordered_mode_styles_change_only_visible_solid_fills() {
    let mut s = Session::new();
    run(&mut s, "file.newWorldMap", json!({}));
    let mut shapes = Vec::new();
    for x in [10, 40] {
        let layer = run(&mut s, "map.detailLevel.create", json!({}))["id"].clone();
        run(&mut s, "layer.setCurrent", json!({"id":layer}));
        let id = run(&mut s, "shape.rectangle", json!({"x":x,"y":10,"width":20,"height":20}))["id"].clone();
        run(&mut s, "paint.setFill", json!({"id":id,"color":"#ff0000"}));
        run(&mut s, "map.setGeography", json!({"id":id,"geography":{"categories":["core:land","core:biome"]}}));
        shapes.push((layer, id));
    }
    let art = s.doc().unwrap().doc.layers.clone();
    let rules = json!([{"category":"core:biome","rgb":[0,255,0]},{"category":"core:land","rgb":[0,0,255]}]);
    run(&mut s, "map.mode.setStyles", json!({"id":2,"rules":rules}));
    let preview = |s: &Session, zoom| {
        Renderer::new().render(&s.doc().unwrap().doc, 80, 40, Affine::IDENTITY, &RenderOptions { world_map_zoom: zoom, ..Default::default() })
    };
    assert_eq!(preview(&s, Some(1.0)).pixel(20, 20), [255, 0, 0, 255]);
    run(&mut s, "map.mode.select", json!({"id":2}));
    let export = Renderer::new().render_region(&s.doc().unwrap().doc, vectorcraft_geom::Rect::new(0.0, 0.0, 80.0, 40.0), 1.0, false);
    for x in [20, 50] {
        assert_eq!(preview(&s, Some(1.0)).pixel(x, 20), [0, 255, 0, 255]);
        assert_eq!(export.pixel(x, 20), [255, 0, 0, 255]);
    }
    assert_eq!(s.doc().unwrap().doc.layers, art);
    let styled = s.doc().unwrap().doc.clone();
    let bytes = vectorcraft_format::save_with(&styled, &Default::default()).unwrap();
    assert_eq!(vectorcraft_format::load(&bytes).unwrap().map_semantics, styled.map_semantics);
    let reversed: Vec<_> = rules.as_array().unwrap().iter().rev().cloned().collect();
    run(&mut s, "map.mode.setStyles", json!({"id":2,"rules":reversed}));
    assert_eq!(preview(&s, Some(1.0)).pixel(20, 20), [0, 0, 255, 255]);
    run(&mut s, "edit.undo", json!({}));
    assert_eq!(s.doc().unwrap().doc, styled);
    run(&mut s, "edit.redo", json!({}));
    assert_eq!(preview(&s, Some(1.0)).pixel(20, 20), [0, 0, 255, 255]);
    let duplicate = run(&mut s, "map.mode.edit", json!({"action":"duplicate","id":2,"name":"Copy"}))["id"].clone();
    run(&mut s, "map.mode.select", json!({"id":duplicate}));
    assert_eq!(preview(&s, Some(1.0)).pixel(20, 20), [0, 0, 255, 255]);
    run(&mut s, "map.mode.edit", json!({"action":"delete","id":duplicate}));
    run(&mut s, "map.mode.select", json!({"id":2}));
    assert!(s.execute("map.mode.setStyles", &json!({"id":2,"rules":[rules[0],rules[0]]})).is_err());
    assert!(s.execute("map.mode.setStyles", &json!({"id":2,"rules":[{"category":"bad","rgb":[0,0,0]}]})).is_err());
    run(&mut s, "layer.setProps", json!({"id":shapes[0].0,"zoomFade":{"fade_in":[1,3]}}));
    assert_eq!(preview(&s, Some(1.0)).pixel(20, 20)[3], 0);
    assert!((1..255).contains(&preview(&s, Some(2.0)).pixel(20, 20)[3]));
    run(&mut s, "map.mode.setFilter", json!({"id":2,"rules":[{"kind":"category","id":"core:biome"}]}));
    assert_eq!(preview(&s, Some(3.0)).pixel(50, 20)[3], 0);
    run(&mut s, "file.new", json!({}));
    assert!(s.execute("map.mode.setStyles", &json!({"id":2,"rules":rules})).is_err());
}
