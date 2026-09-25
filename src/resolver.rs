use crate::error::{Result, SettingSpecError};
use crate::model::*;
use std::collections::BTreeMap;
use std::collections::HashMap;
use toml::Value as TomlValue;

/// Looks up an environment variable, preferring the sourced envfile
/// overrides over the real process environment. This lets envfile-sourced
/// values take precedence without mutating global process state.
fn lookup_env(env_overrides: &HashMap<String, String>, key: &str) -> Option<String> {
    env_overrides
        .get(key)
        .cloned()
        .or_else(|| std::env::var(key).ok())
}

/// Outcome of resolving a single profile declaration to a setting value.
enum DeclResolution {
    /// The declaration produced a concrete value (or explicit null).
    Value(SettingValue),
    /// The declaration requires an environment variable that is not set,
    /// and has no `null`/`val` fallback of its own.
    MissingEnv(String),
    /// The declaration has neither `env`, `null`, nor `val`.
    None,
}

/// Resolves a profile declaration using the standard precedence: environment
/// variable override (parsed using `fallback_val` for type inference), then
/// explicit `null`, then explicit `val`.
fn resolve_declaration(
    decl: &ProfileDeclaration,
    fallback_val: Option<&TomlValue>,
    env_overrides: &HashMap<String, String>,
) -> DeclResolution {
    if let Some(ref env_var) = decl.env {
        if let Some(env_val) = lookup_env(env_overrides, env_var) {
            DeclResolution::Value(SettingValue::Value(parse_env_value(&env_val, fallback_val)))
        } else if decl.null {
            DeclResolution::Value(SettingValue::Null)
        } else if let Some(ref v) = decl.val {
            DeclResolution::Value(SettingValue::Value(v.clone()))
        } else {
            DeclResolution::MissingEnv(env_var.clone())
        }
    } else if decl.null {
        DeclResolution::Value(SettingValue::Null)
    } else if let Some(ref v) = decl.val {
        DeclResolution::Value(SettingValue::Value(v.clone()))
    } else {
        DeclResolution::None
    }
}

pub fn resolve_settings(
    settings: &BTreeMap<String, Setting>,
    spec: &SpecConfig,
    active_profile: Option<&str>,
    env_overrides: &HashMap<String, String>,
) -> Result<Vec<ResolvedSetting>> {
    let mut resolved = Vec::new();

    for (key, setting) in settings {
        let profile_decl = active_profile.and_then(|p| setting.profiles.get(p));
        let default_decl = setting.profiles.get("default");

        let mut final_val: Option<SettingValue> = None;
        let mut missing_env_error: Option<String> = None;

        // Step 1: Check active profile declaration
        if let Some(decl) = profile_decl {
            let fallback = decl
                .val
                .as_ref()
                .or_else(|| default_decl.and_then(|d| d.val.as_ref()));
            match resolve_declaration(decl, fallback, env_overrides) {
                DeclResolution::Value(v) => final_val = Some(v),
                DeclResolution::MissingEnv(env_var) => missing_env_error = Some(env_var),
                DeclResolution::None => {}
            }
        }

        // Step 2: Fallback to default declaration if active profile did not produce a value
        if final_val.is_none() {
            if let Some(decl) = default_decl {
                match resolve_declaration(decl, decl.val.as_ref(), env_overrides) {
                    DeclResolution::Value(v) => final_val = Some(v),
                    DeclResolution::MissingEnv(env_var) => {
                        return Err(SettingSpecError::MissingRequiredEnvVar(
                            key.clone(),
                            env_var,
                        ));
                    }
                    DeclResolution::None => {
                        return Err(SettingSpecError::RequiredValueMissing(key.clone()));
                    }
                }
            } else if let Some(env_var) = missing_env_error {
                return Err(SettingSpecError::MissingRequiredEnvVar(
                    key.clone(),
                    env_var,
                ));
            } else if !spec.profile.options.is_empty() {
                if let Some(p) = active_profile {
                    return Err(SettingSpecError::StrictCoverageFailed(
                        key.clone(),
                        p.to_string(),
                    ));
                } else {
                    return Err(SettingSpecError::RequiredValueMissing(key.clone()));
                }
            } else {
                // Key omitted when options not defined and default absent
                continue;
            }
        }

        let tags = if let Some(decl) = profile_decl {
            if !decl.tags.is_empty() {
                decl.tags.clone()
            } else if let Some(def) = default_decl {
                def.tags.clone()
            } else {
                Vec::new()
            }
        } else if let Some(def) = default_decl {
            def.tags.clone()
        } else {
            Vec::new()
        };

        if let Some(val) = final_val {
            resolved.push(ResolvedSetting {
                key: key.clone(),
                value: val,
                tags,
            });
        }
    }

    Ok(resolved)
}

fn parse_env_value(raw: &str, fallback_val: Option<&TomlValue>) -> TomlValue {
    if let Some(fb) = fallback_val {
        match fb {
            TomlValue::Integer(_) => {
                if let Ok(i) = raw.parse::<i64>() {
                    return TomlValue::Integer(i);
                }
            }
            TomlValue::Float(_) => {
                if let Ok(f) = raw.parse::<f64>() {
                    return TomlValue::Float(f);
                }
            }
            TomlValue::Boolean(_) => {
                if let Ok(b) = raw.parse::<bool>() {
                    return TomlValue::Boolean(b);
                }
            }
            TomlValue::Array(_) | TomlValue::Table(_) => {
                if let Ok(val) = toml::from_str::<TomlValue>(raw) {
                    return val;
                }
            }
            TomlValue::String(_) => {
                return TomlValue::String(raw.to_string());
            }
            TomlValue::Datetime(_) => {
                if let Ok(dt) = raw.parse::<toml::value::Datetime>() {
                    return TomlValue::Datetime(dt);
                }
            }
        }
    }

    if raw == "true" {
        TomlValue::Boolean(true)
    } else if raw == "false" {
        TomlValue::Boolean(false)
    } else if let Ok(i) = raw.parse::<i64>() {
        TomlValue::Integer(i)
    } else if raw.contains('.') && raw.parse::<f64>().is_ok() {
        TomlValue::Float(raw.parse::<f64>().unwrap())
    } else {
        TomlValue::String(raw.to_string())
    }
}
