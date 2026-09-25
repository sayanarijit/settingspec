use crate::model::{ResolvedSetting, SettingValue};
use indexmap::IndexMap;
use toml::Value as TomlValue;

#[derive(Debug, Clone, PartialEq)]
pub enum ConfigNode {
    Null,
    Value(TomlValue),
    Map(IndexMap<String, ConfigNode>),
}

impl ConfigNode {
    pub fn is_empty_map(&self) -> bool {
        match self {
            ConfigNode::Map(m) => m.is_empty(),
            _ => false,
        }
    }
}

pub fn build_config_tree(
    settings: &[ResolvedSetting],
    omit_nulls: bool,
) -> IndexMap<String, ConfigNode> {
    let mut root = IndexMap::new();

    for setting in settings {
        if omit_nulls && setting.value == SettingValue::Null {
            continue;
        }

        let parts: Vec<&str> = setting.key.split('.').collect();
        insert_into_tree(&mut root, &parts, &setting.value);
    }

    root
}

fn insert_into_tree(root: &mut IndexMap<String, ConfigNode>, parts: &[&str], value: &SettingValue) {
    if parts.is_empty() {
        return;
    }

    let head = parts[0];
    if parts.len() == 1 {
        let node = match value {
            SettingValue::Null => ConfigNode::Null,
            SettingValue::Value(v) => ConfigNode::Value(v.clone()),
        };
        root.insert(head.to_string(), node);
    } else {
        let sub = root
            .entry(head.to_string())
            .or_insert_with(|| ConfigNode::Map(IndexMap::new()));
        if let ConfigNode::Map(sub_map) = sub {
            insert_into_tree(sub_map, &parts[1..], value);
        }
    }
}
