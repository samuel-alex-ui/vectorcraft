//! Geographic metadata only. Modes, rendering and layer organization remain separate commands.
use serde::Deserialize;
use serde_json::{Value, json};
use vectorcraft_doc::{GeographicMetadata, MapSemantics};

use super::overprint::edit_counted;
use super::{CommandSpec, Result, Session, bad, cmd, has_doc, targets};

fn world(s: &Session) -> std::result::Result<(), String> {
    has_doc(s)?;
    if s.doc().map_err(|e| e.to_string())?.doc.world_map.is_none() {
        return Err("Requires a world-map document".into());
    }
    Ok(())
}

pub fn specs() -> Vec<CommandSpec> {
    vec![
        cmd!(
            "map.setGeography",
            "Geographic Metadata",
            [],
            None,
            "{ids?|id?, geography: {categories?: [string], tags?: [string], properties?: {key: string}, geographic_entity_id?: UUID|null}|null} replace each target's metadata (null clears), world maps only, one undo step; sharing an explicitly supplied UUID links independent representations → {changed}",
            world,
            set
        ),
        cmd!(
            "map.setCategory",
            "Geographic Category",
            [],
            None,
            "{category: string (user:UUID), label: string|null} define/rename a category's display label, null removes only the label, not node references; world maps only → {changed}",
            world,
            category
        ),
        cmd!(query "map.geographyInfo", "Geographic Metadata Info", [], None,
            "{ids?|id?} read metadata and category definitions; no inferred inheritance or geometry synchronization → {schema, objects: [{id, geography}]}", world, info),
    ]
}

fn set(s: &mut Session, p: &Value) -> Result<Value> {
    const C: &str = "map.setGeography";
    let value = p.get("geography").ok_or_else(|| bad(C, "give geography or null"))?;
    let geography = if value.is_null() {
        None
    } else {
        let g: GeographicMetadata = GeographicMetadata::deserialize(value).map_err(|e| bad(C, e.to_string()))?;
        g.validate().map_err(|e| bad(C, e))?;
        (!g.is_empty()).then_some(Box::new(g))
    };
    let ids = targets(s, p)?;
    if ids.is_empty() {
        return Err(bad(C, "give at least one target"));
    }
    let changed = edit_counted(s, "Geographic Metadata", |d| {
        // Check every target before applying any edit, including explicit missing IDs.
        for id in &ids {
            if d.node(*id).is_none() {
                return Err(bad(C, "target does not exist"));
            }
        }
        let mut changed = 0;
        for id in &ids {
            if let Some(node) = d.node_mut(*id)
                && node.extras().geography != geography
            {
                node.extras_mut().geography.clone_from(&geography);
                changed += 1;
            }
        }
        if changed > 0 {
            if geography.is_some() {
                enable(d).map_err(|e| bad(C, e))?;
            }
            d.validate_geography().map_err(|e| bad(C, e))?;
        }
        Ok(changed)
    })?;
    Ok(json!({"changed": changed}))
}

fn enable(d: &mut vectorcraft_doc::Document) -> std::result::Result<(), String> {
    let map = d.world_map.ok_or("Requires a world-map document")?;
    d.world_map = Some(map.with_content_schema(Some(vectorcraft_doc::geography::SCHEMA_VERSION)).map_err(|e| e.to_string())?);
    d.map_semantics.get_or_insert_with(MapSemantics::default);
    Ok(())
}

fn category(s: &mut Session, p: &Value) -> Result<Value> {
    const C: &str = "map.setCategory";
    let id = p.get("category").and_then(Value::as_str).ok_or_else(|| bad(C, "give category: user:UUID"))?;
    let label = p.get("label").ok_or_else(|| bad(C, "give label or null"))?;
    if id.len() > 64 {
        return Err(bad(C, "category ID is too long"));
    }
    // Reuse the bounded schema reader and validator for both commands and files.
    let probe = json!({"schema_version": 1, "categories": {id: "Category"}});
    let probe: MapSemantics = serde_json::from_value(probe).map_err(|e| bad(C, e.to_string()))?;
    probe.validate().map_err(|e| bad(C, e))?;
    let changed = edit_counted(s, "Geographic Category", |d| {
        let mut schema = d.map_semantics.clone().unwrap_or_default();
        if label.is_null() {
            schema.categories.remove(id);
        } else {
            let label = label.as_str().ok_or_else(|| bad(C, "label must be text or null"))?;
            if label.len() > 256 {
                return Err(bad(C, "label is too long"));
            }
            schema.categories.insert(id.into(), label.into());
        }
        schema.validate().map_err(|e| bad(C, e))?;
        if d.map_semantics.as_ref().map_or(schema.categories.is_empty(), |s| s.categories == schema.categories) {
            return Ok(0);
        }
        enable(d).map_err(|e| bad(C, e))?;
        d.map_semantics = Some(schema);
        Ok(1)
    })?;
    Ok(json!({"changed": changed}))
}

fn info(s: &mut Session, p: &Value) -> Result<Value> {
    let d = &s.doc()?.doc;
    let ids = targets(s, p)?;
    let mut objects = Vec::new();
    for id in ids {
        let node = d.node(id).ok_or_else(|| bad("map.geographyInfo", "target does not exist"))?;
        objects.push(json!({"id": id, "geography": node.extras().geography}));
    }
    Ok(json!({"schema": d.map_semantics, "objects": objects}))
}
