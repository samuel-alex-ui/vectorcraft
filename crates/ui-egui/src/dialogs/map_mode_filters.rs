//! Edit exclusions for one world-map mode. The draft lives in the shared modal.
use serde_json::{Value, json};
use vectorcraft_engine::doc::{
    Document, NodeId,
    geography::{BUILTIN_CATEGORIES, MapFilter, MapFilterRule, MapModes},
};

use super::{DialogSpec, form};
use crate::{VectorcraftApp, state::Dialog};

pub const KIND: &str = "mapModeFilters";
pub(super) const SPEC: DialogSpec =
    DialogSpec { heading: |_| tl!("Map Mode Filters").into(), body, confirm, min_width: 460.0, max_width: Some(560.0), ..DialogSpec::FORM };

pub fn open(app: &mut VectorcraftApp, p: &Value) -> Result<Value, String> {
    let st = app.session.active().filter(|st| st.doc.world_map.is_some()).ok_or("Requires a world-map document")?;
    let modes = st.doc.map_semantics.as_ref().and_then(|s| s.modes.as_ref());
    let id = match p.get("id") {
        Some(v) => v.as_u64().and_then(|id| u32::try_from(id).ok()).ok_or("Invalid map mode ID")?,
        None => modes.map_or(1, |m| m.active),
    };
    if modes.map_or_else(|| (1..=MapModes::STARTER_NAMES.len() as u32).contains(&id), |m| m.names.contains_key(&id)) {
        let filter = modes.and_then(|m| m.filters.get(&id)).cloned().unwrap_or_default();
        app.ui.dialog = Some(Dialog::new(
            KIND,
            json!({
                "id":id, "document":st.uid, "rules":filter, "kind":"category", "category":"core:river",
                "tag":"", "key":"", "value":"", "error":""
            }),
        ));
        Ok(Value::Null)
    } else {
        Err("Unknown map mode".into())
    }
}

fn category_name(doc: &Document, id: &str) -> String {
    if let Some(name) = doc.map_semantics.as_ref().and_then(|s| s.categories.get(id)) {
        return name.clone();
    }
    match id {
        "core:land" => tl!("Land"),
        "core:ocean" => tl!("Ocean"),
        "core:lake" => tl!("Lake"),
        "core:river" => tl!("River"),
        "core:coastline" => tl!("Coastline"),
        "core:biome" => tl!("Biome"),
        "core:border" => tl!("Border"),
        "core:road" => tl!("Road"),
        "core:settlement" => tl!("Settlement"),
        "core:parcel" => tl!("Parcel"),
        "core:building" => tl!("Building"),
        _ => id,
    }
    .to_owned()
}

fn group_name(doc: &Document, id: NodeId) -> String {
    doc.ancestry(id).map_or_else(
        || format!("{} #{}", tl!("Missing group"), id.0),
        |ids| ids.into_iter().filter_map(|id| doc.node(id).map(|n| n.display_name())).collect::<Vec<_>>().join(" › "),
    )
}

fn description(doc: &Document, rule: &MapFilterRule) -> String {
    match rule {
        MapFilterRule::Group { id } => group_name(doc, *id),
        MapFilterRule::Category { id } => category_name(doc, id),
        MapFilterRule::Tag { tag } => format!("{} · {tag}", tl!("Tag")),
        MapFilterRule::Property { key, value } => format!("{key} = {value}"),
    }
}

fn draft(d: &Dialog) -> Result<MapFilter, String> {
    let value = d.fields.get("rules").ok_or("Missing map mode rules")?;
    <MapFilter as serde::Deserialize>::deserialize(value).map_err(|e| e.to_string())
}

