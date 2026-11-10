//! World-map mode selection and configuration. Changes go through engine commands.
use serde_json::{Value, json};
use vectorcraft_engine::doc::geography::MapModes;

use super::{DialogSpec, form};
use crate::{VectorcraftApp, state::Dialog};

pub const KIND: &str = "mapModes";
pub(super) const SPEC: DialogSpec =
    DialogSpec { heading: |_| tl!("Configure Map Modes").into(), body, confirm, min_width: 340.0, ..DialogSpec::FORM };

fn catalogue(app: &VectorcraftApp) -> Option<MapModes> {
    let doc = &app.session.active()?.doc;
    doc.world_map?;
    Some(doc.map_semantics.as_ref().and_then(|s| s.modes.clone()).unwrap_or_default())
}

fn label(id: u32, name: &str) -> &str {
    if MapModes::STARTER_NAMES.get(id.saturating_sub(1) as usize).is_some_and(|starter| *starter == name) { crate::i18n::t(name) } else { name }
}

fn picker(ui: &mut egui::Ui, salt: &str, modes: &MapModes, id: u32) -> u32 {
    let selected = modes.names.get(&id).map_or("", |name| label(id, name));
    crate::widgets::combo_named(ui, salt, Some(tl!("Map Mode")), selected, 180.0, |ui| {
        egui::ScrollArea::vertical()
            .max_height(280.0)
            .show(ui, |ui| {
                let mut choice = None;
                for (&key, name) in &modes.names {
                    if ui.selectable_label(key == id, label(key, name)).clicked() {
                        choice = Some(key);
                    }
                }
                choice
            })
            .inner
    })
    .unwrap_or(id)
}

pub(crate) fn bar(app: &mut VectorcraftApp, ui: &mut egui::Ui) {
    let Some(modes) = catalogue(app) else { return };
    ui.horizontal_wrapped(|ui| {
        ui.label(tl!("Map Mode"));
        let id = picker(ui, "active-map-mode", &modes, modes.active);
        if id != modes.active {
            let _ = app.run("map.mode.select", json!({"id": id}));
        }
        if ui.button(tl!("Configure Map Modes")).clicked() {
            let _ = app.run("ui.mapModes", json!({}));
        }
    });
}

pub fn open(app: &mut VectorcraftApp) -> Result<Value, String> {
    let modes = catalogue(app).ok_or("Requires a world-map document")?;
    let uid = app.session.active().ok_or("No document")?.uid;
    let name = modes.names.get(&modes.active).map_or("", |name| label(modes.active, name));
    app.ui.dialog = Some(Dialog::new(KIND, json!({"action":"rename", "id":modes.active, "name":name, "document":uid})));
    Ok(Value::Null)
}

fn body(app: &mut VectorcraftApp, ui: &mut egui::Ui, d: &mut Dialog) -> bool {
    let Some(modes) = catalogue(app) else { return false };
    let mut action = d.str("action");
    ui.horizontal_wrapped(|ui| {
        for (key, text) in [("create", tl!("New")), ("rename", tl!("Rename")), ("duplicate", tl!("Duplicate")), ("delete", tl!("Delete"))] {
            ui.selectable_value(&mut action, key.into(), text);
        }
    });
    let original = d.fields.get("id").and_then(Value::as_u64).and_then(|id| u32::try_from(id).ok()).unwrap_or(modes.active);
    let id = if action == "create" { original } else { picker(ui, "configure-map-mode", &modes, original) };
    if action != d.str("action") || id != original {
        let name = if action == "create" {
            String::new()
        } else {
            let current = modes.names.get(&id).map_or("", |name| label(id, name));
            if action == "duplicate" { format!("{current} {}", tl!("Copy")) } else { current.to_owned() }
        };
        d.fields.insert("name".into(), json!(name));
    }
    d.fields.insert("action".into(), json!(action));
    d.fields.insert("id".into(), json!(id));
    if action != "delete" {
        ui.label(tl!("Name:"));
        form::text(ui, d, "name", 280.0);
    }
    false
}

fn confirm(app: &mut VectorcraftApp, d: &Dialog) -> Result<Value, String> {
    if app.session.active().map(|st| st.uid) != d.fields.get("document").and_then(Value::as_u64) {
        return Err("The active document changed".into());
    }
    let action = d.str("action");
    let id = d.fields.get("id");
    let draft = d.str("name");
    let modes = catalogue(app).ok_or("Requires a world-map document")?;
    // Confirming a translated starter label without editing it keeps the original name.
    let name = id
        .and_then(Value::as_u64)
        .and_then(|id| u32::try_from(id).ok())
        .and_then(|id| modes.names.get(&id).filter(|name| action == "rename" && label(id, name) == draft))
        .map_or(draft.as_str(), String::as_str);
    app.run("map.mode.edit", json!({"action": action, "id": id, "name": name}))
}
