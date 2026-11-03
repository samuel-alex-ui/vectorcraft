//! Geographic precision at map edges and at building-sized canvas separations.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use vectorcraft_worldmap::{CanvasPoint, GeographicPoint, WorldMap};

#[test]
fn building_scale_geography_roundtrips_without_changing_the_reference() {
    let map = WorldMap::standard();
    let before = map;
    for x in [0.000001, 500.0, 999.999999] {
        for y in [0.000001, 250.0, 499.999999] {
            let p = CanvasPoint { x, y };
            let q = CanvasPoint { x: x + 0.00000025, y: y + 0.00000025 };
            let a = map.project(map.unproject(p).unwrap()).unwrap();
            let b = map.project(map.unproject(q).unwrap()).unwrap();
            // At a synthetic zoom of 1,000,000, the projection round-trip error
            // must remain below 0.000001 screen point; adjacent details stay distinct.
            for (actual, expected) in [(a.x, p.x), (a.y, p.y), (b.x, q.x), (b.y, q.y)] {
                assert!((actual - expected).abs() * 1_000_000.0 < 0.000001);
            }
            assert!(b.x > a.x && b.y > a.y);
        }
    }
    assert_eq!(map, before);
}

#[test]
fn near_seam_and_poles_keep_their_geographic_coordinates() {
    let map = WorldMap::standard();
    for lon in [-180.0, -179.9999999, 0.0, 179.9999999, 180.0] {
        for lat in [-90.0, -89.9999999, 0.0, 89.9999999, 90.0] {
            let p = GeographicPoint { longitude_deg: lon, latitude_deg: lat };
            let back = map.unproject(map.project(p).unwrap()).unwrap();
            assert!((back.longitude_deg - lon).abs() < 1e-11);
            assert!((back.latitude_deg - lat).abs() < 1e-11);
        }
    }
}
