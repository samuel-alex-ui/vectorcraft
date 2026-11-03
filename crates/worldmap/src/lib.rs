//! Foundations for an editable equirectangular 2D world map.
//!
//! This module is deliberately independent of the VectorCraft document model.
//! Vector objects retain their existing document-point geometry; this mapping
//! derives geographic positions without maintaining a second geometry store.
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
    InvalidRadius,
}

impl std::fmt::Display for MapError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for MapError {}

/// Full-planet equirectangular projection (Plate Carrée), 2:1 aspect ratio.
#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(try_from = "MapData", into = "MapData")]
pub struct WorldMap {
    width: f64,
    height: f64,
    planet_radius_m: Option<f64>,
}

impl WorldMap {
    /// Reject non-finite, non-positive and non-2:1 artboards.
    pub fn new(width: f64, height: f64) -> Result<Self, MapError> {
        if !width.is_finite() || !height.is_finite() || width <= 0.0 || height <= 0.0 || (width / 2.0 - height).abs() > height * 1e-9 {
            return Err(MapError::InvalidDimensions);
        }
        Ok(Self { width, height, planet_radius_m: None })
    }

    pub fn standard() -> Self {
        Self { width: DEFAULT_WIDTH, height: DEFAULT_HEIGHT, planet_radius_m: None }
    }

    /// Optional spherical planet radius, in metres. No Earth-sized default is assumed.
    pub fn planet_radius_m(&self) -> Option<f64> {
        self.planet_radius_m
    }

    pub fn with_planet_radius_m(mut self, radius: f64) -> Result<Self, MapError> {
        if !radius.is_finite() || radius <= 0.0 {
            return Err(MapError::InvalidRadius);
        }
        self.planet_radius_m = Some(radius);
        Ok(self)
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
        Ok(CanvasPoint { x: (p.longitude_deg + 180.0) / 360.0 * self.width, y: (90.0 - p.latitude_deg) / 180.0 * self.height })
    }

    /// Convert a position on the flat 2D map back to geographic coordinates.
    pub fn unproject(&self, p: CanvasPoint) -> Result<GeographicPoint, MapError> {
        if !p.x.is_finite() || !p.y.is_finite() || p.x < 0.0 || p.x > self.width || p.y < 0.0 || p.y > self.height {
            return Err(MapError::OutsideCanvas);
        }
        Ok(GeographicPoint { longitude_deg: p.x / self.width * 360.0 - 180.0, latitude_deg: 90.0 - p.y / self.height * 180.0 })
    }

    /// Project a geographic open polyline without connecting across the flat map's seam.
    ///
    /// A longitude jump larger than 180 degrees follows the shorter arc over the
    /// antimeridian. The line is split into two independent canvas polylines at
    /// x=0 and x=width, with linearly interpolated latitude at the crossing.
    /// This is intentionally for *open polylines*: filled polygons require a
    /// separate topology-aware clipping algorithm to preserve their fill rule.
    /// The original points are not changed or duplicated in the document.
    pub fn project_seamless_polyline(&self, points: &[GeographicPoint]) -> Result<Vec<Vec<CanvasPoint>>, MapError> {
        let Some(first) = points.first().copied() else { return Ok(Vec::new()) };
        let mut current = vec![self.project(first)?];
        let mut result = Vec::new();
        for pair in points.windows(2) {
            let (from, to) = (pair[0], pair[1]);
            let endpoint = self.project(to)?;
            let delta = to.longitude_deg - from.longitude_deg;
            if delta.abs() <= 180.0 {
                current.push(endpoint);
                continue;
            }
            // +180 and -180 refer to the same meridian. There is no line to
            // draw between the opposing *flat* edges in that degenerate case.
            if delta.abs() == 360.0 {
                if current.len() > 1 {
                    result.push(std::mem::take(&mut current));
                }
                current = vec![endpoint];
                continue;
            }
            let eastward = delta < 0.0;
            let wrapped_end = to.longitude_deg + if eastward { 360.0 } else { -360.0 };
            let seam = if eastward { 180.0 } else { -180.0 };
            let t = ((seam - from.longitude_deg) / (wrapped_end - from.longitude_deg)).clamp(0.0, 1.0);
            let latitude = from.latitude_deg + t * (to.latitude_deg - from.latitude_deg);
            let y = (90.0 - latitude) / 180.0 * self.height;
            let before = CanvasPoint { x: if eastward { self.width } else { 0.0 }, y };
            let after = CanvasPoint { x: if eastward { 0.0 } else { self.width }, y };
            // At an exact seam endpoint, avoid adding a zero-length segment;
            // a zero-length stroked path can still produce visible end caps.
            if current.last() != Some(&before) {
                current.push(before);
            }
            if current.len() > 1 {
                result.push(std::mem::take(&mut current));
            }
            current = vec![after];
            if endpoint != after {
                current.push(endpoint);
            }
        }
        if current.len() > 1 || points.len() == 1 {
            result.push(current);
        }
        Ok(result)
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

/// Explicit projection discriminator; unknown projections fail rather than being misinterpreted.
#[derive(Clone, Copy, Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
enum Projection {
    Equirectangular,
}

// The full-world angular extent is defined by the projection contract, not duplicated
// as independently mutable bounds. Canvas extent is fixed in document points at (0, 0).
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct MapData {
    projection: Projection,
    width: f64,
    height: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    planet_radius_m: Option<f64>,
}

impl TryFrom<MapData> for WorldMap {
    type Error = MapError;

    fn try_from(data: MapData) -> Result<Self, Self::Error> {
        let map = Self::new(data.width, data.height)?;
        match data.planet_radius_m {
            Some(radius) => map.with_planet_radius_m(radius),
            None => Ok(map),
        }
    }
}

impl From<WorldMap> for MapData {
    fn from(map: WorldMap) -> Self {
        Self { projection: Projection::Equirectangular, width: map.width, height: map.height, planet_radius_m: map.planet_radius_m }
    }
}

#[cfg(test)]
mod metadata_tests {
    use super::*;

    #[test]
    fn metadata_is_validated_and_preserves_coordinates() {
        let map = WorldMap::new(2000.0, 1000.0).unwrap().with_planet_radius_m(1234.5).unwrap();
        let value = serde_json::to_value(map).unwrap();
        assert_eq!(value["projection"], "equirectangular");
        let restored: WorldMap = serde_json::from_value(value).unwrap();
        assert_eq!(map, restored);
        let point = GeographicPoint { longitude_deg: 73.0, latitude_deg: -42.0 };
        assert_eq!(map.project(point), restored.project(point));
        for radius in [0.0, -1.0, f64::NAN, f64::INFINITY] {
            assert_eq!(map.with_planet_radius_m(radius), Err(MapError::InvalidRadius));
        }
        for bad in [
            serde_json::json!({"projection":"equirectangular","width":1000,"height":600}),
            serde_json::json!({"projection":"equirectangular","width":1000,"height":500,"planet_radius_m":-1}),
            serde_json::json!({"projection":"other","width":1000,"height":500}),
        ] {
            assert!(serde_json::from_value::<WorldMap>(bad).is_err());
        }
    }
}
