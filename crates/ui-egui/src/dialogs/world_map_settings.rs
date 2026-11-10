use serde_json::{Value, json};
use vectorcraft_engine::doc::WorldMap;

use super::{DialogSpec, form};
use crate::{VectorcraftApp, state::Dialog};

pub const KIND: &str = "worldMapSettings";
pub(super) const SPEC: DialogSpec = DialogSpec {
    heading: |d| if d.bool("new") { tl!("New World Map…").into() } else { tl!("World Map Settings…").into() },
    body,
    confirm,
    min_width: 340.0,
    ..DialogSpec::FORM
};

pub fn open(app: &mut VectorcraftApp, new: bool) -> Result<Value, String> {
    let (map, title, uid) = if new {
        (WorldMap::standard(), "Aetherys World Map", None)
    } else {
        let st = app.session.active().ok_or("Requires a world-map document")?;
        (st.doc.world_map.ok_or("Requires a world-map document")?, st.doc.title.as_str(), Some(st.uid))
    };
    let radius = map.planet_radius_m().map_or_else(String::new, |m| (m / 1000.0).to_string());
    app.ui.dialog = Some(Dialog::new(KIND, json!({"new":new, "document":uid, "name":title, "radiusKm":radius})));
    Ok(Value::Null)
}

fn body(app: &mut VectorcraftApp, ui: &mut egui::Ui, d: &mut Dialog) -> bool {
    ui.label(tl!("Planet Name"));
    form::text(ui, d, "name", 280.0);
    ui.label(tl!("Planet Radius in km (optional)"));
    form::text(ui, d, "radiusKm", 140.0);
    let map = if d.bool("new") { Some(WorldMap::standard()) } else { app.session.active().and_then(|st| st.doc.world_map) };
    if let Some(map) = map {
        let (w, h) = map.size();
        ui.label(format!("{}  {}", tl!("Projection"), tl!("Equirectangular")));
        ui.label(format!("{}  {w} × {h} pt", tl!("Map Extent")));
        ui.label("180° W – 180° E · 90° S – 90° N");
    }
    false
}

fn confirm(app: &mut VectorcraftApp, d: &Dialog) -> Result<Value, String> {
    let name = d.str("name");
    let name = name.trim();
    if name.is_empty() {
        return Err(tl!("Enter a planet name.").into());
    }
    let radius = d.str("radiusKm");
    let radius = radius.trim();
    let km = if radius.is_empty() {
        None
    } else {
        Some(radius.replace(',', ".").parse::<f64>().map_err(|_| tl!("Enter a positive radius in kilometres.").to_owned())?)
    };
    let metres = km.map(|km| km * 1000.0);
    if metres.is_some_and(|m| !m.is_finite() || m <= 0.0) {
        return Err(tl!("Enter a positive radius in kilometres.").into());
    }
    if d.bool("new") {
        app.run("file.newWorldMap", json!({"name":name}))?;
        if let Some(m) = metres {
            app.run("map.setRadius", json!({"radiusM":m}))?;
        }
    } else {
        let st = app
            .session
            .active()
            .filter(|st| Some(st.uid) == d.fields.get("document").and_then(Value::as_u64))
            .ok_or("The active document changed")?;
        let map = st.doc.world_map.ok_or("Requires a world-map document")?;
        let mut commands = Vec::new();
        if st.doc.title != name {
            commands.push(json!({"command":"file.info","params":{"title":name}}));
        }
        if map.planet_radius_m().map(|m| m / 1000.0) != km {
            commands.push(json!({"command":"map.setRadius","params":{"radiusM":metres}}));
        }
        if !commands.is_empty() {
            app.run("command.batch", json!({"label":"World Map Settings","commands":commands}))?;
        }
    }
    app.ui.dialog = None;
    Ok(Value::Null)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn world_map_settings_create_edit_undo_and_reopen() {
        let mut app = VectorcraftApp::new(vectorcraft_engine::Session::new(), Default::default());
        let field = |app: &mut VectorcraftApp, key: &str, value: &str| {
            app.ui.dialog.as_mut().unwrap().fields.insert(key.into(), json!(value));
        };
        app.run("ui.newWorldMap", json!({})).unwrap();
        assert!(app.session.active().is_none());
        let ctx = egui::Context::default();
        crate::theme::install_fonts(&ctx);
        let mut output = ctx.run_ui(Default::default(), |ui| super::super::show(&mut app, ui.ctx()));
        assert!(!output.shapes.is_empty());
        output.textures_delta.clear();
        field(&mut app, "name", "Eldoria");
        field(&mut app, "radiusKm", "-2");
        assert!(super::super::confirm(&mut app).is_err());
        assert!(app.session.active().is_none());
        field(&mut app, "radiusKm", "1234,5");
        super::super::confirm(&mut app).unwrap();
        let initial = app.session.doc().unwrap().doc.clone();
        assert_eq!(initial.title, "Eldoria");
        assert_eq!(initial.world_map.unwrap().planet_radius_m(), Some(1_234_500.0));
        assert_eq!(initial.world_map.unwrap().size(), (1000.0, 500.0));

        app.run("ui.worldMapSettings", json!({})).unwrap();
        assert_eq!(app.ui.dialog.as_ref().unwrap().str("radiusKm"), "1234.5");
        field(&mut app, "name", "Eldoria II");
        field(&mut app, "radiusKm", "");
        super::super::confirm(&mut app).unwrap();
        let edited = app.session.doc().unwrap().doc.clone();
        assert_eq!(edited.title, "Eldoria II");
        assert_eq!(edited.world_map.unwrap().planet_radius_m(), None);
        app.run("edit.undo", json!({})).unwrap();
        assert_eq!(app.session.doc().unwrap().doc, initial);
        app.run("edit.redo", json!({})).unwrap();
        assert_eq!(app.session.doc().unwrap().doc, edited);

        let bytes = vectorcraft_format::save_with(&edited, &Default::default()).unwrap();
        let reopened = vectorcraft_format::load(&bytes).unwrap();
        app.session.add_document(reopened, None);
        app.run("ui.worldMapSettings", json!({})).unwrap();
        assert_eq!(app.ui.dialog.as_ref().unwrap().str("name"), "Eldoria II");
        assert_eq!(app.ui.dialog.as_ref().unwrap().str("radiusKm"), "");
        let history = app.session.doc().unwrap().history.undo.len();
        super::super::confirm(&mut app).unwrap();
        assert_eq!(app.session.doc().unwrap().history.undo.len(), history);
    }
}
