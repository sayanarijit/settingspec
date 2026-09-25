use super::tree::{ConfigNode, build_config_tree};
use crate::error::Result;
use crate::model::ResolvedSetting;
use toml::Value as TomlValue;

pub fn generate_toml(settings: &[ResolvedSetting]) -> Result<String> {
    let tree = build_config_tree(settings, true); // omit nulls
    let mut root_table = toml::map::Map::new();

    for (k, node) in tree {
        if let Some(v) = node_to_toml_value(&node) {
            root_table.insert(k, v);
        }
    }

    let toml_val = TomlValue::Table(root_table);
    let serialized = toml::to_string_pretty(&toml_val)
        .map_err(|e| crate::error::SettingSpecError::InvalidToml(e.to_string()))?;
    Ok(serialized)
}

fn node_to_toml_value(node: &ConfigNode) -> Option<TomlValue> {
    match node {
        ConfigNode::Null => None,
        ConfigNode::Value(v) => Some(v.clone()),
        ConfigNode::Map(map) => {
            let mut sub_table = toml::map::Map::new();
            for (k, sub_node) in map {
                if let Some(sub_val) = node_to_toml_value(sub_node) {
                    sub_table.insert(k.clone(), sub_val);
                }
            }
            Some(TomlValue::Table(sub_table))
        }
    }
}
