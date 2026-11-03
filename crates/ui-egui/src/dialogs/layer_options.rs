//! Layer Options (double-click a layer row, or ≡ › Options for Selection…) and the same dialog for
//! a new layer or sublayer (≡ › New Layer…, Alt-click Create New Layer). Objects' rows get Name,
//! Show and Lock only.
//!
//! Fields: `ids` (the rows edited; none for a new layer), `mode` (`edit`, `new` or `newSublayer`),
//! `layer` (the rows are layers), `name`, `color` (a preset's name such as "Light Blue", or
//! `#rrggbb`), `template`, `locked`, `visible`, `printable`, `preview`, `dimImages` and
//! `dimPercent`. OK runs `layer.setProps`, `layer.new` or `layer.newSublayer`.

use serde_json::{Value, json};
use vectorcraft_doc::{LAYER_COLORS, LayerColor, Node, NodeId, NodeKind, ZoomFade};

use super::swatch_options::{grid, label};
use super::{DialogSpec, form, run_and_close};
use crate::state::Dialog;
use crate::{VectorcraftApp, widgets};

/// The dialog kind of Layer Options.
pub const KIND: &str = "layerOptions";

pub(super) const SPEC: DialogSpec = DialogSpec { heading, body, confirm, min_width: 320.0, ..DialogSpec::FORM };

fn heading(d: &Dialog) -> String {
    if d.bool("layer") || d.str("mode") != "edit" { tl!("Layer Options").into() } else { tl!("Options").into() }
}

/// A layer colour as the dialog shows it: a preset's name, else `#rrggbb`.
fn color_field(c: LayerColor) -> String {
    match c {
        LayerColor::Preset(i) => LAYER_COLORS.get(i as usize).map_or("Light Blue", |(n, _)| *n).to_string(),
        LayerColor::Custom([r, g, b]) => format!("#{r:02x}{g:02x}{b:02x}"),
    }
}

/// The fields for `n` (a layer or any other row).
/// `xml_id`: Preferences › Units › Identify Objects By › XML ID.
fn fields_of(n: &Node, xml_id: bool) -> serde_json::Map<String, Value> {
    let mut f = serde_json::Map::new();
    f.insert("name".into(), json!(n.identify_label(xml_id)));
    f.insert("visible".into(), json!(n.visible));
    f.insert("locked".into(), json!(n.locked));
    f.insert("layer".into(), json!(n.is_layer()));
    if let NodeKind::Layer { color, template, printable, preview, dim_images, .. } = &n.kind {
        f.insert("color".into(), json!(color_field(*color)));
        f.insert("template".into(), json!(template));
        f.insert("printable".into(), json!(printable));
        f.insert("preview".into(), json!(preview));
        f.insert("dimImages".into(), json!(dim_images.is_some()));
        f.insert("dimPercent".into(), json!(dim_images.unwrap_or(50)));
    }
    f
}

/// Add optional map-specific controls to the existing Layer Options dialog.
fn zoom_fields(fields: &mut serde_json::Map<String, Value>, fade: Option<ZoomFade>) {
    fields.insert("worldMap".into(), json!(true));
    fields.insert("zoomFadeEnabled".into(), json!(fade.is_some()));
    fields.insert("fadeInEnabled".into(), json!(fade.is_some_and(|f| f.fade_in.is_some())));
    fields.insert("fadeOutEnabled".into(), json!(fade.is_some_and(|f| f.fade_out.is_some())));
    let inside = fade.and_then(|f| f.fade_in).unwrap_or([200.0, 500.0]);
    let outside = fade.and_then(|f| f.fade_out).unwrap_or([2000.0, 5000.0]);
    for (key, value) in [("fadeInStart", inside[0]), ("fadeInEnd", inside[1]), ("fadeOutStart", outside[0]), ("fadeOutEnd", outside[1])] {
        fields.insert(key.into(), json!(value));
    }
}

