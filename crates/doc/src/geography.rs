//! Optional world-map semantics; never a second geometry store.
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::marker::PhantomData;

use serde::de::{DeserializeSeed, Error, MapAccess, SeqAccess, Visitor};
use serde::{Deserialize, Deserializer, Serialize};

pub const SCHEMA_VERSION: u32 = 1;
pub const BUILTIN_CATEGORIES: &[&str] = &[
    "core:land",
    "core:ocean",
    "core:lake",
    "core:river",
    "core:coastline",
    "core:biome",
    "core:border",
    "core:road",
    "core:settlement",
    "core:parcel",
    "core:building",
];

/// Accepted UUID text: lower-case hex-and-dash, excluding nil. No UUIDs are generated here.
/// Text layout: https://www.rfc-editor.org/rfc/rfc9562.html#section-4
pub fn valid_entity_id(s: &str) -> bool {
    s.len() == 36
        && s.bytes().enumerate().all(
            |(i, b)| {
                if matches!(i, 8 | 13 | 18 | 23) { b == b'-' } else { b.is_ascii_digit() || (b'a'..=b'f').contains(&b) }
            },
        )
        && s.bytes().any(|b| b != b'0' && b != b'-')
}

fn user_category(s: &str) -> bool {
    s.strip_prefix("user:").is_some_and(valid_entity_id)
}

fn text(s: &str, max: usize) -> bool {
    !s.trim().is_empty() && s.len() <= max && !s.chars().any(char::is_control)
}

struct Text<const N: usize>;
impl<'de, const N: usize> DeserializeSeed<'de> for Text<N> {
    type Value = String;
    fn deserialize<D: Deserializer<'de>>(self, d: D) -> Result<String, D::Error> {
        struct Read<const N: usize>;
        impl<const N: usize> Visitor<'_> for Read<N> {
            type Value = String;
            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, "text of at most {N} bytes")
            }
            fn visit_str<E: Error>(self, s: &str) -> Result<String, E> {
                if s.len() > N {
                    return Err(E::custom("geographic text exceeds byte limit"));
                }
                Ok(s.to_owned())
            }
        }
        d.deserialize_str(Read::<N>)
    }
}

fn strings<'de, D: Deserializer<'de>, const N: usize, const L: usize>(d: D) -> Result<Vec<String>, D::Error> {
    struct Read<const N: usize, const L: usize>;
    impl<'de, const N: usize, const L: usize> Visitor<'de> for Read<N, L> {
        type Value = Vec<String>;
        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "at most {N} strings")
        }
        fn visit_seq<A: SeqAccess<'de>>(self, mut a: A) -> Result<Self::Value, A::Error> {
            let mut values = Vec::new();
            let mut seen = BTreeSet::new();
            while let Some(s) = a.next_element_seed(Text::<L>)? {
                if values.len() >= N || !seen.insert(s.clone()) {
                    return Err(A::Error::custom("too many or duplicate geographic entries"));
                }
                values.push(s);
            }
            Ok(values)
        }
    }
    d.deserialize_seq(Read::<N, L>)
}

fn entries<'de, D: Deserializer<'de>, const N: usize, const K: usize, const V: usize>(d: D) -> Result<BTreeMap<String, String>, D::Error> {
    struct Read<const N: usize, const K: usize, const V: usize>(PhantomData<()>);
    impl<'de, const N: usize, const K: usize, const V: usize> Visitor<'de> for Read<N, K, V> {
        type Value = BTreeMap<String, String>;
        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "at most {N} text properties")
        }
        fn visit_map<A: MapAccess<'de>>(self, mut a: A) -> Result<Self::Value, A::Error> {
            let mut values = BTreeMap::new();
            while let Some(k) = a.next_key_seed(Text::<K>)? {
                if values.len() >= N || values.contains_key(&k) {
                    return Err(A::Error::custom("too many or duplicate geographic properties"));
                }
                let value = a.next_value_seed(Text::<V>)?;
                values.insert(k, value);
            }
            Ok(values)
        }
    }
    d.deserialize_map(Read::<N, K, V>(PhantomData))
}

