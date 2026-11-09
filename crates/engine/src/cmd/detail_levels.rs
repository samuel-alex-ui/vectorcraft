//! Explicit detail-level layers. No automatic adoption of existing map layers.
use super::{CommandSpec, Result, Session, bad, cmd, has_doc};
use serde_json::{Value, json};
use std::sync::Arc;
use vectorcraft_doc::{LayerColor, Node, NodeId};

fn world(s: &Session) -> std::result::Result<(), String> {
    has_doc(s)?;
    super::maskedit::outside_mask_editing(s)?;
    if s.doc().map_err(|e| e.to_string())?.doc.world_map.is_none() {
        return Err("Requires a world-map document".into());
    }
    Ok(())
}
pub fn specs() -> Vec<CommandSpec> {
    vec![
        cmd!(
            "map.detailLevel.create",
            "New Detail Level",
            [],
            None,
            "{name?: string} add a marked top-level layer with five standard groups; existing layers stay unchanged → {id}",
            world,
            create
        ),
        cmd!(
            "map.detailLevel.rename",
            "Rename Detail Level",
            [],
            None,
            "{id, name: string} rename a marked layer, one undo step → {changed}",
            world,
            rename
        ),
        cmd!(
            "map.detailLevel.duplicate",
            "Duplicate Detail Level",
            [],
            None,
            "{id} deep copy with new node IDs and detached geographic entity links → {ids}",
            world,
            duplicate
        ),
        cmd!(
            "map.detailLevel.delete",
            "Delete Detail Level",
            [],
            None,
            "{id} delete a marked layer and contents with undo; the last document layer stays → {deleted}",
            world,
            delete
        ),
        cmd!(query "map.detailLevel.list", "List Detail Levels", [], None, "{} list explicit detail layers, including any moved into other layers; never infer from names → {levels: [{id, name}]}", world, list),
    ]
}
fn name(p: &Value, c: &str) -> Result<String> {
    let n = p.get("name").and_then(Value::as_str).ok_or_else(|| bad(c, "give a text name"))?;
    if n.trim().is_empty() || n.len() > 256 || n.chars().any(char::is_control) {
        return Err(bad(c, "name must be nonblank, at most 256 bytes, without control characters"));
    }
    Ok(n.to_owned())
}
fn target(s: &Session, p: &Value, c: &str) -> Result<NodeId> {
    let id = p.get("id").and_then(Value::as_u64).map(NodeId).ok_or_else(|| bad(c, "give a detail-level id"))?;
    let d = &s.doc()?.doc;
    if !d.node(id).is_some_and(|n| n.is_layer() && n.extras().detail_level) {
        return Err(bad(c, "target is not a detail level"));
    }
    Ok(id)
}
fn create(s: &mut Session, p: &Value) -> Result<Value> {
    const C: &str = "map.detailLevel.create";
    let n = if p.get("name").is_some() {
        name(p, C)?
    } else {
        // Find a free default label; user names need not be unique.
        let d = &s.doc()?.doc;
        let names: std::collections::HashSet<_> = d.layers.iter().filter_map(|n| n.name.as_deref()).collect();
        let mut i = 1usize;
        loop {
            let candidate = format!("Ansicht {i}");
            if !names.contains(candidate.as_str()) {
                break candidate;
            }
            i = i.checked_add(1).ok_or_else(|| bad(C, "no free default name"))?;
        }
    };
    let id = s.edit("New Detail Level", |d, _| {
        let world = d.world_map.ok_or_else(|| bad(C, "requires a world map"))?;
        d.world_map = Some(world.with_content_schema(Some(vectorcraft_doc::geography::SCHEMA_VERSION)).map_err(|e| bad(C, e.to_string()))?);
        d.map_semantics.get_or_insert_with(Default::default).detail_levels_version = Some(1);
        let id = d.alloc_id();
        let mut level = Node::layer(id, &n, LayerColor::Preset(0));
        level.extras_mut().detail_level = true;
        let mut groups = Vec::new();
        for label in ["Konturen & Meere", "Geografie", "Politik", "Wirtschaft", "Infrastruktur"] {
            let mut group = Node::group(d.alloc_id(), vec![]);
            group.name = Some(label.into());
            groups.push(Arc::new(group));
        }
        if let Some(children) = level.children_mut() {
            *children = groups;
        }
        d.layers.push(Arc::new(level));
        Ok(id)
    })?;
    // Creating a level does not silently change the drawing layer.
    Ok(json!({"id": id}))
}
fn rename(s: &mut Session, p: &Value) -> Result<Value> {
    const C: &str = "map.detailLevel.rename";
    let id = target(s, p, C)?;
    let n = name(p, C)?;
    let changed = super::overprint::edit_counted(s, "Rename Detail Level", |d| {
        let node = d.node_mut(id).ok_or_else(|| bad(C, "missing layer"))?;
        if node.name.as_deref() == Some(&n) {
            return Ok(0);
        }
        node.name = Some(n);
        Ok(1)
    })?;
    Ok(json!({"changed": changed}))
}
fn duplicate(s: &mut Session, p: &Value) -> Result<Value> {
    let id = target(s, p, "map.detailLevel.duplicate")?;
    super::layer::duplicate_rows(s, &json!({"id": id}))
}
fn delete(s: &mut Session, p: &Value) -> Result<Value> {
    let id = target(s, p, "map.detailLevel.delete")?;
    super::layer::delete_rows(s, &json!({"id": id}))
}
fn list(s: &mut Session, _: &Value) -> Result<Value> {
    let mut levels = Vec::new();
    s.doc()?.doc.walk(|n| {
        if n.extras().detail_level {
            levels.push(json!({"id": n.id, "name": n.display_name()}));
        }
    });
    Ok(json!({"levels": levels}))
}