/// `ui.layerOptions {ids?|id?}`: open Layer Options for rows (default: the highlighted rows, else
/// the current layer), filled in from the first.
pub fn open(app: &mut VectorcraftApp, p: &Value) -> Result<Value, String> {
    let st = app.session.active().ok_or("no document open")?;
    // Rows given are used as given: a malformed or unknown id fails rather than being skipped or
    // the dialog opening on the highlighted rows instead.
    let row = |v: &Value| -> Result<NodeId, String> {
        let id = NodeId(v.as_u64().ok_or_else(|| format!("object ids are non-negative integers, not {v}"))?);
        st.doc.node(id).map(|_| id).ok_or_else(|| "no such layer or object".to_string())
    };
    let ids: Vec<NodeId> = match (p.get("ids"), p.get("id")) {
        (Some(Value::Array(a)), _) => a.iter().map(row).collect::<Result<_, _>>()?,
        (Some(v), _) => return Err(format!("`ids` must be an array of object ids, not {v}")),
        (None, Some(v)) => vec![row(v)?],
        (None, None) => {
            let rows = st.highlighted_rows();
            if rows.is_empty() { st.current_layer().into_iter().collect() } else { rows }
        }
    };
    let first = ids.first().and_then(|id| st.doc.node(*id)).ok_or("no such layer or object")?;
    let xml_id = app.session.prefs.identify_objects_by == "xmlId";
    let mut f = fields_of(first, xml_id);
    if first.is_layer() && st.doc.world_map.is_some() {
        zoom_fields(&mut f, first.extras().zoom_fade);
    }
    f.insert("ids".into(), json!(ids.iter().map(|i| i.0).collect::<Vec<_>>()));
    f.insert("mode".into(), json!("edit"));
    app.ui.dialog = Some(Dialog { kind: KIND.into(), fields: f });
    Ok(Value::Null)
}

/// `ui.newLayer {sublayer?}`: Layer Options for a new layer (or sublayer), with the name and
/// colour it would get.
pub fn open_new(app: &mut VectorcraftApp, p: &Value) -> Result<Value, String> {
    let st = app.session.active().ok_or("no document open")?;
    let mut layers = vec![];
    st.doc.walk(|n| {
        if n.is_layer() {
            layers.push(n.display_name());
        }
    });
    let mut i = layers.len() + 1;
    while layers.contains(&format!("Layer {i}")) {
        i += 1;
    }
    let sublayer = p.get("sublayer").and_then(Value::as_bool).unwrap_or(false);
    let color = LayerColor::Preset((layers.len() % LAYER_COLORS.len()) as u8);
    let name = format!("Layer {i}");
    let name = if app.session.prefs.identify_objects_by == "xmlId" { vectorcraft_doc::xml_id::from_object_name(&name) } else { name };
    let mut fields = json!({
        "mode": if sublayer { "newSublayer" } else { "new" }, "layer": true, "name": name, "color": color_field(color),
        "template": false, "locked": false, "visible": true, "printable": true, "preview": true, "dimImages": false, "dimPercent": 50,
    });
    if st.doc.world_map.is_some()
        && let Some(fields) = fields.as_object_mut()
    {
        zoom_fields(fields, None);
    }
    app.ui.dialog = Some(Dialog::new(KIND, fields));
    Ok(Value::Null)
}

fn toggle(ui: &mut egui::Ui, d: &mut Dialog, key: &str, text: &str, enabled: bool) -> bool {
    let v = d.bool(key);
    let changed = widgets::check(ui, text, v, enabled);
    if changed {
        d.fields.insert(key.into(), json!(!v));
    }
    changed
}

fn fade_range(ui: &mut egui::Ui, d: &mut Dialog, flag: &str, start: &str, end: &str, name: &str) {
    toggle(ui, d, flag, name, true);
    if d.bool(flag) {
        ui.horizontal(|ui| {
            ui.label("From");
            let lo = d.f64(start, 200.0);
            if let Some(v) = widgets::range_field(ui, start, lo, 0.0313..=1_000_000.0, "×", 2, 92.0) {
                d.fields.insert(start.into(), json!(v));
            }
            ui.label("to");
            let hi = d.f64(end, 500.0);
            if let Some(v) = widgets::range_field(ui, end, hi, 0.0313..=1_000_000.0, "×", 2, 92.0) {
                d.fields.insert(end.into(), json!(v));
            }
        });
    }
}

/// The size of the colour chip before each name in the Color list.
const CHIP: egui::Vec2 = egui::vec2(16.0, 10.0);

