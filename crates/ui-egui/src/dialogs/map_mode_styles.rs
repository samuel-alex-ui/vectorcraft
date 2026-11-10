use serde::Deserialize;
use serde_json::{Value, json};
use vectorcraft_engine::doc::geography::{BUILTIN_CATEGORIES, MapStyleRule, MapStyles};

use super::{DialogSpec, map_mode_filters::category_name};
use crate::{VectorcraftApp, state::Dialog};

pub const KIND: &str = "mapModeStyles";
pub(super) const SPEC: DialogSpec = DialogSpec { heading: |_| tl!("Map Mode Styles").into(), body, confirm, min_width: 440.0, ..DialogSpec::FORM };

pub fn open(app: &mut VectorcraftApp) -> Result<Value, String> {
    let st = app.session.active().filter(|st| st.doc.world_map.is_some()).ok_or("Requires a world-map document")?;
    let modes = st.doc.map_semantics.as_ref().and_then(|s| s.modes.as_ref());
    let id = modes.map_or(1, |m| m.active);
    let rules = modes.and_then(|m| m.styles.get(&id)).cloned().unwrap_or_default();
    app.ui.dialog = Some(Dialog::new(KIND, json!({"id":id,"document":st.uid,"rules":rules,"category":"core:land","rgb":[85,145,85]})));
    Ok(Value::Null)
}

fn body(app: &mut VectorcraftApp, ui: &mut egui::Ui, d: &mut Dialog) -> bool {
    let Some(st) = app.session.active().filter(|st| Some(st.uid) == d.fields.get("document").and_then(Value::as_u64)) else { return true };
    let Some(mut styles) = d.fields.get("rules").and_then(|v| MapStyles::deserialize(v).ok()) else {
        ui.label(tl!("Invalid style rules"));
        return false;
    };
    ui.label(tl!("The first matching category sets the fill color."));
    let mut remove = None;
    let mut swap = None;
    let len = styles.0.len();
    egui::ScrollArea::vertical().id_salt("map-style-order").max_height(240.0).show(ui, |ui| {
        for (i, rule) in styles.0.iter_mut().enumerate() {
            ui.push_id(&rule.category, |ui| {
                ui.horizontal(|ui| {
                    ui.label(format!("{}.", i + 1));
                    let name = category_name(&st.doc, &rule.category);
                    ui.name_inputs(name.as_str(), |ui| ui.color_edit_button_srgb(&mut rule.rgb));
                    ui.label(name);
                    if ui.add_enabled(i > 0, egui::Button::new(tl!("Up"))).clicked() {
                        swap = Some((i, i.saturating_sub(1)));
                    }
                    if ui.add_enabled(i + 1 < len, egui::Button::new(tl!("Down"))).clicked() {
                        swap = Some((i, i + 1));
                    }
                    if ui.button(tl!("Remove")).clicked() {
                        remove = Some(i);
                    }
                });
            });
        }
    });
    if let Some(i) = remove {
        styles.0.remove(i);
    } else if let Some((a, b)) = swap {
        styles.0.swap(a, b);
    }

    let mut category = d.str("category");
    let mut rgb = d.fields.get("rgb").and_then(|v| <[u8; 3]>::deserialize(v).ok()).unwrap_or([85, 145, 85]);
    ui.horizontal_wrapped(|ui| {
        let chosen = crate::widgets::combo(ui, "map-style-category", &category_name(&st.doc, &category), 150.0, |ui| {
            egui::ScrollArea::vertical()
                .max_height(220.0)
                .show(ui, |ui| {
                    let mut choice = None;
                    let custom = st.doc.map_semantics.iter().flat_map(|s| s.categories.keys().map(String::as_str));
                    for id in BUILTIN_CATEGORIES.iter().copied().chain(custom) {
                        if ui.selectable_label(category == id, category_name(&st.doc, id)).clicked() {
                            choice = Some(id.to_owned());
                        }
                    }
                    choice
                })
                .inner
        });
        if let Some(chosen) = chosen {
            category = chosen;
        }
        ui.name_inputs(tl!("Fill Color"), |ui| ui.color_edit_button_srgb(&mut rgb));
        let can_add = styles.0.len() < MapStyles::MAX_RULES && !styles.0.iter().any(|r| r.category == category);
        if ui.add_enabled(can_add, egui::Button::new(tl!("Add"))).clicked() {
            styles.0.push(MapStyleRule { category: category.clone(), rgb });
        }
    });
    d.fields.insert("category".into(), json!(category));
    d.fields.insert("rgb".into(), json!(rgb));
    d.fields.insert("rules".into(), json!(styles));
    false
}

fn confirm(app: &mut VectorcraftApp, d: &Dialog) -> Result<Value, String> {
    if app.session.active().map(|st| st.uid) != d.fields.get("document").and_then(Value::as_u64) {
        return Err("The active document changed".into());
    }
    let result = app.run("map.mode.setStyles", json!({"id":d.fields.get("id"),"rules":d.fields.get("rules")}))?;
    app.ui.dialog = None;
    Ok(result)
}
