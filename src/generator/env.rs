use crate::error::Result;
use crate::model::{ResolvedSetting, SettingValue};
use toml::Value as TomlValue;

pub fn format_env_key(key: &str) -> String {
    key.replace('.', "_").to_uppercase()
}

pub fn format_env_value(val: &TomlValue) -> String {
    match val {
        TomlValue::String(s) => s.clone(),
        TomlValue::Integer(i) => i.to_string(),
        TomlValue::Float(f) => f.to_string(),
        TomlValue::Boolean(b) => b.to_string(),
        TomlValue::Datetime(dt) => dt.to_string(),
        TomlValue::Array(_) | TomlValue::Table(_) => {
            serde_json::to_string(&crate::generator::json::toml_to_json_value(val)).unwrap()
        }
    }
}

pub fn generate_env(settings: &[ResolvedSetting]) -> Result<String> {
    let mut out = String::new();

    for setting in settings {
        if setting.value == SettingValue::Null {
            continue;
        }

        let key = format_env_key(&setting.key);
        if let SettingValue::Value(ref val) = setting.value {
            let val_str = match val {
                TomlValue::String(s) => {
                    if s.contains(' ') || s.contains('"') || s.contains('\n') {
                        serde_json::to_string(s).unwrap()
                    } else {
                        s.clone()
                    }
                }
                _ => format_env_value(val),
            };
            out.push_str(&format!("{}={}\n", key, val_str));
        }
    }

    Ok(out)
}
