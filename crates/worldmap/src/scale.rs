//! A local physical scale for a 2D equirectangular spherical world map.
//!
//! Distances are **local approximations**, not polygon areas or a geodesic
//! measurement tool. A radius is mandatory: no invented Earth-size fallback.
//! At high latitudes the equirectangular horizontal map scale contracts by
//! cos(latitude); a rotated view also mixes the north/south map direction.

use std::f64::consts::{PI, TAU};

use crate::{CanvasPoint, WorldMap};

/// One visually convenient metric distance and its on-screen bar width.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ScaleBar {
    pub distance_metres: f64,
    pub width_screen_points: f64,
}

impl WorldMap {
    /// A Google Maps-style local metric scale bar at the supplied map point.
    ///
    /// `zoom` is screen **points** per document point (not device pixels).
    /// `rotation_degrees` is the clockwise view rotation; the physical scale
    /// of a horizontal on-screen bar is computed in that direction. The bar's
    /// physical distance follows the spherical metric *locally*, evaluated
    /// at `at`, and chooses a 1/2/5 × 10^n value no longer than the target.
    ///
    /// Returns None until a planet radius has been configured. Also returns
    /// None for invalid/extreme coordinates or for an undefined horizontal
    /// distance right at the poles. No document data is modified.
    pub fn scale_bar(
        &self,
        at: CanvasPoint,
        zoom: f64,
        rotation_degrees: f64,
        target_screen_points: f64,
    ) -> Option<ScaleBar> {
        let radius = self.planet_radius_m?;
        let (width, height) = self.size();
        if ![at.x, at.y, zoom, rotation_degrees, target_screen_points].iter().all(|n| n.is_finite())
            || !(0.0..=width).contains(&at.x)
            || !(0.0..=height).contains(&at.y)
            || zoom <= 0.0
            || target_screen_points <= 0.0
        {
            return None;
        }
        let latitude = (0.5 - at.y / height) * PI;
        let (sin_rotation, cos_rotation) = rotation_degrees.to_radians().sin_cos();
        // Equirectangular: dλ/dx=2π/width and dφ/dy=π/height.
        // Because width = 2*height, both base metres/pt factors are equal.
        let local_factor = (latitude.cos() * cos_rotation).hypot(sin_rotation);
        let metres_per_screen_point = TAU * radius / width / zoom * local_factor;
        let maximum = metres_per_screen_point * target_screen_points;
        if local_factor <= 1e-12 || !metres_per_screen_point.is_finite() || metres_per_screen_point <= 0.0 || !maximum.is_finite() || maximum <= 0.0 {
            return None;
        }
        // Quantize in metres to familiar 1, 2, 5, 10, 20, 50… bars.
        // Extremely small/large values outside f64's useful decimal range
        // return None rather than showing a misleading zero/∞ label.
        let power = 10f64.powf(maximum.log10().floor());
        if !power.is_finite() || power <= 0.0 {
            return None;
        }
        let nice = [5.0, 2.0, 1.0]
            .iter()
            .map(|multiple| multiple * power)
            .find(|d| d.is_finite() && *d > 0.0 && *d <= maximum)
            .or_else(|| {
                let lower = power / 10.0;
                (lower > 0.0).then_some(5.0 * lower)
            })?;
        let bar_width = nice / metres_per_screen_point;
        if !bar_width.is_finite() || bar_width <= 0.0 {
            return None;
        }
        Some(ScaleBar { distance_metres: nice, width_screen_points: bar_width })
    }
}