fn entity<'de, D: Deserializer<'de>>(d: D) -> Result<Option<String>, D::Error> {
    // The field itself is optional; explicit null is accepted as removal.
    struct Read;
    impl<'de> Visitor<'de> for Read {
        type Value = Option<String>;
        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
            f.write_str("a UUID or null")
        }
        fn visit_none<E: Error>(self) -> Result<Self::Value, E> {
            Ok(None)
        }
        fn visit_unit<E: Error>(self) -> Result<Self::Value, E> {
            Ok(None)
        }
        fn visit_some<D: Deserializer<'de>>(self, d: D) -> Result<Self::Value, D::Error> {
            Text::<36>.deserialize(d).map(Some)
        }
    }
    d.deserialize_option(Read)
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GeographicMetadata {
    #[serde(default, skip_serializing_if = "Vec::is_empty", deserialize_with = "strings::<_, 32, 64>")]
    pub categories: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty", deserialize_with = "strings::<_, 64, 256>")]
    pub tags: Vec<String>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty", deserialize_with = "entries::<_, 64, 128, 4096>")]
    pub properties: BTreeMap<String, String>,
    #[serde(default, skip_serializing_if = "Option::is_none", deserialize_with = "entity")]
    pub geographic_entity_id: Option<String>,
}

impl GeographicMetadata {
    pub fn is_empty(&self) -> bool {
        *self == Self::default()
    }
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.categories.len() > 32 || self.tags.len() > 64 || self.properties.len() > 64 {
            return Err("geographic collection limit exceeded");
        }
        let unique = |v: &[String]| v.iter().collect::<BTreeSet<_>>().len() == v.len();
        if !unique(&self.categories)
            || !unique(&self.tags)
            || self.categories.iter().any(|s| !BUILTIN_CATEGORIES.contains(&s.as_str()) && !user_category(s))
            || self.tags.iter().any(|s| !text(s, 256))
            || self.properties.iter().any(|(k, v)| !text(k, 128) || v.len() > 4096)
            || self.geographic_entity_id.as_deref().is_some_and(|s| !valid_entity_id(s))
        {
            return Err("invalid geographic category, tag, property or UUID");
        }
        Ok(())
    }
}

/// One way to hide a row and its subtree in a map mode.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum MapFilterRule {
    Group {
        id: crate::NodeId,
    },
    Category {
        #[serde(deserialize_with = "rule_text::<_, 64>")]
        id: String,
    },
    Tag {
        #[serde(deserialize_with = "rule_text::<_, 256>")]
        tag: String,
    },
    Property {
        #[serde(deserialize_with = "rule_text::<_, 128>")]
        key: String,
        #[serde(deserialize_with = "rule_text::<_, 4096>")]
        value: String,
    },
}

fn rule_text<'de, D: Deserializer<'de>, const N: usize>(d: D) -> Result<String, D::Error> {
    Text::<N>.deserialize(d)
}

impl MapFilterRule {
    pub fn is_group(node: &crate::Node) -> bool {
        matches!(node.kind, crate::NodeKind::Layer { .. } | crate::NodeKind::Group { .. })
    }

    fn valid(&self) -> bool {
        match self {
            Self::Group { id } => id.0 != 0,
            Self::Category { id } => BUILTIN_CATEGORIES.contains(&id.as_str()) || user_category(id),
            Self::Tag { tag } => text(tag, 256),
            Self::Property { key, value } => text(key, 128) && value.len() <= 4096,
        }
    }

    fn matches(&self, node: &crate::Node) -> bool {
        let geo = node.extras().geography.as_deref();
        match self {
            Self::Group { id } => node.id == *id && Self::is_group(node),
            Self::Category { id } => geo.is_some_and(|g| g.categories.contains(id)),
            Self::Tag { tag } => geo.is_some_and(|g| g.tags.contains(tag)),
            Self::Property { key, value } => geo.and_then(|g| g.properties.get(key)) == Some(value),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
#[serde(transparent)]
pub struct MapFilter(pub BTreeSet<MapFilterRule>);

impl MapFilter {
    pub const MAX_RULES: usize = 128;

    pub fn hides(&self, node: &crate::Node) -> bool {
        self.0.iter().any(|rule| rule.matches(node))
    }

    fn validate(&self) -> Result<(), &'static str> {
        if self.0.len() > Self::MAX_RULES || self.0.iter().any(|rule| !rule.valid()) {
            return Err("invalid map filter rules");
        }
        Ok(())
    }
}

impl<'de> Deserialize<'de> for MapFilter {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        struct Read;
        impl<'de> Visitor<'de> for Read {
            type Value = MapFilter;
            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("up to 128 distinct map filter rules")
            }
            fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<MapFilter, A::Error> {
                let mut rules = BTreeSet::new();
                while let Some(rule) = seq.next_element()? {
                    if rules.len() >= MapFilter::MAX_RULES || !rules.insert(rule) {
                        return Err(A::Error::custom("too many or duplicate map filter rules"));
                    }
                }
                let filter = MapFilter(rules);
                filter.validate().map_err(A::Error::custom)?;
                Ok(filter)
            }
        }
        d.deserialize_seq(Read)
    }
}

