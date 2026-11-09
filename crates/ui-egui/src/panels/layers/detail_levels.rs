//! World-map level controls; edits always go through engine commands.
use super::{Ui, VectorcraftApp, json};

pub(super) fn show(app: &mut VectorcraftApp, ui: &mut Ui) {
    let Some(st) = app.session.active() else { return };
    if st.doc.world_map.is_none() {
        return;
    }
    let uid = st.uid;
    let mut levels = Vec::new();
    st.doc.walk(|n| {
        if n.extras().detail_level {
            levels.push((n.id, n.display_name()));
        }
    });
    let mut actions = Vec::new();
    egui::CollapsingHeader::new(tl!("Detail Levels")).default_open(true).show(ui, |ui| {
        if ui.button(tl!("New Detail Level")).clicked() {
            actions.push(("map.detailLevel.create", json!({})));
        }
        egui::ScrollArea::vertical().id_salt("detail-level-list").max_height(210.0).show(ui, |ui| {
            for (id, name) in levels {
                ui.push_id((uid, id.0), |ui| {
                    let key = ui.scope_id().with("detail-name");
                    let mut draft = ui
                        .data(|d| d.get_temp::<(String, String)>(key))
                        .filter(|(saved, _)| saved == &name)
                        .map(|(_, draft)| draft)
                        .unwrap_or_else(|| name.clone());
                    ui.text_edit_singleline(&mut draft);
                    ui.data_mut(|d| d.insert_temp(key, (name.clone(), draft.clone())));
                    ui.horizontal_wrapped(|ui| {
                        if ui.button(tl!("Rename")).clicked() {
                            actions.push(("map.detailLevel.rename", json!({"id": id, "name": draft})));
                            ui.data_mut(|d| d.remove::<(String, String)>(key));
                        }
                        if ui.button(tl!("Duplicate")).clicked() {
                            actions.push(("map.detailLevel.duplicate", json!({"id": id})));
                        }
                        if ui.button(tl!("Delete")).on_hover_text(tl!("Deletes this detail level and its contents. Undo restores them.")).clicked() {
                            actions.push(("map.detailLevel.delete", json!({"id": id})));
                        }
                    });
                });
            }
        });
    });
    for (command, params) in actions {
        // app.run presents command errors in the application's existing error surface.
        let _ = app.run(command, params);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use vectorcraft_engine::Session;

    fn texts(shape: &egui::Shape, out: &mut Vec<(String, egui::Pos2)>) {
        match shape {
            egui::Shape::Text(t) => out.push((t.galley.text().to_owned(), t.pos + t.galley.size() * 0.5)),
            egui::Shape::Vec(v) => v.iter().for_each(|s| texts(s, out)),
            _ => {}
        }
    }
    fn frame(app: &mut VectorcraftApp, ctx: &egui::Context, events: Vec<egui::Event>) -> Vec<(String, egui::Pos2)> {
        let mut out = ctx.run_ui(
            egui::RawInput { screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(300.0, 600.0))), events, ..Default::default() },
            |ui| show(app, ui),
        );
        out.textures_delta.clear();
        let mut result = Vec::new();
        for s in &out.shapes {
            texts(&s.shape, &mut result);
        }
        result
    }
    #[test]
    fn detail_level_panel_is_world_only_and_create_button_runs_command() {
        let mut app = VectorcraftApp::new(Session::new(), Default::default());
        app.session.execute("file.new", &json!({})).unwrap();
        let ctx = egui::Context::default();
        assert!(frame(&mut app, &ctx, vec![]).is_empty());
        app.session.execute("file.newWorldMap", &json!({"created":1234567890})).unwrap();
        frame(&mut app, &ctx, vec![]);
        let drawn = frame(&mut app, &ctx, vec![]);
        let pos = drawn.iter().find(|(t, _)| t == "New Detail Level").unwrap().1;
        let event = |pressed| egui::Event::PointerButton { pos, button: egui::PointerButton::Primary, pressed, modifiers: Default::default() };
        frame(&mut app, &ctx, vec![egui::Event::PointerMoved(pos), event(true)]);
        frame(&mut app, &ctx, vec![event(false)]);
        let levels = app.session.execute("map.detailLevel.list", &json!({})).unwrap();
        assert_eq!(levels["levels"].as_array().unwrap().len(), 1);
        let drawn = frame(&mut app, &ctx, vec![]);
        assert!(drawn.iter().any(|(t, _)| t == "Ansicht 1"));
        assert!(drawn.iter().any(|(t, _)| t == "Duplicate"));
    }
}
