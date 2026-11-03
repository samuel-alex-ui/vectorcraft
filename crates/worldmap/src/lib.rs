//! Foundations for an editable equirectangular 2D world map.
//!
//! This module is deliberately independent of the VectorCraft document model.
//! A geographic point is the source of truth; canvas positions are a projection
//! used for editing. The same points can later be rendered on a globe.
//!
//! Coordinate convention: west = -180°, east = +180°, north = +90°,
//! south = -90°. The antimeridian has two *display* edges (x=0 and x=width).
//! Callers must split geometries that cross this seam before drawing them.

/// Recommended initial world-map artboard dimensions, in VectorCraft points.
/// These fit a conventional illustration document while preserving 2:1.
pub const DEFAULT_WIDTH: f64 = 1000.0;
pub const DEFAULT_HEIGHT: f64 = 500.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GeographicPoint {
    pub longitude_deg: f64,
    pub latitude_deg: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CanvasPoint {
    pub x: f64,
    pub y: f64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MapError {
    InvalidDimensions,
    InvalidLongitude,
    InvalidLatitude,
    OutsideCanvas,
}

impl std::fmt::Display for MapError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for MapError {}

/// Full-planet equirectangular projection (Plate Carrée), 2:1 aspect ratio.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WorldMap {
    width: f64,
    height: f64,
}

impl WorldMap {
    /// Reject non-finite, non-positive and non-2:1 artboards.
    pub fn new(width: f64, height: f64) -> Result<Self, MapError> {
        if !width.is_finite()
            || !height.is_finite()
            || width <= 0.0
            || height <= 0.0
            || (width / 2.0 - height).abs() > height * 1e-9
        {
            return Err(MapError::InvalidDimensions);
        }
        Ok(Self { width, height })
    }

    pub fn standard() -> Self {
        Self { width: DEFAULT_WIDTH, height: DEFAULT_HEIGHT }
    }

    pub fn size(&self) -> (f64, f64) {
        (self.width, self.height)
    }

    /// Convert coordinates on the planet to positions on the flat canvas.
    /// North is at the top (y=0); positive y points down.
    pub fn project(&self, p: GeographicPoint) -> Result<CanvasPoint, MapError> {
        if !p.longitude_deg.is_finite() || !(-180.0..=180.0).contains(&p.longitude_deg) {
            return Err(MapError::InvalidLongitude);
        }
        if !p.latitude_deg.is_finite() || !(-90.0..=90.0).contains(&p.latitude_deg) {
            return Err(MapError::InvalidLatitude);
        }
        Ok(CanvasPoint {
            x: (p.longitude_deg + 180.0) / 360.0 * self.width,
            y: (90.0 - p.latitude_deg) / 180.0 * self.height,
        })
    }

    /// Convert a position on the flat 2D map back to geographic coordinates.
    pub fn unproject(&self, p: CanvasPoint) -> Result<GeographicPoint, MapError> {
        if !p.x.is_finite()
            || !p.y.is_finite()
            || p.x < 0.0
            || p.x > self.width
            || p.y < 0.0
            || p.y > self.height
        {
            return Err(MapError::OutsideCanvas);
        }
        Ok(GeographicPoint {
            longitude_deg: p.x / self.width * 360.0 - 180.0,
            latitude_deg: 90.0 - p.y / self.height * 180.0,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn map_dimensions_are_two_to_one() {
        assert!(WorldMap::new(1000.0, 500.0).is_ok());
        assert_eq!(WorldMap::new(1000.0, 600.0), Err(MapError::InvalidDimensions));
        assert_eq!(WorldMap::new(f64::NAN, 500.0), Err(MapError::InvalidDimensions));
        assert_eq!(WorldMap::new(0.0, 0.0), Err(MapError::InvalidDimensions));
    }

    #[test]
    fn world_corners_and_center() {
        let map = WorldMap::standard();
        assert_eq!(map.project(GeographicPoint { longitude_deg: -180.0, latitude_deg: 90.0 }), Ok(CanvasPoint { x: 0.0, y: 0.0 }));
        assert_eq!(map.project(GeographicPoint { longitude_deg: 180.0, latitude_deg: -90.0 }), Ok(CanvasPoint { x: 1000.0, y: 500.0 }));
        assert_eq!(map.project(GeographicPoint { longitude_deg: 0.0, latitude_deg: 0.0 }), Ok(CanvasPoint { x: 500.0, y: 250.0 }));
    }

    #[test]
    fn round_trip() {
        let map = WorldMap::standard();
        for (lon, lat) in [(-180.0, -90.0), (180.0, 90.0), (0.0, 0.0), (124.3, -17.8)] {
            let source = GeographicPoint { longitude_deg: lon, latitude_deg: lat };
            let p = map.project(source).expect("valid geographic point");
            let result = map.unproject(p).expect("valid canvas point");
            assert!((result.longitude_deg - lon).abs() < 1e-10);
            assert!((result.latitude_deg - lat).abs() < 1e-10);
        }
    }

    #[test]
    fn rejects_invalid_coordinates() {
        let map = WorldMap::standard();
        assert_eq!(map.project(GeographicPoint { longitude_deg: 181.0, latitude_deg: 0.0 }), Err(MapError::InvalidLongitude));
        assert_eq!(map.project(GeographicPoint { longitude_deg: 0.0, latitude_deg: -91.0 }), Err(MapError::InvalidLatitude));
        assert_eq!(map.unproject(CanvasPoint { x: -1.0, y: 30.0 }), Err(MapError::OutsideCanvas));
        assert_eq!(map.unproject(CanvasPoint { x: f64::INFINITY, y: 30.0 }), Err(MapError::OutsideCanvas));
    }

    #[test]
    fn opposite_edges_are_same_meridian_for_globe_purposes() {
        let map = WorldMap::standard();
        let west = map.unproject(CanvasPoint { x: 0.0, y: 250.0 }).expect("west");
        let east = map.unproject(CanvasPoint { x: 1000.0, y: 250.0 }).expect("east");
        assert_eq!(west.longitude_deg, -180.0);
        assert_eq!(east.longitude_deg, 180.0);
        assert_eq!(west.latitude_deg, east.latitude_deg);
    }
}
