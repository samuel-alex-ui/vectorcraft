//! Google Maps-style scale overlay for 2D Aetherys world-map documents.
//! UI-only: not part of exported artwork, hit-testing, or saved vector nodes.

use egui::{Align2, Color32, FontId, Pos2, Rect, Stroke, pos2, vec2};
use vectorcraft_doc::Document;
use vectorcraft_worldmap::CanvasPoint;

use super::Xf;

fn label(metres: f64) -> String {
    if metres >= 1e12 {
        format!("{:.1e} km", metres / 1_000.0)
    } else if metres >= 1_000.0 {
        format!("{} km", (metres / 1_000.0).round() as u64)
    } else if metres >= 1.0 {
        format!("{} m", metres.round() as u64)
    } else if metres >= 0.01 {
        format!("{} cm", (metres * 100.0).round() as u64)
    } else if metres >= 0.001 {
        format!("{} mm", (metres * 1000.0).round() as u64)
    } else {
        format!("{metres:.1e} m")
    }
}

/// Draw an always-up-to-date local horizontal distance scale for this
/// on-screen view. This does *not* represent polygon area (m²), and its
/// spherical-metric approximation varies with latitude and rotation.
pub(super) fn paint(painter: &egui::Painter, xf: &Xf, doc: &Document) {
    let Some(world) = doc.world_map else { return };
    let rect = xf.rect;
    if rect.width() < 200.0 || rect.height() < 90.0 {
        return;
    }
    let left = rect.left() + 16.0;
    let bottom = rect.bottom() - 14.0;
    // Centre of the current viewport is the reference latitude. The
    // location beneath the graphic can be outside the map when fitted,
    // so using that corner would incorrectly report a polar scale.
    let center = xf.to_doc(rect.center());
    let at = CanvasPoint { x: center.x, y: center.y };
    let maximum_bar_width = f64::from((rect.width() - 86.0).min(145.0));
    let bar = world.scale_bar(at, xf.zoom, xf.rot.to_degrees(), maximum_bar_width);
    let text_color = Color32::WHITE;
    let backdrop = Color32::from_black_alpha(187);

    if world.planet_radius_m().is_none() {
        // Never present fictional kilometres: a planet radius must first be
        // configured via File → Document Setup → World.
        let box_width = (rect.width() - 32.0).min(295.0);
        let box_rect = Rect::from_min_size(pos2(left, bottom - 29.0), vec2(box_width, 29.0));
        painter.rect_filled(box_rect, 5.0, backdrop);
        painter.text(
            pos2(left + 9.0, bottom - 14.5),
            Align2::LEFT_CENTER,
            "Set planet radius: Document Setup → World",
            FontId::proportional(11.0),
            text_color,
        );
        return;
    }
    let Some(bar) = bar else { return };
    // This overlay is intentionally placed *above* the cached artwork and
    // under no drawing-tool transformations. Screen-point dimensions stay
    // constant even at 1,000,000× world zoom.
    let width = bar.width_screen_points as f32;
    let panel = Rect::from_min_size(pos2(left, bottom - 44.0), vec2(width + 22.0, 44.0));
    painter.rect_filled(panel, 5.0, backdrop);
    let bar_left = left + 11.0;
    let bar_right = bar_left + width;
    let y = bottom - 9.0;
    let stroke = Stroke::new(2.0, text_color);
    painter.line_segment([pos2(bar_left, y), pos2(bar_right, y)], stroke);
    painter.line_segment([pos2(bar_left, y - 8.0), pos2(bar_left, y + 1.0)], stroke);
    painter.line_segment([pos2(bar_right, y - 8.0), pos2(bar_right, y + 1.0)], stroke);
    painter.text(
        Pos2::new(bar_left, bottom - 31.0),
        Align2::LEFT_TOP,
        format!("≈ {}", label(bar.distance_metres)),
        FontId::proportional(12.0),
        text_color,
    );
}

#[cfg(test)]
mod tests {
    use super::label;

    #[test]
    fn metric_labels_are_human_readable_at_all_zoom_levels() {
        assert_eq!(label(5_000.0), "5 km");
        assert_eq!(label(500.0), "500 m");
        assert_eq!(label(5.0), "5 m");
        assert_eq!(label(0.5), "50 cm");
        assert_eq!(label(0.005), "5 mm");
        assert_eq!(label(0.000005), "5.0e-6 m");
    }
}
