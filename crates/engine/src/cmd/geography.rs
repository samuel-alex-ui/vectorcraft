//! Geographic metadata only. Modes, rendering and layer organization remain separate commands.
use serde::Deserialize;
use serde_json::{Value, json};
use vectorcraft_doc::{GeographicMetadata, MapSemantics, geography::valid_entity_id};

use super::overprint::edit_counted;
use super::{CommandSpec, Result, Session, bad, cmd, has_doc, named_ids, targets};

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
        cmd!(
            "map.setEntity",
            "Set Geographic Entity",
            [],
            None,
            "{ids?|id?, entity: UUID|null} set or clear only the shared geographic entity ID, world maps only, one undo step → {changed}",
            world,
            set_entity
        ),
        cmd!(query "map.entityRepresentations", "Geographic Entity Representations", [], None,
            "{entity: UUID} list document objects sharing this entity, including hidden variants, in paint order → {entity, objects: [{id, detailLevel}]}", world, representations),
        cmd!(
            "map.setRadius",
            "Set Planet Radius",
            [],
            None,
            "{radiusM: positive metres|null} set or clear the spherical planet radius, one undo step → {radiusM}",
            world,
            set_radius
        ),
        cmd!(query "map.measure", "Measure World Map", [], None,
            "{from: [longitude,latitude], to: [longitude,latitude]} shortest spherical distance and local scale at from → {distanceM, distanceKm, metresPerPoint: {horizontal, vertical}}",
            world, measure),
        cmd!(
            "map.setLongitudeWrap",
            "Wrap World Map Longitude",
            [],
            None,
            "{enabled: bool} wrap longitude at the two horizontal map edges → {enabled}",
            world,
            set_longitude_wrap
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

fn entity_id<'a>(value: Option<&'a Value>, command: &str) -> Result<&'a str> {
    value.and_then(Value::as_str).filter(|id| valid_entity_id(id)).ok_or_else(|| bad(command, "give a non-nil lowercase UUID"))
}

fn set_entity(s: &mut Session, p: &Value) -> Result<Value> {
    const C: &str = "map.setEntity";
    let value = p.get("entity");
    let entity = if value.is_some_and(Value::is_null) { None } else { Some(entity_id(value, C)?) };
    let ids = named_ids(p)?.unwrap_or_else(|| s.active().map_or_else(Vec::new, |st| st.selection.objects.clone()));
    if ids.is_empty() {
        return Err(bad(C, "give at least one target"));
    }
    let changed = edit_counted(s, "Set Geographic Entity", |d| {
        let mut changed = 0;
        for id in &ids {
            let node = d.node_mut(*id).ok_or_else(|| bad(C, "target does not exist"))?;
            let existing = node.extras().geography.as_deref();
            if existing.and_then(|g| g.geographic_entity_id.as_deref()) == entity {
                continue;
            }
            let mut geography = existing.cloned().unwrap_or_default();
            geography.geographic_entity_id = entity.map(str::to_owned);
            node.extras_mut().geography = (!geography.is_empty()).then_some(Box::new(geography));
            changed += 1;
        }
        if changed > 0 && entity.is_some() {
            enable(d).map_err(|e| bad(C, e))?;
        }
        Ok(changed)
    })?;
    Ok(json!({"changed": changed}))
}

fn representations(s: &mut Session, p: &Value) -> Result<Value> {
    let entity = entity_id(p.get("entity"), "map.entityRepresentations")?;
    let mut objects = Vec::new();
    for layer in &s.doc()?.doc.layers {
        let detail_level = layer.extras().detail_level.then_some(layer.id);
        layer.walk(&mut |node| {
            if node.extras().geography.as_ref().and_then(|g| g.geographic_entity_id.as_deref()) == Some(entity) {
                objects.push(json!({"id": node.id, "detailLevel": detail_level}));
            }
        });
    }
    Ok(json!({"entity": entity, "objects": objects}))
}

fn set_radius(s: &mut Session, p: &Value) -> Result<Value> {
    const C: &str = "map.setRadius";
    let old = s.doc()?.doc.world_map.ok_or_else(|| bad(C, "requires a world map"))?;
    let new = match p.get("radiusM") {
        Some(Value::Null) => old.without_planet_radius(),
        Some(v) => old
            .with_planet_radius_m(v.as_f64().ok_or_else(|| bad(C, "give positive metres or null"))?)
            .map_err(|_| bad(C, "radius must be finite and positive"))?,
        None => return Err(bad(C, "give radiusM")),
    };
    if old != new {
        s.edit("Set Planet Radius", |d, _| {
            d.world_map = Some(new);
            Ok(())
        })?;
    }
    Ok(json!({"radiusM":new.planet_radius_m()}))
}

fn measure(s: &mut Session, p: &Value) -> Result<Value> {
    const C: &str = "map.measure";
    let map = s.doc()?.doc.world_map.ok_or_else(|| bad(C, "requires a world map"))?;
    let point = |key| -> Result<vectorcraft_doc::GeographicPoint> {
        let [longitude_deg, latitude_deg] =
            super::finite_numbers(p.get(key).unwrap_or(&Value::Null)).ok_or_else(|| bad(C, format!("give {key} as [longitude, latitude]")))?;
        Ok(vectorcraft_doc::GeographicPoint { longitude_deg, latitude_deg })
    };
    let from = point("from")?;
    let to = point("to")?;
    let distance = map.distance_m(from, to).map_err(|e| bad(C, e.to_string()))?;
    let (horizontal, vertical) = map.local_scale(from.latitude_deg).map_err(|e| bad(C, e.to_string()))?;
    Ok(json!({"distanceM":distance, "distanceKm":distance / 1000.0, "metresPerPoint":{"horizontal":horizontal,"vertical":vertical}}))
}

fn set_longitude_wrap(s: &mut Session, p: &Value) -> Result<Value> {
    const C: &str = "map.setLongitudeWrap";
    let enabled = p.get("enabled").and_then(Value::as_bool).ok_or_else(|| bad(C, "give enabled as a boolean"))?;
    let old = s.doc()?.doc.world_map.ok_or_else(|| bad(C, "requires a world map"))?;
    if old.longitude_wrap() != enabled {
        s.edit("Wrap World Map Longitude", |d, _| {
            d.world_map = Some(old.with_longitude_wrap(enabled));
            Ok(())
        })?;
    }
    Ok(json!({"enabled":enabled}))
}
