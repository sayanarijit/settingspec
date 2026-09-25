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