/// The Color list: each preset's name after a chip of its colour, then Custom (no chip, its
/// name aligned with the others). Returns the chosen row; Custom is `LAYER_COLORS.len()`.
fn color_dropdown(ui: &mut egui::Ui, shown: &str) -> Option<usize> {
    let border = crate::theme::Tokens::get(ui.ctx()).input_border;
    let rows = LAYER_COLORS.iter().map(|(n, c)| (*n, Some(*c))).chain([("Custom", None)]);
    widgets::combo(ui, "layer-color", tl!(shown), 140.0, |ui| {
        let mut chosen = None;
        for (i, (name, rgb)) in rows.enumerate() {
            let chip = egui::IdSalt::new(("layer-color-chip", i));
            let r = egui::Button::selectable(name == shown, (egui::Atom::custom(chip, CHIP), tl!(name))).atom_ui(ui);
            if let (Some(rect), Some([cr, cg, cb])) = (r.rect(chip), rgb) {
                ui.painter().rect_filled(rect, 1.0, egui::Color32::from_rgb(cr, cg, cb));
                ui.painter().rect_stroke(rect, 1.0, egui::Stroke::new(1.0, border), egui::StrokeKind::Inside);
            }
            if r.response.clicked() {
                chosen = Some(i);
            }
        }
        chosen
    })
}

fn body(_: &mut VectorcraftApp, ui: &mut egui::Ui, d: &mut Dialog) -> bool {
    let layer = d.bool("layer");
    let many = d.fields.get("ids").and_then(Value::as_array).is_some_and(|a| a.len() > 1);
    grid(ui, |ui| {
        if !many {
            label(ui, tl!("Name:"));
            form::text(ui, d, "name", 190.0);
            ui.end_row();
        }
        if layer {
            label(ui, tl!("Color:"));
            ui.horizontal(|ui| {
                let cur = d.str("color");
                let shown = LAYER_COLORS.iter().map(|(n, _)| *n).find(|n| n.eq_ignore_ascii_case(&cur)).unwrap_or("Custom");
                if let Some((n, _)) = color_dropdown(ui, shown).and_then(|i| LAYER_COLORS.get(i)) {
                    d.fields.insert("color".into(), json!(n));
                }
                let [r, g, b] = LAYER_COLORS
                    .iter()
                    .find(|(n, _)| n.eq_ignore_ascii_case(&cur))
                    .map(|(_, c)| *c)
                    .or_else(|| {
                        vectorcraft_color::Color::from_hex(&cur).map(|c| {
                            let [r, g, b, _] = c.to_rgba8(1.0);
                            [r, g, b]
                        })
                    })
                    .unwrap_or(LAYER_COLORS[0].1);
                let mut rgb = [r, g, b];
                if ui.color_edit_button_srgb(&mut rgb).changed() {
                    d.fields.insert("color".into(), json!(format!("#{:02x}{:02x}{:02x}", rgb[0], rgb[1], rgb[2])));
                }
            });
            ui.end_row();
        }
    });
    ui.add_space(8.0);
    if layer {
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                // A template is locked and dims its images; turning it off frees both.
                if toggle(ui, d, "template", tl!("Template"), true) {
                    let on = d.bool("template");
                    d.fields.insert("locked".into(), json!(on));
                    d.fields.insert("dimImages".into(), json!(on));
                }
                toggle(ui, d, "visible", tl!("Show"), true);
                toggle(ui, d, "preview", tl!("Preview"), true);
                ui.horizontal(|ui| {
                    toggle(ui, d, "dimImages", tl!("Dim Images to:"), true);
                    let v = d.f64("dimPercent", 50.0);
                    let dim = d.bool("dimImages");
                    if let Some(v) = ui.add_enabled_ui(dim, |ui| crate::widgets::range_field(ui, "dimPercent", v, 0.0..=100.0, "%", 0, 52.0)).inner {
                        d.fields.insert("dimPercent".into(), json!(v));
                    }
                });
            });
            ui.add_space(24.0);
            ui.vertical(|ui| {
                let template = d.bool("template");
                toggle(ui, d, "locked", tl!("Lock"), !template);
                toggle(ui, d, "printable", tl!("Print"), !template);
            });
        });
    } else {
        toggle(ui, d, "visible", tl!("Show"), true);
        toggle(ui, d, "locked", tl!("Lock"), true);
    }
    if layer && d.bool("worldMap") {
        ui.separator();
        ui.label("Aetherys – Zoom visibility");
        toggle(ui, d, "zoomFadeEnabled", "Fade layer by zoom", true);
        if d.bool("zoomFadeEnabled") {
            ui.indent("map-zoom-fade", |ui| {
                fade_range(ui, d, "fadeInEnabled", "fadeInStart", "fadeInEnd", "Fade in");
                fade_range(ui, d, "fadeOutEnabled", "fadeOutStart", "fadeOutEnd", "Fade out");
            });
        }
    }
    false
}

