//! Camera arithmetic and small vector geometry; high factors do not raise the UI zoom cap.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use egui::{Pos2, Rect as ScreenRect, vec2};
use serde_json::json;
use vectorcraft_engine::{
    Session,
    doc::{Appearance, Document, Node},
};
use vectorcraft_geom::{Affine, Point, Rect, shapes};
use vectorcraft_render::{RenderOptions, Renderer};
use vectorcraft_ui_egui::{VectorcraftApp, canvas::Xf, state::View};

fn viewport() -> ScreenRect {
    ScreenRect::from_center_size(Pos2::new(512.125, 384.125), vec2(1024.0, 768.0))
}

#[test]
fn zoom_precision_rounds_only_the_final_screen_coordinate() {
    let xf = Xf::new(viewport(), &View { center: Point::ZERO, ..View::default() });
    let p = Point::new(-511.99999, -383.99999);
    let c = xf.rect.center();
    let expected = Pos2::new((f64::from(c.x) + p.x) as f32, (f64::from(c.y) + p.y) as f32);
    assert_eq!(xf.to_screen(p), expected);
}

#[test]
fn zoom_precision_promotes_screen_coordinates_before_subtraction() {
    let xf = Xf::new(viewport(), &View { center: Point::ZERO, ..View::default() });
    let p = Pos2::new(0.12501, 0.12501);
    let c = xf.rect.center();
    assert_eq!(xf.to_doc(p), Point::new(f64::from(p.x) - f64::from(c.x), f64::from(p.y) - f64::from(c.y)));
}

#[test]
fn zoom_precision_camera_roundtrips_at_large_factors_and_document_extents() {
    for center in [Point::new(500.0, 250.0), Point::new(3_999_999.0, -3_999_999.0)] {
        for zoom in [0.0313, 1.0, 640.0, 40_000.0, 400_000.0, 1_000_000.0] {
            for rotation in [0.0, 33.0, 90.0, -167.0] {
                let xf = Xf::new(viewport(), &View { center, zoom, rotation, ..View::default() });
                for p in [Pos2::new(12.125, 24.125), viewport().center(), Pos2::new(1000.125, 740.125)] {
                    let doc = xf.to_doc(p);
                    assert!(doc.x.is_finite() && doc.y.is_finite());
                    let affine = xf.affine() * doc;
                    assert!((affine.x - f64::from(p.x)).abs() < 0.01);
                    assert!((affine.y - f64::from(p.y)).abs() < 0.01);
                    let back = xf.to_screen(doc);
                    assert!(back.distance(p) < 0.002, "center={center:?}, zoom={zoom}, rotation={rotation}: {back:?} != {p:?}");
                }
                let step = xf.delta_to_doc(vec2(1.0, 0.0));
                assert!((step.hypot() * zoom - 1.0).abs() < 1e-12);
            }
        }
    }
}

#[test]
fn zoom_precision_navigation_preserves_documents_and_existing_zoom_limits() {
    for command in ["file.new", "file.newWorldMap"] {
        let mut app = VectorcraftApp::new(Session::new(), Default::default());
        app.run(command, json!({})).unwrap();
        app.run("shape.rectangle", json!({"x":500.000001,"y":250.000001,"width":0.00025,"height":0.00025})).unwrap();
        let before = app.session.active().unwrap().doc.clone();
        for (percent, expected) in [(100_000_000.0, 640.0), (0.001, 0.0313), (100.0, 1.0)] {
            app.run("view.setZoom", json!({"zoom":percent, "center":[500.000001,250.000001]})).unwrap();
            assert_eq!(app.view().unwrap().zoom, if command == "file.newWorldMap" && expected == 640.0 { 1_000_000.0 } else { expected });
            assert_eq!(app.session.active().unwrap().doc, before);
        }
        // Existing exception: Print Size presets reach 64,000 displayed percent
        // (853 1/3 internally), while Set Zoom still clamps the internal factor to 640.
        app.session.prefs.display_print_size = true;
        app.run("view.setZoom", json!({"zoom":64_000.0})).unwrap();
        assert_eq!(app.view().unwrap().zoom, if command == "file.newWorldMap" { 64_000.0 / 100.0 * (96.0 / 72.0) } else { 640.0 });
        app.run("view.zoomIn", json!({})).unwrap();
        assert!(
            (app.view().unwrap().zoom - if command == "file.newWorldMap" { 2.0 * 640.0 * (96.0 / 72.0) } else { 640.0 * (96.0 / 72.0) }).abs()
                < 1e-10
        );
        assert_eq!(app.session.active().unwrap().doc, before);
    }
}

