use crate::error::{Result, SettingSpecError};
use crate::model::*;
use serde::de::IntoDeserializer;
use serde::Deserialize;
use std::collections::{BTreeMap, HashSet};
use toml::Value as TomlValue;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SettingSpecSchema {
    #[serde(default)]
    pub spec: Option<SpecConfig>,
    pub settings: Option<TomlValue>,
}

#[derive(Deserialize)]
struct ProfileDirectiveSchema {
    #[serde(default)]
    pub val: Option<TomlValue>,
    #[serde(default)]
    pub env: Option<String>,
    #[serde(default)]
    pub null: Option<bool>,
}

fn find_first_leaf_path(val: &TomlValue, path: &mut Vec<String>) {
    if let TomlValue::Table(t) = val
        && let Some((k, v)) = t.iter().next()
    {
        path.push(k.clone());
        find_first_leaf_path(v, path);
    }
}

pub fn parse_config_str(content: &str) -> Result<SettingSpecDocument> {
    let schema: SettingSpecSchema = toml::from_str(content)
        .map_err(|e| SettingSpecError::InvalidToml(e.to_string()))?;

    let spec = schema.spec.unwrap_or_default();

    // 1. Validate spec.profile.options
    for opt in &spec.profile.options {
        if is_reserved_profile_name(opt) {
            return Err(SettingSpecError::ReservedProfileInOptions(opt.clone()));
        }
    }

    // 2. Validate spec.profile.default
    if let Some(def) = &spec.profile.default
        && is_reserved_profile_name(def)
    {
        return Err(SettingSpecError::ReservedKeywordConflict(def.clone()));
    }

    // 3. Validate spec.envfile.profiles
    for prof in spec.envfile.profiles.keys() {
        if is_reserved_profile_name(prof) {
            return Err(SettingSpecError::ReservedKeywordConflict(prof.clone()));
        }
    }

    // 4. Validate [settings] section exists and is not empty
    let settings_val = schema
        .settings
        .ok_or(SettingSpecError::MissingSettingsSection)?;

    let settings_table = match settings_val {
        TomlValue::Table(tbl) => tbl,
        _ => return Err(SettingSpecError::MissingSettingsSection),
    };

    if settings_table.is_empty() {
        return Err(SettingSpecError::MissingSettingsSection);
    }

    let mut settings: BTreeMap<String, Setting> = BTreeMap::new();
    let mut current_path = Vec::new();
    let mut has_tags_set = HashSet::new();

    extract_settings(
        &settings_table,
        &mut current_path,
        &spec.profile.options,
        &mut settings,
        &mut has_tags_set,
    )?;

    if settings.is_empty() {
        return Err(SettingSpecError::MissingSettingsSection);
    }

    // 5. Strict profile coverage validation
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
    has_tags_set: &mut HashSet<String>,
) -> Result<()> {
    if table.is_empty() && !current_path.is_empty() {
        let decl = current_path.join(".");
        return Err(SettingSpecError::InvalidDeclaration(
            decl,
            "missing '_' separator".into(),
        ));
    }

    // If table contains '_', process the '_' separator declaration for the current setting
    if let Some(under_val) = table.get("_") {
        if current_path.is_empty() {
            let mut leaf_path = vec!["_".to_string()];
            find_first_leaf_path(under_val, &mut leaf_path);
            return Err(SettingSpecError::InvalidDeclaration(
                leaf_path.join("."),
                "setting key cannot be empty".into(),
            ));
        }

        // Validate that setting key does not contain reserved '_' component
        if current_path
            .iter()
            .any(|seg| seg == "_" || seg.split('.').any(|s| s == "_"))
        {
            let setting_key = current_path.join(".");
            return Err(SettingSpecError::InvalidDeclaration(
                setting_key,
                "reserved separator '_' cannot be used as part of a setting key".into(),
            ));
        }

        let setting_key = current_path.join(".");

        let under_table = match under_val {
            TomlValue::Table(t) => t,
            _ => {
                let decl = format!("{}._", setting_key);
                return Err(SettingSpecError::InvalidDeclaration(
                    decl,
                    "expected profile table or tags after '_'".into(),
                ));
            }
        };

        if under_table.is_empty() {
            let decl = format!("{}._", setting_key);
            return Err(SettingSpecError::InvalidDeclaration(
                decl,
                "empty declaration after '_'".into(),
            ));
        }

        for (k, v) in under_table {
            if k == "tags" {
                if let TomlValue::Table(sub_t) = v
                    && !sub_t.is_empty()
                {
                    let mut inner_path = Vec::new();
                    find_first_leaf_path(v, &mut inner_path);
                    let decl = format!("{}._.tags.{}", setting_key, inner_path.join("."));
                    return Err(SettingSpecError::InvalidDeclaration(
                        decl,
                        "reserved identifier 'tags' cannot be used as a profile name".into(),
                    ));
                }

                if !has_tags_set.insert(setting_key.clone()) {
                    return Err(SettingSpecError::DuplicateTags(setting_key.clone()));
                }

                let tags_vec: Vec<String> =
                    Vec::deserialize(v.clone().into_deserializer())
                        .map_err(|e| SettingSpecError::InvalidToml(e.to_string()))?;

                let entry = settings
                    .entry(setting_key.clone())
                    .or_insert_with(|| Setting {
                        key: setting_key.clone(),
                        tags: Vec::new(),
                        profiles: BTreeMap::new(),
                    });
                entry.tags = tags_vec;
            } else {
                let profile = k;

                let prof_table = match v {
                    TomlValue::Table(t) => t,
                    _ => {
                        let decl = format!("{}._.{}", setting_key, profile);
                        return Err(SettingSpecError::InvalidDeclaration(
                            decl,
                            "missing directive".into(),
                        ));
                    }
                };

                if prof_table.is_empty() {
                    let decl = format!("{}._.{}", setting_key, profile);
                    return Err(SettingSpecError::InvalidDeclaration(
                        decl,
                        "missing directive".into(),
                    ));
                }

                // Check for more than one '_' component
                if let Some(second_under) = prof_table.get("_") {
                    let mut inner_path = Vec::new();
                    find_first_leaf_path(second_under, &mut inner_path);
                    let decl = if inner_path.is_empty() {
                        format!("{}._.{}._", setting_key, profile)
                    } else {
                        format!("{}._.{}._.{}", setting_key, profile, inner_path.join("."))
                    };
                    return Err(SettingSpecError::InvalidDeclaration(
                        decl,
                        "more than one '_' component".into(),
                    ));
                }

                // Validate profile membership against profile_options
                if !profile_options.is_empty()
                    && profile != "default"
                    && !profile_options.contains(profile)
                {
                    return Err(SettingSpecError::UnknownProfileInSettings(profile.clone()));
                }

                // Validate directive keywords
                for dir_k in prof_table.keys() {
                    if !is_valid_directive(dir_k) {
                        return Err(SettingSpecError::InvalidDirective(
                            setting_key.clone(),
                            profile.clone(),
                            dir_k.clone(),
                        ));
                    }
                }

                // Deserialize directives
                let decl_schema: ProfileDirectiveSchema =
                    ProfileDirectiveSchema::deserialize(v.clone().into_deserializer())
                        .map_err(|e| SettingSpecError::InvalidToml(e.to_string()))?;

                if let Some(false) = decl_schema.null {
                    return Err(SettingSpecError::InvalidNullValue(
                        setting_key.clone(),
                        profile.clone(),
                    ));
                }

                if decl_schema.null == Some(true) && decl_schema.val.is_some() {
                    return Err(SettingSpecError::NullCoexistsWithVal(
                        setting_key.clone(),
                        profile.clone(),
                    ));
                }

                let entry = settings
                    .entry(setting_key.clone())
                    .or_insert_with(|| Setting {
                        key: setting_key.clone(),
                        tags: Vec::new(),
                        profiles: BTreeMap::new(),
                    });

                let profile_decl = entry.profiles.entry(profile.clone()).or_default();
                if let Some(val) = prof_table.get("val").cloned() {
                    profile_decl.val = Some(val);
                }
                if let Some(env) = decl_schema.env {
                    profile_decl.env = Some(env);
                }
                if decl_schema.null == Some(true) {
                    profile_decl.null = true;
                }
                if profile_decl.null && profile_decl.val.is_some() {
                    return Err(SettingSpecError::NullCoexistsWithVal(
                        setting_key.clone(),
                        profile.clone(),
                    ));
                }
            }
        }
    }

    // Descend into child tables for any keys other than '_'
    for (k, v) in table {
        if k == "_" {
            continue;
        }

        if let TomlValue::Table(sub_table) = v {
            current_path.push(k.clone());
            extract_settings(
                sub_table,
                current_path,
                profile_options,
                settings,
                has_tags_set,
            )?;
            current_path.pop();
        } else {
            // Found a leaf that was not preceded by '_'
            current_path.push(k.clone());
            let decl = current_path.join(".");
            current_path.pop();
            return Err(SettingSpecError::InvalidDeclaration(
                decl,
                "missing '_' separator".into(),
            ));
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
key1._.default.val = { val = 1 }
key2._.default.val = { null = true }
key3._.default.val = { tags = [1, 2, 3] }
key4._.default.val = { env = "MY_VAR" }
key5._.default.val = { default = { val = 1 } }
"#;

        let doc = parse_config_str(toml_str).expect("Failed to parse config");

        // key1: key1._.default.val = { val = 1 }
        let key1_decl = doc
            .settings
            .get("key1")
            .unwrap()
            .profiles
            .get("default")
            .unwrap();
        assert!(!key1_decl.null);
        assert!(key1_decl.env.is_none());
        let val1 = key1_decl.val.as_ref().unwrap().as_table().unwrap();
        assert_eq!(val1.get("val").unwrap().as_integer(), Some(1));

        // key2: key2._.default.val = { null = true }
        let key2_decl = doc
            .settings
            .get("key2")
            .unwrap()
            .profiles
            .get("default")
            .unwrap();
        assert!(!key2_decl.null);
        assert!(key2_decl.env.is_none());
        let val2 = key2_decl.val.as_ref().unwrap().as_table().unwrap();
        assert_eq!(val2.get("null").unwrap().as_bool(), Some(true));

        // key3: key3._.default.val = { tags = [1, 2, 3] }
        let key3_decl = doc
            .settings
            .get("key3")
            .unwrap()
            .profiles
            .get("default")
            .unwrap();
        assert!(!key3_decl.null);
        assert!(key3_decl.env.is_none());
        let val3 = key3_decl.val.as_ref().unwrap().as_table().unwrap();
        let tags = val3.get("tags").unwrap().as_array().unwrap();
        assert_eq!(tags.len(), 3);
        assert_eq!(tags[0].as_integer(), Some(1));
        assert_eq!(tags[1].as_integer(), Some(2));
        assert_eq!(tags[2].as_integer(), Some(3));

        // key4: key4._.default.val = { env = "MY_VAR" }
        let key4_decl = doc
            .settings
            .get("key4")
            .unwrap()
            .profiles
            .get("default")
            .unwrap();
        assert!(!key4_decl.null);
        assert!(key4_decl.env.is_none());
        let val4 = key4_decl.val.as_ref().unwrap().as_table().unwrap();
        assert_eq!(val4.get("env").unwrap().as_str(), Some("MY_VAR"));

        // key5: key5._.default.val = { default = { val = 1 } }
        let key5_decl = doc
            .settings
            .get("key5")
            .unwrap()
            .profiles
            .get("default")
            .unwrap();
        assert!(!key5_decl.null);
        assert!(key5_decl.env.is_none());
        let val5 = key5_decl.val.as_ref().unwrap().as_table().unwrap();
        let nested_default = val5.get("default").unwrap().as_table().unwrap();
        assert_eq!(nested_default.get("val").unwrap().as_integer(), Some(1));
    }
}