/// The command and params OK runs.
pub(crate) fn params(d: &Dialog) -> (&'static str, Value) {
    let mut p = serde_json::Map::new();
    let many = d.fields.get("ids").and_then(Value::as_array).is_some_and(|a| a.len() > 1);
    if !many {
        p.insert("name".into(), json!(d.str("name")));
    }
    p.insert("visible".into(), json!(d.bool("visible")));
    p.insert("locked".into(), json!(d.bool("locked")));
    if d.bool("layer") {
        p.insert("color".into(), json!(d.str("color")));
        p.insert("template".into(), json!(d.bool("template")));
        p.insert("printable".into(), json!(d.bool("printable")));
        p.insert("preview".into(), json!(d.bool("preview")));
        p.insert("dimImages".into(), if d.bool("dimImages") { json!(d.f64("dimPercent", 50.0).clamp(0.0, 100.0)) } else { json!(false) });
        if d.bool("worldMap") {
            let range = |flag: &str, start: &str, end: &str| d.bool(flag).then(|| [d.f64(start, 200.0), d.f64(end, 500.0)]);
            let fade = d.bool("zoomFadeEnabled").then(|| {
                json!({
                    "fade_in": range("fadeInEnabled", "fadeInStart", "fadeInEnd"),
                    "fade_out": range("fadeOutEnabled", "fadeOutStart", "fadeOutEnd")
                })
            });
            p.insert("zoomFade".into(), fade.unwrap_or(Value::Null));
        }
    }
    match d.str("mode").as_str() {
        "new" => ("layer.new", Value::Object(p)),
        "newSublayer" => ("layer.newSublayer", Value::Object(p)),
        _ => {
            p.insert("ids".into(), d.fields.get("ids").cloned().unwrap_or(json!([])));
            ("layer.setProps", Value::Object(p))
        }
    }
}

fn confirm(app: &mut VectorcraftApp, d: &Dialog) -> Result<Value, String> {
    let (cmd, p) = params(d);
    run_and_close(app, cmd, p)
}

#[cfg(test)]
mod tests {
    use super::*;
    use vectorcraft_engine::Session;

    fn app() -> VectorcraftApp {
        let mut app = VectorcraftApp::new(Session::new(), Default::default());
        app.session.execute("file.new", &json!({"width": 100, "height": 100})).unwrap();
        app
    }

    fn draw(app: &mut VectorcraftApp) {
        let ctx = egui::Context::default();
        crate::theme::install_fonts(&ctx);
        let mut out = ctx.run_ui(Default::default(), |ui| super::super::show(app, ui.ctx()));
        out.textures_delta.clear();
    }

    fn set(app: &mut VectorcraftApp, k: &str, v: Value) {
        app.ui.dialog.as_mut().unwrap().fields.insert(k.into(), v);
    }

    /// Explicit object ids must not silently disappear or fall back to the highlighted row.
    #[test]
    fn invalid_layer_options_targets_do_not_open_a_partial_dialog() {
        let mut app = app();
        let id = app.run("shape.rectangle", json!({"x": 0, "y": 0, "width": 20, "height": 20})).unwrap()["id"].as_u64().unwrap();
        for params in
            [json!({"ids": [id, "invalid"]}), json!({"ids": [id, 999999]}), json!({"ids": "invalid"}), json!({"id": "invalid"}), json!({"id": null})]
        {
            assert!(app.run("ui.layerOptions", params.clone()).is_err(), "accepted {params}");
            assert!(app.ui.dialog.is_none(), "opened partial options for {params}");
        }
        app.run("ui.layerOptions", json!({"ids": [id]})).unwrap();
        assert_eq!(app.ui.dialog.as_ref().unwrap().fields["ids"], json!([id]));
    }

    #[test]
    fn layer_options_edit_the_layer_and_ok_is_one_undo_step() {
        let mut app = app();
        let layer = app.session.doc().unwrap().doc.layers[0].id;
        app.run("ui.layerOptions", json!({})).unwrap();
        let d = app.ui.dialog.clone().unwrap();
        assert_eq!((d.kind.as_str(), d.str("name"), d.str("color")), (KIND, "Layer 1".to_string(), "Light Blue".to_string()));
        draw(&mut app);
        set(&mut app, "name", json!("Sky"));
        set(&mut app, "color", json!("#00ff00"));
        set(&mut app, "dimImages", json!(true));
        set(&mut app, "dimPercent", json!(30));
        set(&mut app, "preview", json!(false));
        let undo = app.session.doc().unwrap().history.undo.len();
        super::super::confirm(&mut app).unwrap();
        assert!(app.ui.dialog.is_none());
        let n = app.session.doc().unwrap().doc.node(layer).unwrap().clone();
        assert_eq!(n.display_name(), "Sky");
        assert!(matches!(n.kind, NodeKind::Layer { color: LayerColor::Custom([0, 255, 0]), dim_images: Some(30), preview: false, .. }));
        assert_eq!(app.session.doc().unwrap().history.undo.len(), undo + 1);
    }

