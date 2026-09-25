use super::tree::{ConfigNode, build_config_tree};
use crate::error::Result;
use crate::model::ResolvedSetting;
use serde_json::{Map, Value as JsonValue};

pub fn generate_json(settings: &[ResolvedSetting]) -> Result<String> {
    let tree = build_config_tree(settings, false); // keep nulls
    let mut root_obj = Map::new();

    for (k, node) in tree {
        root_obj.insert(k, node_to_json_value(&node));
    }

    let json_val = JsonValue::Object(root_obj);
    let serialized = serde_json::to_string_pretty(&json_val)
        .map_err(|e| crate::error::SettingSpecError::IoError(e.to_string()))?;
    Ok(serialized)
}

fn node_to_json_value(node: &ConfigNode) -> JsonValue {
    match node {
        ConfigNode::Null => JsonValue::Null,
        ConfigNode::Value(v) => toml_to_json_value(v),
        ConfigNode::Map(map) => {
            let mut obj = Map::new();
            for (k, sub_node) in map {
                obj.insert(k.clone(), node_to_json_value(sub_node));
            }
            JsonValue::Object(obj)
        }
    }
}

pub fn toml_to_json_value(v: &toml::Value) -> JsonValue {
    match v {
        toml::Value::String(s) => JsonValue::String(s.clone()),
        toml::Value::Integer(i) => JsonValue::Number((*i).into()),
        toml::Value::Float(f) => serde_json::Number::from_f64(*f)
            .map(JsonValue::Number)
            .unwrap_or(JsonValue::Null),
        toml::Value::Boolean(b) => JsonValue::Bool(*b),
        toml::Value::Datetime(dt) => JsonValue::String(dt.to_string()),
        toml::Value::Array(arr) => JsonValue::Array(arr.iter().map(toml_to_json_value).collect()),
        toml::Value::Table(tbl) => {
            let mut obj = Map::new();
            for (k, v) in tbl {
                obj.insert(k.clone(), toml_to_json_value(v));
            }
            JsonValue::Object(obj)
        }
    }
}
