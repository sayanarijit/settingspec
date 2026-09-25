use super::tree::{ConfigNode, build_config_tree};
use crate::error::Result;
use crate::model::ResolvedSetting;
use serde_yaml::Value as YamlValue;

pub fn generate_yaml(settings: &[ResolvedSetting]) -> Result<String> {
    let tree = build_config_tree(settings, false); // keep nulls
    let mut root_map = serde_yaml::Mapping::new();

    for (k, node) in tree {
        root_map.insert(YamlValue::String(k), node_to_yaml_value(&node));
    }

    let yaml_val = YamlValue::Mapping(root_map);
    let serialized = serde_yaml::to_string(&yaml_val)
        .map_err(|e| crate::error::SettingSpecError::IoError(e.to_string()))?;
    Ok(serialized)
}

fn node_to_yaml_value(node: &ConfigNode) -> YamlValue {
    match node {
        ConfigNode::Null => YamlValue::Null,
        ConfigNode::Value(v) => toml_to_yaml_value(v),
        ConfigNode::Map(map) => {
            let mut mapping = serde_yaml::Mapping::new();
            for (k, sub_node) in map {
                mapping.insert(YamlValue::String(k.clone()), node_to_yaml_value(sub_node));
            }
            YamlValue::Mapping(mapping)
        }
    }
}

fn toml_to_yaml_value(v: &toml::Value) -> YamlValue {
    match v {
        toml::Value::String(s) => YamlValue::String(s.clone()),
        toml::Value::Integer(i) => YamlValue::Number((*i).into()),
        toml::Value::Float(f) => YamlValue::Number(serde_yaml::Number::from(*f)),
        toml::Value::Boolean(b) => YamlValue::Bool(*b),
        toml::Value::Datetime(dt) => YamlValue::String(dt.to_string()),
        toml::Value::Array(arr) => {
            YamlValue::Sequence(arr.iter().map(toml_to_yaml_value).collect())
        }
        toml::Value::Table(tbl) => {
            let mut mapping = serde_yaml::Mapping::new();
            for (k, v) in tbl {
                mapping.insert(YamlValue::String(k.clone()), toml_to_yaml_value(v));
            }
            YamlValue::Mapping(mapping)
        }
    }
}
