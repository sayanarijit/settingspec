pub mod c;
pub mod common;
pub mod cpp;
pub mod elm;
pub mod env;
pub mod go;
pub mod haskell;
pub mod java;
pub mod javascript;
pub mod json;
pub mod lua;
pub mod python;
pub mod ruby;
pub mod rust;
pub mod scala;
pub mod terraform;
pub mod toml;
pub mod tree;
pub mod typescript;
pub mod yaml;
pub mod zig;

use crate::error::{Result, SettingSpecError};
use crate::model::{ResolvedSetting, SettingValue, ValueKind};
use std::path::Path;

pub fn generate_by_path(
    path: &Path,
    settings: &[ResolvedSetting],
    resolve: bool,
) -> Result<String> {
    let file_name = path.file_name().and_then(|f| f.to_str()).unwrap_or("");
    let lower_file_name = file_name.to_lowercase();

    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    if lower_file_name == ".env"
        || lower_file_name == "env"
        || lower_file_name.starts_with(".env.")
        || lower_file_name.starts_with("env.")
        || ext == "env"
    {
        return generate_by_format("env", settings, resolve);
    }

    match ext.as_str() {
        "toml" | "json" | "yaml" | "yml" | "py" | "js" | "mjs" | "ts" | "lua" | "rs" | "go"
        | "zig" | "c" | "h" | "cpp" | "cc" | "cxx" | "hpp" | "java" | "elm" | "rb" | "scala"
        | "hs" | "tf" | "tfvars" => generate_by_format(&ext, settings, resolve),
        _ => {
            let desc = if ext.is_empty() {
                path.to_string_lossy().to_string()
            } else {
                format!(".{}", ext)
            };
            Err(SettingSpecError::InvalidExportFormat(desc))
        }
    }
}

/// Generates `format_name` output. With `resolve == false`, settings that came
/// from an `env` directive are written as references to the environment
/// variable instead of its resolved value.
pub fn generate_by_format(
    format_name: &str,
    settings: &[ResolvedSetting],
    resolve: bool,
) -> Result<String> {
    let format = format_name.to_lowercase();
    // Python is not statically typed, but its settings carry type hints.
    let typed = is_statically_typed(&format) || matches!(format.as_str(), "py" | "python");

    let prepared = settings
        .iter()
        .map(|s| prepare_setting(&format, s, resolve, typed))
        .collect::<Result<Vec<_>>>()?;
    generate_resolved(&format, &prepared)
}

/// Applies `resolve` to the setting, and for statically typed formats gives
/// its value the type shared by all profiles of the setting.
fn prepare_setting(
    format: &str,
    s: &ResolvedSetting,
    resolve: bool,
    typed: bool,
) -> Result<ResolvedSetting> {
    // Only dynamically typed formats can embed a fallback next to the reference.
    // Elm and Terraform reject environment references on their own.
    if !resolve
        && s.env_ref.as_ref().is_some_and(|r| r.fallback.is_some())
        && !matches!(
            format,
            "py" | "python"
                | "js"
                | "mjs"
                | "javascript"
                | "rb"
                | "ruby"
                | "lua"
                | "elm"
                | "tf"
                | "tfvars"
                | "terraform"
        )
    {
        return Err(SettingSpecError::EnvFallbackUnsupported(
            format.to_string(),
            s.key.clone(),
        ));
    }
    let mut value = match &s.env_ref {
        Some(r) if !resolve => SettingValue::Env(r.clone()),
        _ => s.value.clone(),
    };
    if !typed {
        return Ok(ResolvedSetting { value, ..s.clone() });
    }

    // An environment variable parsed as a kind outside the type is text.
    if let SettingValue::Value(v) = &value
        && !s.shape.kinds.contains(&ValueKind::of(v))
    {
        value = SettingValue::Value(::toml::Value::String(env::format_env_value(v)));
    }

    let plain = s.shape.kinds.len() == 1 && !s.shape.nullable && !(s.shape.has_env && !resolve);
    let hinted = matches!(format, "py" | "python");
    if !plain || hinted {
        let composite = s
            .shape
            .kinds
            .iter()
            .any(|k| matches!(k, ValueKind::Array | ValueKind::Table));
        let only_strings = s.shape.kinds == [ValueKind::String];
        let supported = match format {
            "rs" | "rust" | "zig" | "cpp" | "c++" | "cxx" | "cc" | "hpp" | "hs" | "haskell"
            | "elm" => !composite,
            // C can only point at string literals or call getters of the
            // environment.
            "c" | "h" => {
                only_strings
                    && !(s.shape.has_env && !resolve && !matches!(value, SettingValue::Env(_)))
            }
            _ => true,
        };
        if !supported {
            return Err(SettingSpecError::UnsupportedType(
                format.to_string(),
                s.key.clone(),
            ));
        }
        value = SettingValue::Typed(s.shape.clone(), Box::new(value));
    }
    Ok(ResolvedSetting { value, ..s.clone() })
}

fn is_statically_typed(format: &str) -> bool {
    matches!(
        format,
        "ts" | "typescript"
            | "rs"
            | "rust"
            | "go"
            | "golang"
            | "zig"
            | "c"
            | "h"
            | "cpp"
            | "c++"
            | "cxx"
            | "cc"
            | "hpp"
            | "java"
            | "elm"
            | "scala"
            | "hs"
            | "haskell"
    )
}

fn generate_resolved(format: &str, settings: &[ResolvedSetting]) -> Result<String> {
    match format {
        "toml" => toml::generate_toml(settings),
        "json" => json::generate_json(settings),
        "yaml" | "yml" => yaml::generate_yaml(settings),
        "py" | "python" => python::generate_python(settings),
        "js" | "mjs" | "javascript" => javascript::generate_javascript(settings),
        "ts" | "typescript" => typescript::generate_typescript(settings),
        "lua" => lua::generate_lua(settings),
        "env" => env::generate_env(settings),
        "rs" | "rust" => rust::generate_rust(settings),
        "go" | "golang" => go::generate_go(settings),
        "zig" => zig::generate_zig(settings),
        "c" | "h" => c::generate_c(settings),
        "cpp" | "c++" | "cxx" | "cc" | "hpp" => cpp::generate_cpp(settings),
        "java" => java::generate_java(settings),
        "elm" => elm::generate_elm(settings),
        "rb" | "ruby" => ruby::generate_ruby(settings),
        "scala" => scala::generate_scala(settings),
        "hs" | "haskell" => haskell::generate_haskell(settings),
        "tf" | "tfvars" | "terraform" => terraform::generate_terraform(settings),
        other => Err(SettingSpecError::InvalidExportFormat(other.to_string())),
    }
}