#[test]
fn zoom_precision_simple_vector_renders_at_large_factors() {
    use vectorcraft_engine::doc::color::{Color, Paint};
    // A narrow, isolated rectangle, not evidence for effects, dense maps or all drawing tools.
    for zoom in [640.0, 40_000.0, 400_000.0, 1_000_000.0] {
        let mut doc = Document::new(1000.0, 500.0);
        let radius = 10.0 / zoom;
        let node = Node::path(
            doc.alloc_id(),
            shapes::rectangle(Rect::new(500.0 - radius, 250.0 - radius, 500.0 + radius, 250.0 + radius)),
            Appearance::basic(Paint::solid(Color::rgb(1.0, 0.0, 0.0)), Paint::None, 0.0),
        );
        doc.insert(Some(doc.layers[0].id), 0, node).unwrap();
        let view = Affine::translate((32.0, 32.0)) * Affine::scale(zoom) * Affine::translate((-500.0, -250.0));
        let image = Renderer::new().render(&doc, 64, 64, view, &RenderOptions { background: Some([255, 255, 255, 255]), ..Default::default() });
        assert_eq!(image.pixel(32, 32), [255, 0, 0, 255], "zoom={zoom}");
        assert_eq!(image.pixel(8, 8), [255, 255, 255, 255], "zoom={zoom}");
    }
}

#[test]
fn world_map_zoom_limits_and_restore_follow_the_document() {
    use vectorcraft_engine::doc::SavedView;
    let saved = SavedView { name: "house".into(), center: Point::new(500.0, 250.0), zoom: 2_000_000.0, rotation: 33.0 };
    assert_eq!(View::saved_for(&saved, true).unwrap().zoom, 1_000_000.0);
    assert_eq!(View::saved(&saved).unwrap().zoom, 640.0);
    let mut app = VectorcraftApp::new(Session::new(), Default::default());
    app.run("file.newWorldMap", json!({})).unwrap();
    app.run("view.setZoom", json!({"zoom":100_000_000.0})).unwrap();
    assert_eq!(app.view().unwrap().zoom, 1_000_000.0);
    app.run("view.zoomIn", json!({})).unwrap();
    assert_eq!(app.view().unwrap().zoom, 1_000_000.0);
    app.run("view.zoomOut", json!({})).unwrap();
    assert_eq!(app.view().unwrap().zoom, 500_000.0);
    app.run("view.saved.new", json!({"name":"house"})).unwrap();
    app.run("view.setZoom", json!({"zoom":100.0})).unwrap();
    app.run("view.goto1", json!({})).unwrap();
    assert_eq!(app.view().unwrap().zoom, 500_000.0);
    // The stored engine view retains its full factor; restoration uses the document policy.
    let doc = &app.session.active().unwrap().doc;
    assert_eq!(View::saved_for(&doc.views[0], true).unwrap().zoom, 500_000.0);
}

#[test]
fn world_map_zoom_draw_select_and_move_a_small_rectangle() {
    use vectorcraft_engine::ViewInfo;
    use vectorcraft_tools::{PointerEvent, PointerKind};
    let mut session = Session::new();
    session.execute("file.newWorldMap", &json!({})).unwrap();
    let view =
        ViewInfo { zoom: 1_000_000.0, smart_guides: false, snap_to_point: false, show_bbox: false, corner_widgets: false, ..ViewInfo::default() };
    session.select_tool("rectangle", view).unwrap();
    for (kind, x, y) in [(PointerKind::Down, 500.0, 250.0), (PointerKind::Drag, 500.0001, 250.0001), (PointerKind::Up, 500.0001, 250.0001)] {
        session.pointer(&PointerEvent::new(kind, x, y), view).unwrap();
    }
    let id = *session.active().unwrap().selection.objects.first().unwrap();
    let before = session.active().unwrap().doc.node(id).unwrap().geometric_bounds().unwrap();
    assert!((before.width() - 0.0001).abs() < 1e-10);
    session.execute("select.none", &json!({})).unwrap();
    session.select_tool("selection", view).unwrap();
    for (kind, x, y) in
        [(PointerKind::Down, 500.00005, 250.00005), (PointerKind::Drag, 500.00007, 250.00005), (PointerKind::Up, 500.00007, 250.00005)]
    {
        session.pointer(&PointerEvent::new(kind, x, y), view).unwrap();
    }
    assert!(session.active().unwrap().selection.objects.contains(&id));
    let after = session.active().unwrap().doc.node(id).unwrap().geometric_bounds().unwrap();
    assert!((after.x0 - before.x0 - 0.00002).abs() < 1e-10);
    session.execute("edit.undo", &json!({})).unwrap();
    let restored = session.active().unwrap().doc.node(id).unwrap().geometric_bounds().unwrap();
    assert!((restored.x0 - before.x0).abs() < 1e-10);
}
