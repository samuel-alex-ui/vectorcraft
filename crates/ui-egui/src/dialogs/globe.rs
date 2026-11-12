//! Read-only globe preview of the active world's rendered 2D artwork.
use std::sync::Arc;

use egui::{Color32, ColorImage, TextureHandle};
use serde_json::{Value, json};
use vectorcraft_engine::doc::GeographicPoint;
use vectorcraft_geom::{Point, Rect};
use vectorcraft_render::{RenderOptions, Rendered, Renderer};

use super::DialogSpec;
use crate::{VectorcraftApp, state::Dialog};

pub const KIND: &str = "globe";
pub(super) const SPEC: DialogSpec =
    DialogSpec { heading: |_| tl!("3D Globe").into(), body, confirm: center_map, ok: Some("Center 2D Map"), min_width: 260.0, ..DialogSpec::FORM };

const DIAMETER: usize = 480;

#[derive(Clone, Copy, PartialEq)]
struct Camera {
    lon: f64,
    lat: f64,
    zoom: f64,
}

impl Camera {
    fn of(d: &Dialog) -> Self {
        let number = |key, fallback| d.fields.get(key).and_then(Value::as_f64).filter(|v| v.is_finite()).unwrap_or(fallback);
        Self { lon: longitude(number("longitude", 0.0)), lat: number("latitude", 0.0).clamp(-90.0, 90.0), zoom: number("zoom", 1.0).clamp(0.65, 4.0) }
    }
}

fn longitude(degrees: f64) -> f64 {
    (degrees + 180.0).rem_euclid(360.0) - 180.0
}

#[derive(Clone, Copy, PartialEq)]
struct Source {
    uid: u64,
    revision: u64,
    zoom: f64,
    templates: bool,
}

#[derive(Clone)]
struct Preview {
    source: Source,
    map: Arc<Rendered>,
    camera: Camera,
    globe: TextureHandle,
}

pub fn open(app: &mut VectorcraftApp) -> Result<Value, String> {
    let st = app.session.active().ok_or("Requires a world-map document")?;
    let map = st.doc.world_map.ok_or("Requires a world-map document")?;
    let (w, h) = map.size();
    let center = app.view().filter(|v| v.fitted).map_or(Point::new(w * 0.5, h * 0.5), |v| v.center);
    app.ui.dialog = Some(Dialog::new(
        KIND,
        json!({
            "document": st.uid,
            "longitude": center.x.clamp(0.0, w) / w * 360.0 - 180.0,
            "latitude": 90.0 - center.y.clamp(0.0, h) / h * 180.0,
            "zoom": 1.0
        }),
    ));
    Ok(Value::Null)
}

fn body(app: &mut VectorcraftApp, ui: &mut egui::Ui, d: &mut Dialog) -> bool {
    let Some(st) = app.session.active().filter(|st| Some(st.uid) == d.fields.get("document").and_then(Value::as_u64)) else {
        return true;
    };
    let Some(map) = st.doc.world_map else { return true };
    let source = Source { uid: st.uid, revision: st.revision, zoom: app.view().map_or(1.0, |v| v.zoom), templates: app.ui.view.templates };
    let (width, height) = map.size();
    let scale = 1024.0 / width;
    if !scale.is_finite() {
        ui.label(tl!("Map extent is too small for the globe preview."));
        return false;
    }
    let available = ui.ctx().content_rect();
    let side = (available.height() - 180.0).min(available.width() - 80.0).clamp(180.0, DIAMETER as f32);
    let (rect, response) = ui.allocate_exact_size(egui::Vec2::splat(side), egui::Sense::drag());
    let mut camera = Camera::of(d);
    if response.dragged_by(egui::PointerButton::Primary) {
        let delta = ui.input(|i| i.pointer.delta());
        let sensitivity = 180.0 / (f64::from(side) * camera.zoom);
        camera.lon = longitude(camera.lon - f64::from(delta.x) * sensitivity);
        camera.lat = (camera.lat + f64::from(delta.y) * sensitivity).clamp(-90.0, 90.0);
    }
    if response.hovered() {
        let wheel = ui.input_mut(|i| {
            let y = i.smooth_scroll_delta.y;
            i.smooth_scroll_delta.y = 0.0;
            y
        });
        camera.zoom = (camera.zoom * (f64::from(wheel) * 0.005).exp()).clamp(0.65, 4.0);
    }
    d.fields.insert("longitude".into(), json!(camera.lon));
    d.fields.insert("latitude".into(), json!(camera.lat));
    d.fields.insert("zoom".into(), json!(camera.zoom));

    let id = egui::Id::unique("world-globe-preview");
    let cached = ui.ctx().data(|data| data.get_temp::<Preview>(id));
    let texture = if let Some(preview) = cached.as_ref().filter(|p| p.source == source && p.camera == camera) {
        preview.globe.id()
    } else {
        let map_pixels = cached.as_ref().filter(|p| p.source == source).map_or_else(
            || {
                let region = Rect::new(0.0, 0.0, width, height);
                let options = RenderOptions {
                    world_map_zoom: Some(source.zoom),
                    background: Some([255; 4]),
                    hide_templates: !source.templates,
                    hide_guides: true,
                    ..Default::default()
                };
                Arc::new(Renderer::new().render_region_with(&st.doc, region, scale, &options))
            },
            |p| p.map.clone(),
        );
        let image = project(&map_pixels, camera, DIAMETER);
        let globe = if let Some(preview) = cached {
            let mut texture = preview.globe;
            texture.set(image, egui::TextureOptions::LINEAR);
            texture
        } else {
            ui.ctx().load_texture("world-globe", image, egui::TextureOptions::LINEAR)
        };
        let texture = globe.id();
        ui.ctx().data_mut(|data| data.insert_temp(id, Preview { source, map: map_pixels, camera, globe }));
        texture
    };
    ui.painter().image(texture, rect, egui::Rect::from_min_max(egui::Pos2::ZERO, egui::pos2(1.0, 1.0)), Color32::WHITE);
    ui.label(tl!("Drag to rotate. Scroll to zoom."));
    false
}

