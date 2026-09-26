use std::fmt;

#[derive(Debug)]
pub enum SettingSpecError {
    ConfigNotFound,
    InvalidToml(String),
    MissingSettingsSection,
    ReservedProfileInOptions,
    ReservedKeywordConflict(String),
    UnknownProfileInSettings(String),
    ActiveProfileNotInOptions(String),
    NoProfileSelected,
    StrictCoverageFailed(String, String), // setting key, missing profile
    InvalidDirective(String, String, String), // setting key, profile, directive
    NullCoexistsWithVal(String, String),  // setting key, profile
    InvalidNullValue(String, String),     // setting key, profile
    InvalidTagsValue(String, String),     // setting key, profile
    InvalidEnvValue(String, String),      // setting key, profile
    MissingRequiredEnvVar(String, String), // setting key, env var name
    RequiredValueMissing(String),         // setting key
    EnvFileNotFound(String),
    InvalidExportFormat(String),
    IoError(String),
    CommandFailed(i32),
}

impl fmt::Display for SettingSpecError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ConfigNotFound => write!(
                f,
                "Configuration file 'settingspec.toml' not found in project root"
            ),
            Self::InvalidToml(msg) => write!(f, "Invalid TOML syntax: {}", msg),
            Self::MissingSettingsSection => write!(
                f,
                "Missing required '[settings]' section in settingspec.toml"
            ),
            Self::ReservedProfileInOptions => write!(
                f,
                "Reserved profile identifier 'default' MUST NOT be in spec.profile.options"
            ),
            Self::ReservedKeywordConflict(seg) => write!(
                f,
                "Reserved directive keyword '{}' cannot be used as a setting key segment or profile name",
                seg
            ),
            Self::UnknownProfileInSettings(p) => write!(
                f,
                "Unknown profile '{}' used in [settings] (not in spec.profile.options)",
                p
            ),
            Self::ActiveProfileNotInOptions(p) => write!(
                f,
                "Active profile '{}' is not an element of spec.profile.options",
                p
            ),
            Self::NoProfileSelected => write!(
                f,
                "No profile was selected and spec.profile.options is defined"
            ),
            Self::StrictCoverageFailed(k, p) => write!(
                f,
                "Strict coverage failure: setting '{}' is missing a declaration for profile '{}'",
                k, p
            ),
            Self::InvalidDirective(k, p, d) => write!(
                f,
                "Invalid directive '{}' for setting '{}' in profile '{}'",
                d, k, p
            ),
            Self::NullCoexistsWithVal(k, p) => write!(
                f,
                "Directive 'null' cannot coexist with 'val' for setting '{}' in profile '{}'",
                k, p
            ),
            Self::InvalidNullValue(k, p) => write!(
                f,
                "Directive 'null' must be true for setting '{}' in profile '{}'",
                k, p
            ),
            Self::InvalidTagsValue(k, p) => write!(
                f,
                "Directive 'tags' must be an array of strings for setting '{}' in profile '{}'",
                k, p
            ),
            Self::InvalidEnvValue(k, p) => write!(
                f,
                "Directive 'env' must be a string for setting '{}' in profile '{}'",
                k, p
            ),
            Self::MissingRequiredEnvVar(k, env) => write!(
                f,
                "Missing required environment variable '{}' for setting '{}'",
                env, k
            ),
            Self::RequiredValueMissing(k) => {
                write!(f, "Required value missing for setting '{}'", k)
            }
            Self::EnvFileNotFound(path) => write!(f, "Environment file '{}' not found", path),
            Self::InvalidExportFormat(fmt) => write!(f, "Unsupported export format: '{}'", fmt),
            Self::IoError(msg) => write!(f, "I/O error: {}", msg),
            Self::CommandFailed(code) => write!(f, "Child command exited with code {}", code),
        }
    }
}

impl std::error::Error for SettingSpecError {}

impl From<std::io::Error> for SettingSpecError {
    fn from(err: std::io::Error) -> Self {
        Self::IoError(err.to_string())
    }
}

pub type Result<T> = std::result::Result<T, SettingSpecError>;
