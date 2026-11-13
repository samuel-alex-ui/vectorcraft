//! Paint-only geographic scale at the latitude of the viewport center.

use super::Xf;
use crate::theme::Tokens;
use egui::{FontId, Painter, Rect, Stroke, pos2, vec2};
use vectorcraft_worldmap::{CanvasPoint, MapError, WorldMap};

fn scale(map: WorldMap, xf: &Xf, max_width: f64) -> Result<(f64, f32), MapError> {
    let (width, height) = map.size();
    // Longitude is immaterial to the local scale. Clamp a center on the
    // pasteboard to the nearest latitude on the map.
    let point = map.unproject(CanvasPoint { x: width / 2.0, y: xf.center.y.clamp(0.0, height) })?;
    let (east, north) = map.local_scale(point.latitude_deg)?;
    let delta = xf.delta_to_doc(vec2(1.0, 0.0));
    let metres_per_point = (east * delta.x).hypot(north * delta.y);
    let limit = metres_per_point * max_width;
    if !limit.is_finite() || limit <= 0.0 {
        return Err(MapError::MeasurementOutOfRange);
    }
    let decade = 10.0_f64.powf(limit.log10().floor());
    let distance = if 5.0 * decade <= limit {
        5.0 * decade
    } else if 2.0 * decade <= limit {
        2.0 * decade
    } else {
        decade
    };
    if distance <= 0.0 {
        return Err(MapError::MeasurementOutOfRange);
    }
    Ok((distance, (distance / metres_per_point).min(max_width) as f32))
}

fn distance_label(metres: f64) -> String {
    if metres >= 1000.0 {
        format!("{:.0} km", metres / 1000.0)
    } else if metres >= 1.0 {
        format!("{metres:.0} m")
    } else if metres >= 0.001 {
        format!("{} m", format!("{metres:.3}").trim_end_matches('0'))
    } else {
        format!("{metres:.0e} m")
    }
}

pub(super) fn paint(painter: &Painter, xf: &Xf, map: WorldMap, theme: &Tokens) {
    let max_width = (xf.rect.width() - 36.0).min(140.0);
    if max_width < 20.0 || xf.rect.height() < 42.0 {
        return;
    }
    let (label, width) = match scale(map, xf, f64::from(max_width)) {
        Ok((metres, width)) => (distance_label(metres), Some(width)),
        Err(MapError::MissingRadius) => (tl!("Set a planet radius to show the scale").to_string(), None),
        Err(_) => (tl!("Scale unavailable at this position").to_string(), None),
    };
    let text = painter.layout(label, FontId::proportional(12.0), theme.text, xf.rect.width() - 36.0);
    let width_needed = width.unwrap_or_default().max(text.size().x);
    let height = text.size().y + if width.is_some() { 19.0 } else { 12.0 };
    let bottom = xf.rect.left_bottom() + vec2(10.0, -10.0);
    let top = bottom - vec2(0.0, height);
    painter.rect_filled(Rect::from_min_size(top, vec2(width_needed + 16.0, height)), 5.0, theme.measure_bg);
    painter.galley(top + vec2(8.0, 6.0), text, theme.text);
    if let Some(width) = width {
        let y = bottom.y - 7.0;
        let left = bottom.x + 8.0;
        let right = left + width;
        let stroke = Stroke::new(1.5, theme.text);
        painter.line_segment([pos2(left, y), pos2(right, y)], stroke);
        for x in [left, right] {
            painter.line_segment([pos2(x, y - 4.0), pos2(x, y + 2.0)], stroke);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use vectorcraft_geom::Point;

    fn view(zoom: f64, y: f64) -> Xf {
        Xf { rect: Rect::from_min_size(pos2(0.0, 0.0), vec2(800.0, 600.0)), zoom, center: Point::new(500.0, y), rot: 0.0 }
    }

    #[test]
    fn scale_uses_radius_latitude_and_zoom() {
        let map = WorldMap::standard().with_planet_radius_m(6_371_000.0).unwrap();
        for (zoom, label) in [(1.0, "5000 km"), (10_000.0, "500 m"), (1_000_000.0, "5 m")] {
            let (metres, width) = scale(map, &view(zoom, 250.0), 140.0).unwrap();
            assert_eq!(distance_label(metres), label);
            assert!(width > 0.0 && width <= 140.0);
            let expected = std::f64::consts::PI * 6_371_000.0 / 500.0 / zoom;
            assert!((metres / f64::from(width) / expected - 1.0).abs() < 1e-6);
        }
        let ratio = |y| {
            let (metres, width) = scale(map, &view(4.0, y), 140.0).unwrap();
            metres / f64::from(width)
        };
        assert!((ratio(250.0 / 3.0) / ratio(250.0) - 0.5).abs() < 1e-6);
        assert_eq!(scale(map, &view(1.0, 0.0), 140.0), Err(MapError::MeasurementOutOfRange));
        assert_eq!(distance_label(0.05), "0.05 m");
    }

    #[test]
    fn a_map_without_radius_has_no_numeric_scale() {
        assert_eq!(scale(WorldMap::standard(), &view(1.0, 250.0), 140.0), Err(MapError::MissingRadius));
    }
}
