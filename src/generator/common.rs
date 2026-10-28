use crate::model::{ResolvedSetting, SettingValue};
use toml::Value as TomlValue;

/// Returns true if `name` can be used as-is as an identifier (e.g. a Python
/// attribute name, a Lua table key, or a TypeScript object key) without
/// quoting or escaping.
pub fn is_valid_identifier(name: &str) -> bool {
    !name.is_empty()
        && !name.chars().next().unwrap().is_ascii_digit()
        && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
}

/// Formats the scalar TOML variants shared by C-like literal syntaxes:
/// JSON-compatible quoted strings, integers, and floats that always include
/// a decimal point. Returns `None` for booleans, arrays, and tables, which
/// differ enough between target languages that callers must format them
/// themselves.
pub fn format_scalar_literal(v: &TomlValue) -> Option<String> {
    match v {
        TomlValue::String(s) => Some(serde_json::to_string(s).unwrap()),
        TomlValue::Integer(i) => Some(i.to_string()),
        TomlValue::Float(f) => {
            let mut s = f.to_string();
            if !s.contains('.') {
                s.push_str(".0");
            }
            Some(s)
        }
        TomlValue::Datetime(dt) => Some(serde_json::to_string(&dt.to_string()).unwrap()),
        TomlValue::Boolean(_) | TomlValue::Array(_) | TomlValue::Table(_) => None,
    }
}

/// Returns true if any setting is exported as an environment variable
/// reference.
pub fn has_env_refs(settings: &[ResolvedSetting]) -> bool {
    fn is_env(v: &SettingValue) -> bool {
        match v {
            SettingValue::Env(_) => true,
            SettingValue::Typed(_, inner) => is_env(inner),
            _ => false,
        }
    }
    settings.iter().any(|s| is_env(&s.value))
}

/// A double-quoted string literal for an environment variable name.
pub fn env_name_literal(name: &str) -> String {
    serde_json::to_string(name).unwrap()
}

/// The unwrapped contents of a `ConfigNode::Typed`.
pub enum Leaf<'a> {
    Null,
    Value(&'a TomlValue),
    Env(&'a crate::model::EnvRef),
}

/// Returns the leaf of a typed node, or `None` for a node that is not a leaf
/// (a map).
pub fn leaf_of(node: &super::tree::ConfigNode) -> Option<Leaf<'_>> {
    use super::tree::ConfigNode;
    match node {
        ConfigNode::Null => Some(Leaf::Null),
        ConfigNode::Value(v) => Some(Leaf::Value(v)),
        ConfigNode::Env(r) => Some(Leaf::Env(r)),
        ConfigNode::Typed(_, inner) => leaf_of(inner),
        ConfigNode::Map(_) => None,
    }
}

/// The single kind of a shape that has exactly one, if any.
pub fn single_kind(shape: &crate::model::TypeShape) -> Option<crate::model::ValueKind> {
    match shape.kinds.as_slice() {
        [k] => Some(*k),
        _ => None,
    }
}

/// Name used for a kind in generated sum types.
pub fn kind_name(kind: crate::model::ValueKind) -> &'static str {
    use crate::model::ValueKind::*;
    match kind {
        String => "Str",
        Integer => "Int",
        Float => "Float",
        Boolean => "Bool",
        Array => "List",
        Table => "Map",
    }
}
