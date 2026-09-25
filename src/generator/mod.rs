pub mod common;
pub mod env;
pub mod javascript;
pub mod json;
pub mod lua;
pub mod python;
pub mod toml;
pub mod tree;
pub mod typescript;
pub mod yaml;

use crate::error::{Result, SettingSpecError};
use crate::model::ResolvedSetting;
use std::path::Path;

pub fn generate_by_path(path: &Path, settings: &[ResolvedSetting]) -> Result<String> {
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
        return env::generate_env(settings);
    }

    match ext.as_str() {
        "toml" => toml::generate_toml(settings),
        "json" => json::generate_json(settings),
        "yaml" | "yml" => yaml::generate_yaml(settings),
        "py" => python::generate_python(settings),
        "js" | "mjs" => javascript::generate_javascript(settings),
        "ts" => typescript::generate_typescript(settings),
        "lua" => lua::generate_lua(settings),
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

pub fn generate_by_format(format_name: &str, settings: &[ResolvedSetting]) -> Result<String> {
    match format_name.to_lowercase().as_str() {
        "toml" => toml::generate_toml(settings),
        "json" => json::generate_json(settings),
        "yaml" | "yml" => yaml::generate_yaml(settings),
        "py" | "python" => python::generate_python(settings),
        "js" | "mjs" | "javascript" => javascript::generate_javascript(settings),
        "ts" | "typescript" => typescript::generate_typescript(settings),
        "lua" => lua::generate_lua(settings),
        "env" => env::generate_env(settings),
        other => Err(SettingSpecError::InvalidExportFormat(other.to_string())),
    }
}
