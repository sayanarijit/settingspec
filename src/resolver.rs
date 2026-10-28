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
    /// The declaration produced a concrete value (or explicit null), along
    /// with the environment variable reference if it has an `env` directive.
    Value(SettingValue, Option<EnvRef>),
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
    let own_fallback = if decl.null {
        Some(EnvFallback::Null)
    } else {
        decl.val.clone().map(EnvFallback::Value)
    };

    if let Some(ref env_var) = decl.env {
        let env_ref = Some(EnvRef {
            name: env_var.clone(),
            fallback: own_fallback.clone(),
        });
        if let Some(env_val) = lookup_env(env_overrides, env_var) {
            DeclResolution::Value(
                SettingValue::Value(parse_env_value(&env_val, fallback_val)),
                env_ref,
            )
        } else if let Some(fb) = own_fallback {
            let value = match fb {
                EnvFallback::Null => SettingValue::Null,
                EnvFallback::Value(v) => SettingValue::Value(v),
            };
            DeclResolution::Value(value, env_ref)
        } else {
            DeclResolution::MissingEnv(env_var.clone())
        }
    } else if decl.null {
        DeclResolution::Value(SettingValue::Null, None)
    } else if let Some(ref v) = decl.val {
        DeclResolution::Value(SettingValue::Value(v.clone()), None)
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

        let mut final_val: Option<(SettingValue, Option<EnvRef>)> = None;

        // Step 1: Check active profile declaration. An `env` without a value in
        // that profile must fall back on the profile's own `val`/`null`, never
        // on the default declaration, so a missing variable is a hard error.
        if let Some(decl) = profile_decl {
            let fallback = decl
                .val
                .as_ref()
                .or_else(|| default_decl.and_then(|d| d.val.as_ref()));
            match resolve_declaration(decl, fallback, env_overrides) {
                DeclResolution::Value(v, env_ref) => final_val = Some((v, env_ref)),
                DeclResolution::MissingEnv(env_var) => {
                    return Err(SettingSpecError::MissingRequiredEnvVar(
                        key.clone(),
                        env_var,
                    ));
                }
                DeclResolution::None => {}
            }
        }

        // Step 2: Fallback to default declaration if active profile did not produce a value
        if final_val.is_none() {
            if let Some(decl) = default_decl {
                match resolve_declaration(decl, decl.val.as_ref(), env_overrides) {
                    DeclResolution::Value(v, env_ref) => final_val = Some((v, env_ref)),
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

        let tags = setting.tags.clone();
        let shape = type_shape(setting);

        if let Some((val, env_ref)) = final_val {
            resolved.push(ResolvedSetting {
                key: key.clone(),
                value: val,
                tags,
                env_ref,
                shape,
            });
        }
    }

    Ok(resolved)
}

fn type_shape(setting: &Setting) -> TypeShape {
    let mut kinds = Vec::new();
    for decl in setting.profiles.values() {
        if let Some(ref v) = decl.val {
            kinds.push(ValueKind::of(v));
        }
        if decl.env.is_some() {
            kinds.push(ValueKind::String);
        }
    }
    kinds.sort();
    kinds.dedup();
    if kinds.is_empty() {
        kinds.push(ValueKind::String);
    }
    TypeShape {
        kinds,
        nullable: setting.profiles.values().any(|d| d.null),
        has_env: setting.profiles.values().any(|d| d.env.is_some()),
    }
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
