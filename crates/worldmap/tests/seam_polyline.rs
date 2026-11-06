//! Open coastline polylines cross the antimeridian without drawing a line across the map.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use vectorcraft_worldmap::{GeographicPoint, MapError, WorldMap};

fn point(lon: f64, lat: f64) -> GeographicPoint {
    GeographicPoint { longitude_deg: lon, latitude_deg: lat }
}

#[test]
fn eastward_and_westward_crossings_have_matching_seam_endpoints() {
    let map = WorldMap::standard();
    let east = map.project_seamless_polyline(&[point(170.0, 0.0), point(-170.0, 20.0)]).unwrap();
    assert_eq!(east.len(), 2);
    assert_eq!(east[0].len(), 2);
    assert_eq!(east[1].len(), 2);
    assert_eq!(east[0][1].x, 1000.0);
    assert_eq!(east[1][0].x, 0.0);
    assert!((east[0][1].y - east[1][0].y).abs() < 1e-10);
    assert!((east[0][1].y - 500.0 * 80.0 / 180.0).abs() < 1e-10);

    let west = map.project_seamless_polyline(&[point(-170.0, 20.0), point(170.0, 0.0)]).unwrap();
    assert_eq!(west.len(), 2);
    assert_eq!(west[0][1].x, 0.0);
    assert_eq!(west[1][0].x, 1000.0);
    assert!((west[0][1].y - west[1][0].y).abs() < 1e-10);
}

#[test]
fn repeated_crossings_do_not_create_long_lines_on_the_flat_map() {
    let map = WorldMap::standard();
    let pieces = map
        .project_seamless_polyline(&[point(170.0, 5.0), point(-175.0, 10.0), point(178.0, 15.0), point(-170.0, 20.0), point(-165.0, 30.0)])
        .unwrap();
    assert_eq!(pieces.len(), 4);
    for piece in &pieces {
        for segment in piece.windows(2) {
            assert!((segment[1].x - segment[0].x).abs() < 500.0, "artificial seam connector");
            assert!((0.0..=1000.0).contains(&segment[0].x));
            assert!((0.0..=1000.0).contains(&segment[1].x));
        }
    }
}

#[test]
fn exact_edges_and_invalid_coordinates_do_not_make_artifacts() {
    let map = WorldMap::standard();
    assert!(map.project_seamless_polyline(&[]).unwrap().is_empty());
    assert_eq!(map.project_seamless_polyline(&[point(0.0, 0.0)]).unwrap().len(), 1);
    assert!(map.project_seamless_polyline(&[point(180.0, 0.0), point(-180.0, 0.0)]).unwrap().is_empty());
    let from_edge = map.project_seamless_polyline(&[point(180.0, 0.0), point(-179.0, 1.0)]).unwrap();
    assert_eq!(from_edge.last().unwrap().first().unwrap().x, 0.0);
    assert_eq!(map.project_seamless_polyline(&[point(190.0, 0.0)]), Err(MapError::InvalidLongitude));
    assert_eq!(map.project_seamless_polyline(&[point(0.0, 0.0), point(20.0, 90.01)]), Err(MapError::InvalidLatitude));
}
