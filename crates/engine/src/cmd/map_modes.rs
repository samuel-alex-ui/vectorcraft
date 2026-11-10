//! Document-owned map mode catalogue and active choice.
use super::{CommandSpec, Result, Session, bad, cmd};
use serde_json::{Value, json};
use vectorcraft_doc::geography::{MapFilter, MapFilterRule, MapModes, SCHEMA_VERSION};

fn world(s: &Session) -> std::result::Result<(), String> {
    super::maskedit::outside_mask_editing(s)?;
    if s.doc().map_err(|e| e.to_string())?.doc.world_map.is_none() {
        return Err("Requires a world-map document".into());
    }
    Ok(())
}

pub fn specs() -> Vec<CommandSpec> {
    vec![
        cmd!(query "map.mode.list", "List Map Modes", [], None, "{} map mode catalogue and active ID → {active, modes: [{id, name}]}", world, list),
        cmd!("map.mode.select", "Select Map Mode", [], None, "{id} select a mode in the current world map", world, select),
        cmd!(
            "map.mode.setFilter",
            "Set Map Mode Filter",
            [],
            None,
            "{id, rules: [{kind: group, id}|{kind: category, id}|{kind: tag, tag}|{kind: property, key, value}]} replace the mode’s exclusion rules; [] clears them",
            world,
            set_filter
        ),
        cmd!(
            "map.mode.edit",
            "Configure Map Modes",
            [],
            None,
            "{action: create|rename|duplicate|delete, id? (required except create), name? (required except delete)} edit one mode without changing the drawing layer → {id}",
            world,
            edit
        ),
    ]
}

fn modes(s: &Session) -> Result<MapModes> {
    Ok(s.doc()?.doc.map_semantics.as_ref().and_then(|s| s.modes.clone()).unwrap_or_default())
}

fn list(s: &mut Session, _: &Value) -> Result<Value> {
    let modes = modes(s)?;
    let rows: Vec<_> = modes
        .names
        .into_iter()
        .map(|(id, name)| json!({"id": id, "name": name, "rules": modes.filters.get(&id).map_or_else(Vec::new, |f| f.0.iter().collect::<Vec<_>>())}))
        .collect();
    Ok(json!({"active": modes.active, "modes": rows}))
}

fn mode_id(p: &Value, modes: &MapModes, command: &str) -> Result<u32> {
    p.get("id")
        .and_then(Value::as_u64)
        .and_then(|id| u32::try_from(id).ok())
        .filter(|id| modes.names.contains_key(id))
        .ok_or_else(|| bad(command, "unknown map mode"))
}

fn save(s: &mut Session, label: &str, modes: MapModes, command: &str) -> Result<()> {
    modes.validate().map_err(|e| bad(command, e))?;
    s.edit(label, |doc, _| {
        let world = doc.world_map.ok_or_else(|| bad(command, "requires a world map"))?;
        doc.world_map = Some(world.with_content_schema(Some(SCHEMA_VERSION)).map_err(|e| bad(command, e.to_string()))?);
        doc.map_semantics.get_or_insert_with(Default::default).modes = Some(modes);
        Ok(())
    })
}

fn select(s: &mut Session, p: &Value) -> Result<Value> {
    const C: &str = "map.mode.select";
    let mut modes = modes(s)?;
    let id = mode_id(p, &modes, C)?;
    if modes.active != id {
        modes.active = id;
        save(s, "Select Map Mode", modes, C)?;
    }
    Ok(json!({"id": id}))
}

fn edit(s: &mut Session, p: &Value) -> Result<Value> {
    const C: &str = "map.mode.edit";
    let action = p.get("action").and_then(Value::as_str);
    let mut modes = modes(s)?;
    let target = if action == Some("create") { None } else { Some(mode_id(p, &modes, C)?) };
    let name = || {
        p.get("name")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|name| !name.is_empty() && name.len() <= 256 && !name.chars().any(char::is_control))
            .ok_or_else(|| bad(C, "name must be nonblank, at most 256 bytes, without control characters"))
    };
    let id = match (action, target) {
        (Some("create" | "duplicate"), _) => {
            let id = modes.next_id;
            modes.next_id = id.checked_add(1).ok_or_else(|| bad(C, "no free mode ID"))?;
            modes.names.insert(id, name()?.into());
            if let Some(filter) = target.and_then(|source| modes.filters.get(&source)).cloned() {
                modes.filters.insert(id, filter);
            }
            id
        }
        (Some("rename"), Some(id)) => {
            let name = name()?;
            if modes.names.get(&id).is_some_and(|old| old == name) {
                return Ok(json!({"id": id}));
            }
            modes.names.insert(id, name.into());
            id
        }
        (Some("delete"), Some(id)) => {
            modes.names.remove(&id);
            modes.filters.remove(&id);
            let first = modes.names.keys().next().copied().ok_or_else(|| bad(C, "keep at least one map mode"))?;
            if modes.active == id {
                modes.active = first;
            }
            id
        }
        _ => return Err(bad(C, "unknown map mode action")),
    };
    save(s, "Configure Map Modes", modes, C)?;
    Ok(json!({"id": id}))
}

fn set_filter(s: &mut Session, p: &Value) -> Result<Value> {
    const C: &str = "map.mode.setFilter";
    let mut modes = modes(s)?;
    let id = mode_id(p, &modes, C)?;
    let rules = p.get("rules").ok_or_else(|| bad(C, "missing filter rules"))?;
    let filter = <MapFilter as serde::Deserialize>::deserialize(rules).map_err(|e| bad(C, e.to_string()))?;
    let doc = &s.doc()?.doc;
    if filter.0.iter().any(|rule| matches!(rule, MapFilterRule::Group { id } if doc.node(*id).is_some_and(|node| !MapFilterRule::is_group(node)))) {
        return Err(bad(C, "group rules require a layer or group ID"));
    }
    if modes.filters.get(&id).map_or(filter.0.is_empty(), |old| old == &filter) {
        return Ok(json!({"id": id}));
    }
    if filter.0.is_empty() {
        modes.filters.remove(&id);
    } else {
        modes.filters.insert(id, filter);
    }
    save(s, "Set Map Mode Filter", modes, C)?;
    Ok(json!({"id": id}))
}
