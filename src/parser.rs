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

    // Validate spec.profile.options
    for opt in &spec.profile.options {
        if opt == "default" {
            return Err(SettingSpecError::ReservedProfileInOptions);
        }
        if is_reserved_directive_keyword(opt) {
            return Err(SettingSpecError::ReservedKeywordConflict(opt.clone()));
        }
    }

    if let Some(ref def) = spec.profile.default
        && is_reserved_directive_keyword(def)
    {
        return Err(SettingSpecError::ReservedKeywordConflict(def.clone()));
    }

    for prof in spec.envfile.profiles.keys() {
        if is_reserved_directive_keyword(prof) {
            return Err(SettingSpecError::ReservedKeywordConflict(prof.clone()));
        }
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

fn has_known_directive(table: &toml::map::Map<String, TomlValue>) -> bool {
    table.contains_key("val")
        || table.get("env").is_some_and(|v| !v.is_table())
        || table.get("null").is_some_and(|v| !v.is_table())
        || table.get("tags").is_some_and(|v| !v.is_table())
}

fn is_profile_declaration_table(
    table: &toml::map::Map<String, TomlValue>,
    current_path: &[String],
    profile_options: &[String],
) -> bool {
    if current_path.len() < 2 {
        return false;
    }

    if table.contains_key("default") || table.keys().any(|k| profile_options.contains(k)) {
        return false;
    }

    let last_segment = current_path.last().unwrap();

    if last_segment == "default" || profile_options.contains(last_segment) {
        return has_known_directive(table)
            || (!table.is_empty() && !table.values().any(|v| v.is_table()));
    }

    if let Some(val_entry) = table.get("val") {
        if let Some(sub_table) = val_entry.as_table()
            && has_known_directive(sub_table)
        {
            return false;
        }
        return true;
    }

    if table.get("env").is_some_and(|v| !v.is_table())
        || table.get("null").is_some_and(|v| !v.is_table())
        || table.get("tags").is_some_and(|v| !v.is_table())
    {
        return true;
    }

    if !table.is_empty() && !table.values().any(|v| v.is_table()) {
        return true;
    }

    false
}

fn extract_settings(
    table: &toml::map::Map<String, TomlValue>,
    current_path: &mut Vec<String>,
    profile_options: &[String],
    settings: &mut BTreeMap<String, Setting>,
) -> Result<()> {
    if is_profile_declaration_table(table, current_path, profile_options) {
        let profile = current_path.last().unwrap().clone();
        let key_segments = &current_path[..current_path.len() - 1];
        let setting_key = key_segments.join(".");

        // 1. Validate setting key segments for reserved keywords
        for seg in key_segments {
            if is_reserved_directive_keyword(seg) {
                return Err(SettingSpecError::ReservedKeywordConflict(seg.clone()));
            }
        }

        // 2. Validate profile name for reserved keywords
        if is_reserved_directive_keyword(&profile) {
            return Err(SettingSpecError::ReservedKeywordConflict(profile));
        }

        // 3. Validate profile options membership
        if !profile_options.is_empty()
            && profile != "default"
            && !profile_options.contains(&profile)
        {
            return Err(SettingSpecError::UnknownProfileInSettings(profile));
        }

        // 4. Validate directives: check for unknown directives
        for k in table.keys() {
            if !is_reserved_directive_keyword(k) {
                return Err(SettingSpecError::InvalidDirective(
                    setting_key,
                    profile,
                    k.clone(),
                ));
            }
        }

        // 5. Deserialize profile declaration schema using Serde
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_reserved_directive_keywords_and_default_in_table_values() {
        let toml_str = r#"
[settings]
key1.default.val = { val = 1 }
key2.default.val = { null = true }
key3.default.val = { tags = [1, 2, 3] }
key4.default.val = { env = "MY_VAR" }
key5.default.val = { default = { val = 1 } }
"#;

        let doc = parse_config_str(toml_str).expect("Failed to parse config");

        // key1: key1.default.val = { val = 1 }
        let key1_decl = doc
            .settings
            .get("key1")
            .unwrap()
            .profiles
            .get("default")
            .unwrap();
        assert!(!key1_decl.null);
        assert!(key1_decl.env.is_none());
        assert!(key1_decl.tags.is_empty());
        let val1 = key1_decl.val.as_ref().unwrap().as_table().unwrap();
        assert_eq!(val1.get("val").unwrap().as_integer(), Some(1));

        // key2: key2.default.val = { null = true }
        let key2_decl = doc
            .settings
            .get("key2")
            .unwrap()
            .profiles
            .get("default")
            .unwrap();
        assert!(!key2_decl.null);
        assert!(key2_decl.env.is_none());
        assert!(key2_decl.tags.is_empty());
        let val2 = key2_decl.val.as_ref().unwrap().as_table().unwrap();
        assert_eq!(val2.get("null").unwrap().as_bool(), Some(true));

        // key3: key3.default.val = { tags = [1, 2, 3] }
        let key3_decl = doc
            .settings
            .get("key3")
            .unwrap()
            .profiles
            .get("default")
            .unwrap();
        assert!(!key3_decl.null);
        assert!(key3_decl.env.is_none());
        assert!(key3_decl.tags.is_empty());
        let val3 = key3_decl.val.as_ref().unwrap().as_table().unwrap();
        let tags = val3.get("tags").unwrap().as_array().unwrap();
        assert_eq!(tags.len(), 3);
        assert_eq!(tags[0].as_integer(), Some(1));
        assert_eq!(tags[1].as_integer(), Some(2));
        assert_eq!(tags[2].as_integer(), Some(3));

        // key4: key4.default.val = { env = "MY_VAR" }
        let key4_decl = doc
            .settings
            .get("key4")
            .unwrap()
            .profiles
            .get("default")
            .unwrap();
        assert!(!key4_decl.null);
        assert!(key4_decl.env.is_none());
        assert!(key4_decl.tags.is_empty());
        let val4 = key4_decl.val.as_ref().unwrap().as_table().unwrap();
        assert_eq!(val4.get("env").unwrap().as_str(), Some("MY_VAR"));

        // key5: key5.default.val = { default = { val = 1 } }
        let key5_decl = doc
            .settings
            .get("key5")
            .unwrap()
            .profiles
            .get("default")
            .unwrap();
        assert!(!key5_decl.null);
        assert!(key5_decl.env.is_none());
        assert!(key5_decl.tags.is_empty());
        let val5 = key5_decl.val.as_ref().unwrap().as_table().unwrap();
        let nested_default = val5.get("default").unwrap().as_table().unwrap();
        assert_eq!(nested_default.get("val").unwrap().as_integer(), Some(1));
    }
}
