use indexmap::IndexMap;
use serde::{Deserialize, Deserializer, Serialize};
use std::collections::BTreeMap;
use toml::Value as TomlValue;

pub fn is_reserved_directive_keyword(name: &str) -> bool {
    matches!(name, "val" | "env" | "null" | "tags")
}

fn default_profile_key() -> String {
    "SETTINGSPEC_PROFILE".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProfileConfig {
    #[serde(default = "default_profile_key")]
    pub key: String,
    #[serde(default)]
    pub options: Vec<String>,
    #[serde(default)]
    pub default: Option<String>,
}

impl Default for ProfileConfig {
    fn default() -> Self {
        Self {
            key: default_profile_key(),
            options: Vec::new(),
            default: None,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct EnvFileConfig {
    pub default: Option<String>,
    pub profiles: BTreeMap<String, String>,
}

impl<'de> Deserialize<'de> for EnvFileConfig {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let map = BTreeMap::<String, String>::deserialize(deserializer)?;
        let mut default = None;
        let mut profiles = BTreeMap::new();
        for (k, v) in map {
            if k == "default" {
                default = Some(v);
            } else {
                profiles.insert(k, v);
            }
        }
        Ok(EnvFileConfig { default, profiles })
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum FilterRule {
    Include,
    Exclude,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct FilterScope {
    pub rules: BTreeMap<String, FilterRule>, // key or subscope name -> rule
    pub subscopes: BTreeMap<String, FilterScope>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum ExportFilter {
    All,
    Disabled,
    Table(FilterScope),
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
enum ExportFilterHelper {
    Bool(bool),
    Table(toml::Table),
}

impl<'de> Deserialize<'de> for ExportFilter {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let helper = ExportFilterHelper::deserialize(deserializer)?;
        match helper {
            ExportFilterHelper::Bool(true) => Ok(ExportFilter::All),
            ExportFilterHelper::Bool(false) => Ok(ExportFilter::Disabled),
            ExportFilterHelper::Table(tbl) => {
                let mut scope = FilterScope::default();
                for (k, v) in &tbl {
                    insert_filter_rule_model(&mut scope, k, v).map_err(serde::de::Error::custom)?;
                }
                Ok(ExportFilter::Table(scope))
            }
        }
    }
}

fn insert_filter_rule_model(
    scope: &mut FilterScope,
    key_path: &str,
    val: &TomlValue,
) -> std::result::Result<(), String> {
    if key_path.starts_with('#') {
        match val {
            TomlValue::Boolean(b) => {
                let rule = if *b {
                    FilterRule::Include
                } else {
                    FilterRule::Exclude
                };
                scope.rules.insert(key_path.to_string(), rule);
                Ok(())
            }
            _ => Err(format!("Tag selector '{}' must be boolean", key_path)),
        }
    } else if let Some((first, rest)) = key_path.split_once('.') {
        let sub = scope.subscopes.entry(first.to_string()).or_default();
        insert_filter_rule_model(sub, rest, val)
    } else {
        match val {
            TomlValue::Boolean(b) => {
                let rule = if *b {
                    FilterRule::Include
                } else {
                    FilterRule::Exclude
                };
                scope.rules.insert(key_path.to_string(), rule);
                Ok(())
            }
            TomlValue::Table(sub_table) => {
                let sub = scope.subscopes.entry(key_path.to_string()).or_default();
                for (sub_k, sub_v) in sub_table {
                    insert_filter_rule_model(sub, sub_k, sub_v)?;
                }
                Ok(())
            }
            _ => Err(format!(
                "Filter selector '{}' must be boolean or table",
                key_path
            )),
        }
    }
}

fn default_export_mode() -> u32 {
    0o600
}

fn deserialize_mode<'de, D>(deserializer: D) -> std::result::Result<u32, D::Error>
where
    D: Deserializer<'de>,
{
    let val = i64::deserialize(deserializer)?;
    if (0..=0o777).contains(&val) {
        return Ok(val as u32);
    }
    let hex = format!("{:x}", val);
    if let Ok(octal) = u32::from_str_radix(&hex, 8)
        && (0..=0o7777).contains(&octal)
    {
        return Ok(octal);
    }
    Ok(0o600)
}

/// A field that accepts either a boolean toggle or a string value in TOML.
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
enum BoolOrString {
    Bool(bool),
    Str(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum ExportEnvSpec {
    Disabled,
    Enabled(String),
}

fn deserialize_export_env<'de, D>(
    deserializer: D,
) -> std::result::Result<Option<ExportEnvSpec>, D::Error>
where
    D: Deserializer<'de>,
{
    let value = BoolOrString::deserialize(deserializer)
        .map_err(|_| serde::de::Error::custom("expected a boolean or a string prefix"))?;
    let spec = match value {
        BoolOrString::Bool(true) => ExportEnvSpec::Enabled(String::new()),
        BoolOrString::Bool(false) => ExportEnvSpec::Disabled,
        BoolOrString::Str(s) => ExportEnvSpec::Enabled(s),
    };
    Ok(Some(spec))
}

#[derive(Debug, Clone, PartialEq)]
pub enum ExportStdinSpec {
    Disabled,
    Enabled(String),
}

fn deserialize_export_stdin<'de, D>(
    deserializer: D,
) -> std::result::Result<Option<ExportStdinSpec>, D::Error>
where
    D: Deserializer<'de>,
{
    let value = BoolOrString::deserialize(deserializer)
        .map_err(|_| serde::de::Error::custom("expected a boolean or a format string"))?;
    let spec = match value {
        BoolOrString::Bool(true) => ExportStdinSpec::Enabled("toml".to_string()),
        BoolOrString::Bool(false) => ExportStdinSpec::Disabled,
        BoolOrString::Str(s) if s.is_empty() => ExportStdinSpec::Enabled("toml".to_string()),
        BoolOrString::Str(s) => ExportStdinSpec::Enabled(s),
    };
    Ok(Some(spec))
}

#[derive(Debug, Clone, PartialEq)]
pub enum ExportStdoutSpec {
    Disabled,
    Enabled(String),
}

fn deserialize_export_stdout<'de, D>(
    deserializer: D,
) -> std::result::Result<Option<ExportStdoutSpec>, D::Error>
where
    D: Deserializer<'de>,
{
    let value = BoolOrString::deserialize(deserializer)
        .map_err(|_| serde::de::Error::custom("expected a boolean or a format string"))?;
    let spec = match value {
        BoolOrString::Bool(true) => ExportStdoutSpec::Enabled("toml".to_string()),
        BoolOrString::Bool(false) => ExportStdoutSpec::Disabled,
        BoolOrString::Str(s) if s.is_empty() => ExportStdoutSpec::Enabled("toml".to_string()),
        BoolOrString::Str(s) => ExportStdoutSpec::Enabled(s),
    };
    Ok(Some(spec))
}

#[derive(Debug, Clone, PartialEq)]
pub enum ExportFileSpec {
    Disabled,
    Default,
    Map(IndexMap<String, ExportFilter>),
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
enum ExportFileHelper {
    Bool(bool),
    Map(IndexMap<String, ExportFilter>),
    Str(String),
}

fn deserialize_export_file<'de, D>(
    deserializer: D,
) -> std::result::Result<Option<ExportFileSpec>, D::Error>
where
    D: Deserializer<'de>,
{
    let value = ExportFileHelper::deserialize(deserializer).map_err(|_| {
        serde::de::Error::custom("expected a boolean, a string, or a table of target files")
    })?;
    let spec = match value {
        ExportFileHelper::Bool(true) => ExportFileSpec::Default,
        ExportFileHelper::Bool(false) => ExportFileSpec::Disabled,
        ExportFileHelper::Map(m) => ExportFileSpec::Map(m),
        ExportFileHelper::Str(s) if s.is_empty() => ExportFileSpec::Default,
        ExportFileHelper::Str(s) => {
            let mut map = IndexMap::new();
            map.insert(s, ExportFilter::All);
            ExportFileSpec::Map(map)
        }
    };
    Ok(Some(spec))
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct ExportConfigSchema {
    #[serde(default = "default_export_mode", deserialize_with = "deserialize_mode")]
    pub mode: u32,
    #[serde(default)]
    pub keep: bool,
    #[serde(default, deserialize_with = "deserialize_export_stdout")]
    pub stdout: Option<ExportStdoutSpec>,
    #[serde(default, deserialize_with = "deserialize_export_file")]
    pub file: Option<ExportFileSpec>,
    #[serde(default, deserialize_with = "deserialize_export_env")]
    pub env: Option<ExportEnvSpec>,
    #[serde(default, deserialize_with = "deserialize_export_stdin")]
    pub stdin: Option<ExportStdinSpec>,
    #[serde(default)]
    pub skip_gitignore: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ExportConfig {
    pub mode: u32,
    pub keep: bool,
    pub stdout: Option<String>,
    pub file: IndexMap<String, ExportFilter>,
    pub env: Option<String>,
    pub stdin: Option<String>,
    pub skip_gitignore: bool,
}

impl Default for ExportConfig {
    fn default() -> Self {
        let mut file = IndexMap::new();
        file.insert("settings.toml".to_string(), ExportFilter::All);
        Self {
            mode: default_export_mode(),
            keep: false,
            stdout: None,
            file,
            env: None,
            stdin: None,
            skip_gitignore: false,
        }
    }
}

impl<'de> Deserialize<'de> for ExportConfig {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let schema = ExportConfigSchema::deserialize(deserializer)?;
        let has_stdout = schema.stdout.is_some();
        let has_file = schema.file.is_some();
        let mut file = match schema.file {
            Some(ExportFileSpec::Default) => {
                let mut map = IndexMap::new();
                map.insert("settings.toml".to_string(), ExportFilter::All);
                map
            }
            Some(ExportFileSpec::Map(m)) => m,
            Some(ExportFileSpec::Disabled) | None => IndexMap::new(),
        };
        let stdout = match schema.stdout {
            Some(ExportStdoutSpec::Enabled(fmt)) => Some(fmt),
            Some(ExportStdoutSpec::Disabled) | None => None,
        };
        let env = match schema.env {
            Some(ExportEnvSpec::Enabled(prefix)) => Some(prefix),
            Some(ExportEnvSpec::Disabled) | None => None,
        };
        let stdin = match schema.stdin {
            Some(ExportStdinSpec::Enabled(fmt)) => Some(fmt),
            Some(ExportStdinSpec::Disabled) | None => None,
        };
        if !has_stdout && !has_file && env.is_none() && stdin.is_none() {
            file.insert("settings.toml".to_string(), ExportFilter::All);
        }
        Ok(ExportConfig {
            mode: schema.mode,
            keep: schema.keep,
            stdout,
            file,
            env,
            stdin,
            skip_gitignore: schema.skip_gitignore,
        })
    }
}

fn default_decryption_key_env() -> String {
    "SETTINGSPEC_DECRYPTION_KEY".to_string()
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
enum StringOrVec {
    Single(String),
    Multiple(Vec<String>),
}

fn deserialize_key_path<'de, D>(deserializer: D) -> std::result::Result<Vec<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let opt = Option::<StringOrVec>::deserialize(deserializer)
        .map_err(|_| serde::de::Error::custom("expected a string or an array of strings"))?;
    match opt {
        Some(StringOrVec::Single(s)) => Ok(vec![s]),
        Some(StringOrVec::Multiple(v)) => Ok(v),
        None => Ok(Vec::new()),
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DecryptionKeyConfig {
    #[serde(default = "default_decryption_key_env")]
    pub env: String,
    #[serde(default, deserialize_with = "deserialize_key_path")]
    pub path: Vec<String>,
}

impl Default for DecryptionKeyConfig {
    fn default() -> Self {
        Self {
            env: default_decryption_key_env(),
            path: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DecryptionConfig {
    #[serde(default)]
    pub key: DecryptionKeyConfig,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SpecConfig {
    #[serde(default)]
    pub profile: ProfileConfig,
    #[serde(default)]
    pub envfile: EnvFileConfig,
    #[serde(default)]
    pub export: ExportConfig,
    #[serde(default)]
    pub decryption: DecryptionConfig,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProfileDeclaration {
    #[serde(default)]
    pub val: Option<TomlValue>,
    #[serde(default)]
    pub env: Option<String>,
    #[serde(default)]
    pub null: bool,
    #[serde(default)]
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Setting {
    pub key: String,
    pub profiles: BTreeMap<String, ProfileDeclaration>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SettingValue {
    Null,
    Value(TomlValue),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResolvedSetting {
    pub key: String,
    pub value: SettingValue,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct SettingSpecDocument {
    pub spec: SpecConfig,
    pub settings: BTreeMap<String, Setting>,
}
