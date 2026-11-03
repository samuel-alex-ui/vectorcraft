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
            assert_eq!(app.view().unwrap().zoom, expected);
            assert_eq!(app.session.active().unwrap().doc, before);
        }
        // Existing exception: Print Size presets reach 64,000 displayed percent
        // (853 1/3 internally), while Set Zoom still clamps the internal factor to 640.
        app.session.prefs.display_print_size = true;
        app.run("view.setZoom", json!({"zoom":64_000.0})).unwrap();
        assert_eq!(app.view().unwrap().zoom, 640.0);
        app.run("view.zoomIn", json!({})).unwrap();
        assert!((app.view().unwrap().zoom - 640.0 * (96.0 / 72.0)).abs() < 1e-10);
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