fn center_map(app: &mut VectorcraftApp, d: &Dialog) -> Result<Value, String> {
    let st =
        app.session.active().filter(|st| Some(st.uid) == d.fields.get("document").and_then(Value::as_u64)).ok_or("The active document changed")?;
    let camera = Camera::of(d);
    let map = st.doc.world_map.ok_or("Requires a world-map document")?;
    let p = map.project(GeographicPoint { longitude_deg: camera.lon, latitude_deg: camera.lat }).map_err(|e| e.to_string())?;
    let view = app.view_mut().ok_or("No view")?;
    view.center = Point::new(p.x, p.y);
    view.fitted = true;
    app.ui.dialog = None;
    Ok(json!({"center":[p.x,p.y]}))
}

fn project(map: &Rendered, camera: Camera, side: usize) -> ColorImage {
    let (sin_lon, cos_lon) = camera.lon.to_radians().sin_cos();
    let (sin_lat, cos_lat) = camera.lat.to_radians().sin_cos();
    let radius = side as f64 * 0.48 * camera.zoom;
    let middle = side as f64 * 0.5;
    let mut pixels = Vec::with_capacity(side * side);
    for row in 0..side {
        let north = (middle - row as f64 - 0.5) / radius;
        for col in 0..side {
            let east = (col as f64 + 0.5 - middle) / radius;
            let front_squared = 1.0 - east * east - north * north;
            if front_squared < 0.0 {
                pixels.push(Color32::TRANSPARENT);
                continue;
            }
            let front = front_squared.sqrt();
            let along = front * cos_lat - north * sin_lat;
            let z = (front * sin_lat + north * cos_lat).clamp(-1.0, 1.0);
            let lon = (along * sin_lon + east * cos_lon).atan2(along * cos_lon - east * sin_lon);
            let u = lon / std::f64::consts::TAU + 0.5;
            let v = 0.5 - z.asin() / std::f64::consts::PI;
            pixels.push(sample(map, u, v));
        }
    }
    ColorImage::new([side, side], pixels)
}

/// Periodic longitude and clamped latitude. Interpolate in premultiplied RGBA.
fn sample(map: &Rendered, u: f64, v: f64) -> Color32 {
    let (w, h) = (i64::from(map.width), i64::from(map.height));
    if w == 0 || h == 0 {
        return Color32::TRANSPARENT;
    }
    let x = u * w as f64 - 0.5;
    let y = v * h as f64 - 0.5;
    let (left, top) = (x.floor() as i64, y.floor() as i64);
    let (fx, fy) = (x - x.floor(), y - y.floor());
    let read = |x: i64, y: i64| {
        let i = ((y.clamp(0, h - 1) * w + x.rem_euclid(w)) * 4) as usize;
        map.pixels.get(i..).and_then(|p| p.first_chunk::<4>()).copied().unwrap_or([0; 4])
    };
    let [a, b, c, d] = [read(left, top), read(left + 1, top), read(left, top + 1), read(left + 1, top + 1)];
    let mix = |a, b, c, d| {
        ((f64::from(a) * (1.0 - fx) + f64::from(b) * fx) * (1.0 - fy) + (f64::from(c) * (1.0 - fx) + f64::from(d) * fx) * fy).round() as u8
    };
    let [r, g, b, a] = std::array::from_fn(|i| mix(a[i], b[i], c[i], d[i]));
    Color32::from_rgba_premultiplied(r, g, b, a)
}