fn body(app: &mut VectorcraftApp, ui: &mut egui::Ui, d: &mut Dialog) -> bool {
    let Some(st) = app.session.active().filter(|st| st.doc.world_map.is_some()) else { return false };
    let doc = &st.doc;
    let Ok(mut filter) = draft(d) else {
        ui.label(tl!("Invalid filter rules"));
        return false;
    };
    let mut remove = None;
    ui.label(tl!("Hidden in this mode"));
    egui::ScrollArea::vertical().id_salt("map-hidden-rules").max_height(180.0).show(ui, |ui| {
        if filter.0.is_empty() {
            ui.label(tl!("No filter rules"));
        }
        for (index, rule) in filter.0.iter().enumerate() {
            ui.push_id(index, |ui| {
                ui.horizontal(|ui| {
                    if ui.button(tl!("Show")).clicked() {
                        remove = Some(rule.clone());
                    }
                    ui.label(description(doc, rule));
                });
            });
        }
    });
    let mut changed = remove.is_some_and(|rule| filter.0.remove(&rule));
    ui.separator();
    let mut kind = d.str("kind");
    ui.horizontal_wrapped(|ui| {
        for (id, label) in [("category", tl!("Category")), ("group", tl!("Groups")), ("tag", tl!("Tag")), ("property", tl!("Property"))] {
            ui.selectable_value(&mut kind, id.into(), label);
        }
    });
    d.fields.insert("kind".into(), json!(kind));
    let mut add = Vec::new();
    match kind.as_str() {
        "category" => {
            let current = d.str("category");
            let chosen = crate::widgets::combo(ui, "map-filter-category", &category_name(doc, &current), 180.0, |ui| {
                egui::ScrollArea::vertical()
                    .max_height(250.0)
                    .show(ui, |ui| {
                        let mut choice = None;
                        let custom = doc.map_semantics.iter().flat_map(|s| s.categories.keys().map(String::as_str));
                        for id in BUILTIN_CATEGORIES.iter().copied().chain(custom) {
                            if ui.selectable_label(id == current, category_name(doc, id)).clicked() {
                                choice = Some(id.to_owned());
                            }
                        }
                        choice
                    })
                    .inner
            });
            if let Some(id) = chosen {
                d.fields.insert("category".into(), json!(id));
            }
            if ui.button(tl!("Hide Category")).clicked() {
                add.push(MapFilterRule::Category { id: d.str("category") });
            }
        }
        "group" => {
            let groups: Vec<_> = st.highlighted_rows().into_iter().filter(|id| doc.node(*id).is_some_and(MapFilterRule::is_group)).collect();
            if groups.is_empty() {
                ui.label(tl!("Highlight layers or groups in the Layers panel first."));
            }
            if ui.add_enabled(!groups.is_empty(), egui::Button::new(tl!("Hide Highlighted Groups"))).clicked() {
                add.extend(groups.into_iter().map(|id| MapFilterRule::Group { id }));
            }
        }
        "tag" => {
            form::text(ui, d, "tag", 280.0);
            if ui.button(tl!("Hide Tag")).clicked() {
                add.push(MapFilterRule::Tag { tag: d.str("tag") });
            }
        }
        "property" => {
            ui.label(tl!("Property"));
            form::text(ui, d, "key", 280.0);
            ui.label(tl!("Value"));
            form::text(ui, d, "value", 280.0);
            if ui.button(tl!("Hide Property Value")).clicked() {
                add.push(MapFilterRule::Property { key: d.str("key"), value: d.str("value") });
            }
        }
        _ => {}
    }
    if !add.is_empty() {
        let mut next = filter.clone();
        next.0.extend(add);
        match serde_json::from_value::<MapFilter>(json!(next)) {
            Ok(valid) => {
                filter = valid;
                changed = true;
                d.fields.insert("error".into(), json!(""));
            }
            Err(_) => {
                d.fields.insert("error".into(), json!("Invalid filter rules"));
            }
        }
    }
    if changed {
        d.fields.insert("rules".into(), json!(filter));
    }
    let error = d.str("error");
    if !error.is_empty() {
        ui.label(crate::i18n::t(&error));
    }
    false
}

fn confirm(app: &mut VectorcraftApp, d: &Dialog) -> Result<Value, String> {
    if app.session.active().map(|st| st.uid) != d.fields.get("document").and_then(Value::as_u64) {
        return Err("The active document changed".into());
    }
    let filter = draft(d)?;
    let result = app.run("map.mode.setFilter", json!({"id": d.fields.get("id"), "rules": filter}));
    if result.is_ok() {
        app.ui.dialog = None;
    }
    result
}
