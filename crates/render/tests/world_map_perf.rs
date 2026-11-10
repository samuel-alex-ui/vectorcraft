//! Repeatable render baseline for the AETH-09 large-map cases.
#![allow(clippy::unwrap_used)]

use std::{hint::black_box, sync::Arc, time::Instant};
use vectorcraft_color::{Color, Paint};
use vectorcraft_doc::{Appearance, Document, Node, WorldMap};
use vectorcraft_geom::{Affine, BezPath, PathData, Point, Rect, shapes};
use vectorcraft_render::{RenderOptions, Renderer};

#[test]
#[ignore = "manual timing diagnostic, with no CI timing threshold"]
fn world_map_render_baseline() {
    let mut grid = Document::new(1000.0, 500.0);
    grid.world_map = Some(WorldMap::standard());
    let paint = Appearance::basic(Paint::solid(Color::BLACK), Paint::None, 0.0);
    let mut paths = Vec::with_capacity(50_000);
    for row in 0..200 {
        for col in 0..250 {
            let (x, y) = (f64::from(col) * 4.0, f64::from(row) * 2.5);
            let shape = shapes::rectangle(Rect::new(x, y, x + 1.5, y + 1.0));
            paths.push(Arc::new(Node::path(grid.alloc_id(), shape, paint.clone())));
        }
    }
    let layer = grid.layers[0].id;
    let group = Node::group(grid.alloc_id(), paths);
    grid.insert(Some(layer), 0, group).unwrap();

    let mut coast = Document::new(1000.0, 500.0);
    coast.world_map = grid.world_map;
    let mut path = BezPath::new();
    for i in 0..32_000 {
        let angle = f64::from(i) * std::f64::consts::TAU / 32_000.0;
        let radius = 100.0 + 2.0 * (angle * 197.0).sin();
        let p = Point::new(500.0 + radius * angle.cos(), 250.0 + radius * angle.sin());
        if i == 0 {
            path.move_to(p);
        } else {
            path.line_to(p);
        }
    }
    path.close_path();
    let layer = coast.layers[0].id;
    let shape = Node::path(coast.alloc_id(), PathData::from_bezpath(&path), paint);
    coast.insert(Some(layer), 0, shape).unwrap();

    for (label, doc, center) in [("50,000 paths", &grid, Point::new(500.0, 250.5)), ("32,000-vertex coastline", &coast, Point::new(600.0, 250.0))] {
        for zoom in [4.0, 1_000_000.0] {
            let opts = RenderOptions { world_map_zoom: Some(zoom), ..Default::default() };
            let mut renderer = Renderer::new();
            let mut samples = Vec::new();
            for frame in 0..12 {
                let pan = f64::from(frame % 3) * 24.0;
                let view = Affine::translate((640.0 + pan, 360.0)) * Affine::scale(zoom) * Affine::translate(-center.to_vec2());
                let start = Instant::now();
                let image = renderer.render(doc, 1280, 720, view, &opts);
                let elapsed = start.elapsed().as_secs_f64() * 1000.0;
                assert!(image.pixels.chunks_exact(4).any(|p| p[3] != 0));
                black_box(image);
                if frame >= 3 {
                    samples.push(elapsed);
                }
            }
            samples.sort_by(f64::total_cmp);
            println!("{label}, {zoom}×: {:.2} ms median", samples[4]);
        }
    }
}