fn mode_filters<'de, D: Deserializer<'de>>(d: D) -> Result<BTreeMap<u32, MapFilter>, D::Error> {
    struct Read;
    impl<'de> Visitor<'de> for Read {
        type Value = BTreeMap<u32, MapFilter>;
        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
            f.write_str("filters for up to 128 map modes")
        }
        fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
            let mut filters = BTreeMap::new();
            while let Some(id) = map.next_key::<u32>()? {
                if filters.len() >= 128 || filters.contains_key(&id) {
                    return Err(A::Error::custom("too many or duplicate mode filters"));
                }
                filters.insert(id, map.next_value()?);
            }
            Ok(filters)
        }
    }
    d.deserialize_map(Read)
}

/// Document-scoped presentation choices. Mode IDs do not refer to artwork or detail levels.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MapModes {
    pub active: u32,
    pub next_id: u32,
    #[serde(deserialize_with = "mode_names")]
    pub names: BTreeMap<u32, String>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty", deserialize_with = "mode_filters")]
    pub filters: BTreeMap<u32, MapFilter>,
}

fn mode_names<'de, D: Deserializer<'de>>(d: D) -> Result<BTreeMap<u32, String>, D::Error> {
    entries::<_, 128, 10, 256>(d)?
        .into_iter()
        .map(|(key, name)| {
            let id = key.parse::<u32>().map_err(D::Error::custom)?;
            if id.to_string() != key {
                return Err(D::Error::custom("noncanonical map mode ID"));
            }
            Ok((id, name))
        })
        .collect()
}

impl MapModes {
    pub const STARTER_NAMES: [&str; 6] = ["Geographic", "Biomes", "Political", "Climate", "Economy", "Infrastructure"];

    pub fn validate(&self) -> Result<(), &'static str> {
        if self.names.len() > 128
            || !self.names.contains_key(&self.active)
            || self.names.iter().any(|(&id, name)| id == 0 || id >= self.next_id || !text(name, 256))
        {
            return Err("invalid map modes");
        }
        for (id, filter) in &self.filters {
            if !self.names.contains_key(id) {
                return Err("map filter references an unknown mode");
            }
            filter.validate()?;
        }
        Ok(())
    }
}

impl Default for MapModes {
    fn default() -> Self {
        Self { active: 1, next_id: 7, names: (1..).zip(Self::STARTER_NAMES.map(str::to_owned)).collect(), filters: BTreeMap::new() }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MapSemantics {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub modes: Option<MapModes>,
    /// Feature guard: older semantic readers reject this unknown field rather than lose levels.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub detail_levels_version: Option<u32>,
    pub schema_version: u32,
    /// Own category UUID -> display label. Built-in categories are not duplicated here.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty", deserialize_with = "entries::<_, 1024, 64, 256>")]
    pub categories: BTreeMap<String, String>,
}
impl Default for MapSemantics {
    fn default() -> Self {
        Self { schema_version: SCHEMA_VERSION, categories: BTreeMap::new(), detail_levels_version: None, modes: None }
    }
}
impl MapSemantics {
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.detail_levels_version.is_some_and(|v| v != 1)
            || self.schema_version != SCHEMA_VERSION
            || self.categories.len() > 1024
            || self.categories.iter().any(|(id, label)| !user_category(id) || !text(label, 256))
        {
            return Err("unsupported or invalid map semantics schema/categories");
        }
        if let Some(modes) = &self.modes {
            modes.validate()?;
        }
        Ok(())
    }
}

impl crate::Document {
    /// Empty rules and older maps use the ordinary visibility policy.
    pub fn active_map_filter(&self) -> Option<&MapFilter> {
        self.world_map?;
        let modes = self.map_semantics.as_ref()?.modes.as_ref()?;
        modes.filters.get(&modes.active).filter(|filter| !filter.0.is_empty())
    }