    #[test]
    fn world_map_layer_dialog_edits_zoom_fade_but_ordinary_dialog_does_not() {
        let mut app = VectorcraftApp::new(Session::new(), Default::default());
        app.run("file.newWorldMap", json!({})).unwrap();
        let id = app.session.doc().unwrap().doc.layers[0].id;
        app.run("ui.layerOptions", json!({"id":id.0})).unwrap();
        assert!(app.ui.dialog.as_ref().unwrap().bool("worldMap"));
        set(&mut app, "zoomFadeEnabled", json!(true));
        set(&mut app, "fadeInEnabled", json!(false));
        set(&mut app, "fadeOutEnabled", json!(true));
        set(&mut app, "fadeOutStart", json!(200.0));
        set(&mut app, "fadeOutEnd", json!(500.0));
        let (_, p) = params(app.ui.dialog.as_ref().unwrap());
        assert_eq!(p["zoomFade"], json!({"fade_in":null,"fade_out":[200.0,500.0]}));
        super::super::confirm(&mut app).unwrap();
        let fade = app.session.doc().unwrap().doc.node(id).unwrap().extras().zoom_fade.unwrap();
        assert_eq!(fade.opacity(200.0), 1.0);
        assert_eq!(fade.opacity(500.0), 0.0);
        app.run("ui.layerOptions", json!({"id":id.0})).unwrap();
        assert!(app.ui.dialog.as_ref().unwrap().bool("zoomFadeEnabled"));
        assert!(app.ui.dialog.as_ref().unwrap().bool("fadeOutEnabled"));
        app.ui.dialog = None;

        let mut ordinary = VectorcraftApp::new(Session::new(), Default::default());
        ordinary.run("file.new", json!({})).unwrap();
        ordinary.run("ui.layerOptions", json!({})).unwrap();
        let d = ordinary.ui.dialog.as_ref().unwrap();
        assert!(!d.bool("worldMap"));
        assert!(params(d).1.get("zoomFade").is_none());
    }

    #[test]
    fn new_layer_options_make_a_layer_and_objects_get_name_show_and_lock() {
        let mut app = app();
        app.run("ui.newLayer", json!({})).unwrap();
        assert_eq!(app.ui.dialog.as_ref().unwrap().str("name"), "Layer 2");
        set(&mut app, "name", json!("Ink"));
        set(&mut app, "template", json!(true));
        set(&mut app, "locked", json!(true));
        draw(&mut app);
        super::super::confirm(&mut app).unwrap();
        let d = &app.session.doc().unwrap().doc;
        let top = d.layers.last().unwrap();
        assert_eq!(top.display_name(), "Ink");
        assert!(top.is_template() && top.locked);
        // An object's row: Name, Show and Lock.
        let r = app.session.execute("shape.rectangle", &json!({"x": 0, "y": 0, "width": 10, "height": 10}));
        assert!(r.is_ok());
        app.session.execute("layer.setCurrent", &json!({"id": app.session.doc().unwrap().doc.layers[0].id.0})).unwrap();
        let rect = app.session.execute("shape.rectangle", &json!({"x": 0, "y": 0, "width": 10, "height": 10})).unwrap()["id"].as_u64().unwrap();
        app.run("ui.layerOptions", json!({"id": rect})).unwrap();
        assert!(!app.ui.dialog.as_ref().unwrap().bool("layer"));
        draw(&mut app);
        set(&mut app, "name", json!("Box"));
        set(&mut app, "locked", json!(true));
        let (cmd, p) = params(app.ui.dialog.as_ref().unwrap());
        assert_eq!(cmd, "layer.setProps");
        assert!(p.get("template").is_none());
        super::super::confirm(&mut app).unwrap();
        let n = app.session.doc().unwrap().doc.node(NodeId(rect)).unwrap().clone();
        assert_eq!((n.display_name(), n.locked), ("Box".to_string(), true));
    }
}
