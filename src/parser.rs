use crate::error::{Result, SettingSpecError};
use crate::model::*;
use serde::Deserialize;
use serde::de::IntoDeserializer;
use std::collections::BTreeMap;
use toml::Value as TomlValue;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SettingSpecSchema {
    #[serde(default)]
    pub spec: Option<SpecConfig>,
    pub settings: Option<toml::Table>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProfileDeclarationSchema {
    #[serde(default)]
    pub val: Option<TomlValue>,
    #[serde(default)]
    pub env: Option<String>,
    #[serde(default)]
    pub null: Option<bool>,
    #[serde(default)]
    pub tags: Option<Vec<String>>,
}

pub fn parse_config_str(content: &str) -> Result<SettingSpecDocument> {
    let schema: SettingSpecSchema =
        toml::from_str(content).map_err(|e| SettingSpecError::InvalidToml(e.to_string()))?;

    let spec = schema.spec.unwrap_or_default();

    // Validate spec.profile.options does not contain "default"
    if spec.profile.options.iter().any(|o| o == "default") {
        return Err(SettingSpecError::ReservedProfileInOptions);
    }

    // Parse [settings] (REQUIRED)
    let settings_table = schema
        .settings
        .ok_or(SettingSpecError::MissingSettingsSection)?;

    if settings_table.is_empty() {
        return Err(SettingSpecError::MissingSettingsSection);
    }

    let mut settings: BTreeMap<String, Setting> = BTreeMap::new();
    let mut current_path = Vec::new();
    extract_settings(
        &settings_table,
        &mut current_path,
        &spec.profile.options,
        &mut settings,
    )?;

    if settings.is_empty() {
        return Err(SettingSpecError::MissingSettingsSection);
    }

    // Static validation: Strict Coverage Rule
    if !spec.profile.options.is_empty() {
        for (key, setting) in &settings {
            if !setting.profiles.contains_key("default") {
                for opt in &spec.profile.options {
                    if !setting.profiles.contains_key(opt) {
                        return Err(SettingSpecError::StrictCoverageFailed(
                            key.clone(),
                            opt.clone(),
                        ));
                    }
                }
            }
        }
    }

    Ok(SettingSpecDocument { spec, settings })
}

fn extract_settings(
    table: &toml::map::Map<String, TomlValue>,
    current_path: &mut Vec<String>,
    profile_options: &[String],
    settings: &mut BTreeMap<String, Setting>,
) -> Result<()> {
    let has_directive = table
        .keys()
        .any(|k| matches!(k.as_str(), "val" | "env" | "null" | "tags"));
    let last_is_profile = if let Some(last) = current_path.last() {
        last == "default" || profile_options.contains(last)
    } else {
        false
    };

    if has_directive
        || (last_is_profile && current_path.len() >= 2 && !table.values().any(|v| v.is_table()))
    {
        // This is a profile declaration!
        if current_path.len() < 2 {
            return Err(SettingSpecError::InvalidToml(
                "Malformed setting declaration: missing setting key or profile".into(),
            ));
        }

        let profile = current_path.pop().unwrap();
        let setting_key = current_path.join(".");
        current_path.push(profile.clone());

        // Validate profile name
        if !profile_options.is_empty()
            && profile != "default"
            && !profile_options.contains(&profile)
        {
            return Err(SettingSpecError::UnknownProfileInSettings(profile));
        }

        // Validate directives: check for unknown directives
        for k in table.keys() {
            if !matches!(k.as_str(), "val" | "env" | "null" | "tags") {
                return Err(SettingSpecError::InvalidDirective(
                    setting_key,
                    profile,
                    k.clone(),
                ));
            }
        }

        // Deserialize profile declaration schema using Serde
        let decl_schema: ProfileDeclarationSchema =
            ProfileDeclarationSchema::deserialize(table.clone().into_deserializer())
                .map_err(|e| SettingSpecError::InvalidToml(e.to_string()))?;

        if let Some(null_val) = decl_schema.null
            && !null_val
        {
            return Err(SettingSpecError::InvalidNullValue(setting_key, profile));
        }

        if decl_schema.null == Some(true) && decl_schema.val.is_some() {
            return Err(SettingSpecError::NullCoexistsWithVal(setting_key, profile));
        }

        if decl_schema.val.is_none() && decl_schema.env.is_none() && decl_schema.null != Some(true)
        {
            return Err(SettingSpecError::RequiredValueMissing(format!(
                "{}.{}",
                setting_key, profile
            )));
        }

        let decl = ProfileDeclaration {
            val: table.get("val").cloned(),
            env: decl_schema.env,
            null: decl_schema.null.unwrap_or(false),
            tags: decl_schema.tags.unwrap_or_default(),
        };

        let entry = settings
            .entry(setting_key.clone())
            .or_insert_with(|| Setting {
                key: setting_key,
                profiles: BTreeMap::new(),
            });
        entry.profiles.insert(profile, decl);
        return Ok(());
    }

    // Otherwise, descend into child tables
    for (k, v) in table {
        if let Some(sub_table) = v.as_table() {
            current_path.push(k.clone());
            extract_settings(sub_table, current_path, profile_options, settings)?;
            current_path.pop();
        } else {
            let mut full = current_path.clone();
            full.push(k.clone());
            return Err(SettingSpecError::InvalidToml(format!(
                "Invalid setting structure at '{}': expected profile sub-table",
                full.join(".")
            )));
        }
    }

    Ok(())
}
