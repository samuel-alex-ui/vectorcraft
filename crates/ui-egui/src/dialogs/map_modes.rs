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
        if ui.button(tl!("Map Mode Styles")).clicked() {
            let _ = app.run("ui.mapModeStyles", json!({}));
        }
        if ui.button(tl!("Map Mode Filters")).clicked() {
            let _ = app.run("ui.mapModeFilters", json!({}));
        }
        if ui.button(tl!("Configure Map Modes")).clicked() {
            let _ = app.run("ui.mapModes", json!({}));
        }
        if ui.button(tl!("World Map Settings…")).clicked() {
            let _ = app.run("ui.worldMapSettings", json!({}));
        }
        let mut wrap = app.session.active().and_then(|st| st.doc.world_map).is_some_and(|map| map.longitude_wrap());
        if ui.checkbox(&mut wrap, tl!("Wrap Map Longitude")).changed() {
            let _ = app.run("map.setLongitudeWrap", json!({"enabled": wrap}));
        }
        let editing = app.view().is_some_and(|v| v.edit_hidden_map_art);
        if ui.selectable_label(editing, tl!("Edit Hidden Map Art")).clicked() {
            let _ = app.run("ui.editHiddenMapArt", json!({}));
        }
    });
}

pub fn toggle_edit_hidden(app: &mut VectorcraftApp) -> Result<Value, String> {
    let view = app.view_mut().filter(|v| v.world_map).ok_or("Requires a world-map document")?;
    view.edit_hidden_map_art = !view.edit_hidden_map_art;
    Ok(json!(view.edit_hidden_map_art))
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
    let result = app.run("map.mode.edit", json!({"action": action, "id": id, "name": name}));
    if result.is_ok() {
        app.ui.dialog = None;
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use vectorcraft_engine::{
        Session,
        tools::{PointerEvent, PointerKind},
    };

    #[test]
    fn temporary_map_editing_respects_layer_locks_and_keeps_saved_rules() {
        let mut app = VectorcraftApp::new(Session::new(), Default::default());
        let run = |app: &mut VectorcraftApp, id, params| app.run(id, params).unwrap();
        run(&mut app, "file.new", json!({}));
        assert!(app.run("ui.editHiddenMapArt", json!({})).is_err());
        run(&mut app, "file.newWorldMap", json!({}));
        let layer = run(&mut app, "map.detailLevel.create", json!({}))["id"].clone();
        run(&mut app, "layer.setCurrent", json!({"id":layer}));
        let id = run(&mut app, "shape.rectangle", json!({"x":10,"y":10,"width":20,"height":20}))["id"].clone();
        run(&mut app, "paint.setFill", json!({"id":id,"color":"#ff0000"}));
        run(&mut app, "map.setGeography", json!({"id":id,"geography":{"categories":["core:river"]}}));
        run(&mut app, "map.mode.setFilter", json!({"id":1,"rules":[{"kind":"category","id":"core:river"}]}));
        run(&mut app, "layer.setProps", json!({"id":layer,"zoomFade":{"fade_in":[2,4]}}));
        app.view_mut().unwrap().zoom = 1.0;
        let click = |app: &mut VectorcraftApp| {
            run(app, "select.none", json!({}));
            let view = app.view_info();
            for kind in [PointerKind::Down, PointerKind::Up] {
                app.session.pointer(&PointerEvent::new(kind, 20.0, 20.0), view).unwrap();
            }
            !app.session.active().unwrap().selection.objects.is_empty()
        };
        assert!(!click(&mut app));
        let before = app.session.active().unwrap().doc.clone();
        assert_eq!(run(&mut app, "ui.editHiddenMapArt", json!({})), json!(true));
        assert!(click(&mut app));
        assert_eq!(app.session.active().unwrap().doc, before);
        run(&mut app, "object.move", json!({"dx":40,"dy":0}));
        let moved = app.session.active().unwrap().doc.clone();
        assert_ne!(moved, before);
        run(&mut app, "edit.undo", json!({}));
        assert_eq!(app.session.active().unwrap().doc, before);
        run(&mut app, "edit.redo", json!({}));
        assert_eq!(app.session.active().unwrap().doc, moved);
        run(&mut app, "edit.undo", json!({}));

        run(&mut app, "layer.setProps", json!({"id":layer,"locked":true}));
        assert!(!click(&mut app));
        run(&mut app, "layer.setProps", json!({"id":layer,"locked":false,"visible":false}));
        assert!(!click(&mut app));
        run(&mut app, "layer.setProps", json!({"id":layer,"visible":true}));
        assert!(click(&mut app));
        assert_eq!(run(&mut app, "ui.editHiddenMapArt", json!({})), json!(false));
        assert!(!click(&mut app));

        let saved = vectorcraft_format::save_with(&before, &Default::default()).unwrap();
        let loaded = vectorcraft_format::load(&saved).unwrap();
        assert_eq!(loaded.map_semantics, before.map_semantics);
        assert_eq!(loaded.layers, before.layers);
        assert!(!crate::state::View::of(app.session.active().unwrap()).edit_hidden_map_art);
    }
}