    /// Validate the content-version guard and every stored geographic record, including library art.
    pub fn validate_geography(&self) -> Result<(), &'static str> {
        match (&self.map_semantics, self.world_map.and_then(|m| m.content_schema())) {
            (None, None) => {}
            (Some(s), Some(SCHEMA_VERSION)) => s.validate()?,
            _ => return Err("map semantics require a matching world-map content schema"),
        }
        let mut result = Ok(());
        let mut check = |n: &crate::Node| {
            if n.extras().detail_level && (!n.is_layer() || self.map_semantics.as_ref().and_then(|s| s.detail_levels_version) != Some(1)) {
                result = Err("detail levels require a layer and the detail-level schema guard");
            }
            if let Some(g) = &n.extras().geography
                && !g.is_empty()
            {
                result = result.and_then(|_| {
                    self.map_semantics.as_ref().ok_or("geography requires a world-map semantics schema")?;
                    g.validate()?;
                    Ok(())
                });
            }
        };
        for layer in &self.layers {
            crate::swatches::walk_with_masks(layer, &mut check);
        }
        for symbol in &self.symbols {
            crate::swatches::walk_with_masks(&symbol.art, &mut check);
        }
        for pattern in &self.patterns {
            for art in &pattern.art {
                crate::swatches::walk_with_masks(art, &mut check);
            }
        }
        for design in &self.graph_designs {
            crate::swatches::walk_with_masks(&design.art, &mut check);
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Document, Node, OpacityMask, WorldMap};
    const ENTITY: &str = "7a7eb4cc-b286-44dd-9949-4542254e5ee0";
    #[test]
    fn copying_masks_detaches_identity_and_ordinary_targets_remain_ordinary() {
        let mut d = Document::new(1000.0, 500.0);
        d.world_map = Some(WorldMap::standard());
        let mut mask = Node::group(d.alloc_id(), vec![]);
        mask.extras_mut().geography = Some(Box::new(GeographicMetadata {
            categories: vec!["core:land".into()],
            geographic_entity_id: Some(ENTITY.into()),
            ..Default::default()
        }));
        let mut original = Node::group(d.alloc_id(), vec![]);
        original.extras_mut().mask = Some(Box::new(OpacityMask::new(mask, true)));
        let copy = d.reid(&original);
        let copied_mask = &copy.extras().mask.as_ref().unwrap().art;
        let metadata = copied_mask.extras().geography.as_ref().unwrap();
        assert_eq!(metadata.categories, vec!["core:land"]);
        assert!(metadata.geographic_entity_id.is_none());
        assert_eq!(d.world_map.unwrap().content_schema(), Some(1));
        let mut ordinary = Document::new(100.0, 100.0);
        let copy = ordinary.reid(&original);
        assert!(copy.extras().mask.as_ref().unwrap().art.extras().geography.is_none());
        assert!(ordinary.world_map.is_none());
        assert!(ordinary.map_semantics.is_none());
        assert_eq!(original.extras().mask.as_ref().unwrap().art.extras().geography.as_ref().unwrap().geographic_entity_id.as_deref(), Some(ENTITY));
    }

    #[test]
    fn uuid_and_programmatic_validation_reject_invalid_values() {
        for id in ["", "00000000-0000-0000-0000-000000000000", "7A7EB4CC-b286-44dd-9949-4542254e5ee0", "7a7eb4cc-b286-44dd-9949-4542254e5ee!"] {
            assert!(!valid_entity_id(id));
        }
        assert!(valid_entity_id(ENTITY));
        let g = GeographicMetadata { categories: vec!["core:bogus".into()], ..Default::default() };
        assert!(g.validate().is_err());
        let mut d = Document::new(1000.0, 500.0);
        d.layers[0] = std::sync::Arc::new({
            let mut n = (*d.layers[0]).clone();
            n.extras_mut().geography = Some(Box::new(g));
            n
        });
        assert!(d.validate_geography().is_err());
    }
}
