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
            let options = [true, false].map(|disable_world_map_index| RenderOptions {
                world_map_zoom: Some(zoom),
                disable_world_map_index,
                ..Default::default()
            });
            let mut renderers = [Renderer::new(), Renderer::new()];
            let mut samples = [Vec::new(), Vec::new()];
            for frame in 0..12 {
                let pan = f64::from(frame % 3) * 24.0;
                let view = Affine::translate((640.0 + pan, 360.0)) * Affine::scale(zoom) * Affine::translate(-center.to_vec2());
                let mut results = [None, None];
                for index in if frame % 2 == 0 { [0, 1] } else { [1, 0] } {
                    let start = Instant::now();
                    let image = renderers[index].render(doc, 1280, 720, view, &options[index]);
                    let elapsed = start.elapsed().as_secs_f64() * 1000.0;
                    results[index] = Some(black_box(image.pixels));
                    if frame >= 3 {
                        samples[index].push(elapsed);
                    }
                }
                assert!(results[0].as_ref().unwrap().chunks_exact(4).any(|p| p[3] != 0));
                assert!(results[0] == results[1], "pixels changed for {label} at {zoom}×, frame {frame}");
            }
            for series in &mut samples {
                series.sort_by(f64::total_cmp);
            }
            println!("{label}, {zoom}×: {:.2} → {:.2} ms median", samples[0][4], samples[1][4]);
        }
    }
}

#[test]
fn indexed_group_matches_reference_after_an_edit() {
    let mut doc = Document::new(1000.0, 500.0);
    doc.world_map = Some(WorldMap::standard());
    let layer = doc.layers[0].id;
    let mut children = Vec::new();
    for x in 0..180 {
        let color = if x % 2 == 0 { Color::BLACK } else { Color::WHITE };
        let left = f64::from(x) * 4.0;
        let path = shapes::rectangle(Rect::new(left, 20.0, left + 9.0, 45.0));
        children.push(Arc::new(Node::path(doc.alloc_id(), path, Appearance::basic(Paint::solid(color), Paint::None, 0.0))));
    }
    let first = children[0].id;
    let group = Node::group(doc.alloc_id(), children);
    doc.insert(Some(layer), 0, group).unwrap();
    let mut indexed = Renderer::new();
    let mut reference = Renderer::new();
    let opts = RenderOptions { world_map_zoom: Some(4.0), ..Default::default() };
    let linear = RenderOptions { disable_world_map_index: true, ..opts.clone() };
    let view = Affine::translate((20.0, 0.0)) * Affine::rotate(0.1) * Affine::scale(2.0);
    for visible in [true, false] {
        doc.node_mut(first).unwrap().visible = visible;
        let a = indexed.render(&doc, 160, 120, view, &opts);
        let b = reference.render(&doc, 160, 120, view, &linear);
        assert!(a.pixels == b.pixels);
    }
}
