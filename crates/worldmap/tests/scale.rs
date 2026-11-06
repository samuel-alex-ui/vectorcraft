//! Local (not global geodesic) spherical metric scale bar for the equirectangular editor.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::f64::consts::TAU;

use vectorcraft_worldmap::{CanvasPoint, WorldMap};

fn at(x: f64, y: f64) -> CanvasPoint {
    CanvasPoint { x, y }
}

fn world() -> WorldMap {
    WorldMap::standard().with_planet_radius_m(6_371_000.0).unwrap()
}

#[test]
fn a_world_without_defined_radius_has_no_false_kilometre_scale() {
    assert!(WorldMap::standard().scale_bar(at(500.0, 250.0), 1.0, 0.0, 140.0).is_none());
}

#[test]
fn equator_scale_uses_real_spherical_circumference_and_nice_bar_lengths() {
    let map = world();
    let scale = map.scale_bar(at(500.0, 250.0), 1.0, 0.0, 140.0).unwrap();
    let metres_per_point = TAU * 6_371_000.0 / 1000.0;
    assert!((scale.distance_metres / scale.width_screen_points / metres_per_point - 1.0).abs() < 1e-12);
    assert_eq!(scale.distance_metres, 5_000_000.0);
    assert!(scale.width_screen_points <= 140.0);
    assert!(scale.width_screen_points >= 60.0);
}

#[test]
fn horizontal_distance_shrinks_by_cos_latitude_and_respects_rotation() {
    let map = world();
    let equator = map.scale_bar(at(500.0, 250.0), 1.0, 0.0, 140.0).unwrap();
    let latitude_60 = map.scale_bar(at(500.0, 500.0 / 6.0), 1.0, 0.0, 140.0).unwrap();
    let rotated_60 = map.scale_bar(at(500.0, 500.0 / 6.0), 1.0, 90.0, 140.0).unwrap();
    let metric = |s: vectorcraft_worldmap::ScaleBar| s.distance_metres / s.width_screen_points;
    assert!((metric(latitude_60) / metric(equator) - 0.5).abs() < 1e-12);
    assert!((metric(rotated_60) / metric(equator) - 1.0).abs() < 1e-12);
}

#[test]
fn maximum_zoom_recomputes_the_bar_without_rounding_it_to_zero() {
    let map = world();
    let scale = map.scale_bar(at(500.0, 250.0), 1_000_000.0, 0.0, 150.0).unwrap();
    assert_eq!(scale.distance_metres, 5.0);
    assert!(scale.width_screen_points >= 100.0 && scale.width_screen_points <= 150.0);
}

#[test]
fn nonfinite_and_undefined_scale_requests_are_safely_hidden() {
    let map = world();
    for (point, zoom, rotation, target) in [
        (at(-1.0, 250.0), 1.0, 0.0, 140.0),
        (at(500.0, 501.0), 1.0, 0.0, 140.0),
        (at(f64::NAN, 200.0), 1.0, 0.0, 140.0),
        (at(500.0, 250.0), f64::INFINITY, 0.0, 140.0),
        (at(500.0, 250.0), 0.0, 0.0, 140.0),
        (at(500.0, 250.0), 1.0, f64::NAN, 140.0),
        (at(500.0, 250.0), 1.0, 0.0, -50.0),
        (at(500.0, 0.0), 1.0, 0.0, 140.0),
    ] {
        assert!(map.scale_bar(point, zoom, rotation, target).is_none());
    }
    // At the pole a rotated screen direction along the meridian is valid.
    assert!(map.scale_bar(at(500.0, 0.0), 1.0, 90.0, 140.0).is_some());
}