#[cfg(test)]
mod tests {
    use super::*;
    use vectorcraft_engine::Session;

    #[test]
    fn globe_projection_wraps_longitude_and_keeps_poles_upright() {
        let map = Rendered {
            width: 4,
            height: 2,
            pixels: [
                [255, 0, 0, 255],
                [0, 255, 0, 255],
                [0, 255, 0, 255],
                [255, 0, 0, 255],
                [0, 0, 255, 255],
                [0, 0, 255, 255],
                [0, 0, 255, 255],
                [0, 0, 255, 255],
            ]
            .into_iter()
            .flatten()
            .collect(),
        };
        let camera = Camera { lon: 0.0, lat: 90.0, zoom: 1.0 };
        let center = |camera| project(&map, camera, 9).pixels[40];
        assert_eq!(center(camera), Color32::GREEN);
        assert_eq!(center(Camera { lon: 180.0, ..camera }), Color32::RED);
        assert_eq!(center(Camera { lon: -180.0, ..camera }), Color32::RED);
        assert_eq!(center(Camera { lat: -90.0, ..camera }), Color32::BLUE);
        assert_eq!(project(&map, camera, 9).pixels[0], Color32::TRANSPARENT);
        assert_ne!(project(&map, Camera { zoom: 2.0, ..camera }, 9).pixels[0], Color32::TRANSPARENT);
    }

    #[test]
    fn world_globe_draws_and_centers_without_editing_the_document() {
        let mut app = VectorcraftApp::new(Session::new(), Default::default());
        assert!(app.run("ui.globe", json!({})).is_err());
        app.run("file.newWorldMap", json!({})).unwrap();
        let id = app.run("shape.rectangle", json!({"x":490,"y":240,"width":20,"height":20})).unwrap()["id"].clone();
        app.run("paint.setFill", json!({"id":id,"color":"#ff0000"})).unwrap();
        app.run("ui.globe", json!({})).unwrap();
        let ctx = egui::Context::default();
        crate::theme::install_fonts(&ctx);
        let mut output = ctx.run_ui(Default::default(), |ui| super::super::show(&mut app, ui.ctx()));
        output.textures_delta.clear();
        let cached = ctx.data(|data| data.get_temp::<Preview>(egui::Id::unique("world-globe-preview"))).unwrap();
        assert_eq!(cached.map.pixel(512, 256), [255, 0, 0, 255]);
        let d = app.ui.dialog.as_mut().unwrap();
        d.fields.insert("longitude".into(), json!(90.0));
        d.fields.insert("latitude".into(), json!(45.0));
        let mut output = ctx.run_ui(Default::default(), |ui| super::super::show(&mut app, ui.ctx()));
        output.textures_delta.clear();
        let next = ctx.data(|data| data.get_temp::<Preview>(egui::Id::unique("world-globe-preview"))).unwrap();
        assert!(Arc::ptr_eq(&cached.map, &next.map));
        app.run("map.setGeography", json!({"id":id,"geography":{"categories":["core:land"],"tags":["preview-test"]}})).unwrap();
        app.run("map.mode.setStyles", json!({"id":1,"rules":[{"category":"core:land","rgb":[0,255,0]}]})).unwrap();
        let mut output = ctx.run_ui(Default::default(), |ui| super::super::show(&mut app, ui.ctx()));
        output.textures_delta.clear();
        let styled = ctx.data(|data| data.get_temp::<Preview>(egui::Id::unique("world-globe-preview"))).unwrap();
        assert_eq!(styled.map.pixel(512, 256), [0, 255, 0, 255]);
        app.run("map.mode.setFilter", json!({"id":1,"rules":[{"kind":"tag","tag":"preview-test"}]})).unwrap();
        let mut output = ctx.run_ui(Default::default(), |ui| super::super::show(&mut app, ui.ctx()));
        output.textures_delta.clear();
        let filtered = ctx.data(|data| data.get_temp::<Preview>(egui::Id::unique("world-globe-preview"))).unwrap();
        assert!(!Arc::ptr_eq(&styled.map, &filtered.map));
        assert_eq!(filtered.map.pixel(512, 256), [255, 255, 255, 255]);
        let doc = app.session.doc().unwrap().doc.clone();
        let history = app.session.doc().unwrap().history.undo.len();
        super::super::confirm(&mut app).unwrap();
        assert_eq!(app.view().unwrap().center, Point::new(750.0, 125.0));
        assert_eq!(app.session.doc().unwrap().doc, doc);
        assert_eq!(app.session.doc().unwrap().history.undo.len(), history);
    }
}
