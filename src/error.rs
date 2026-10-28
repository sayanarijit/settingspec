use std::fmt;

#[derive(Debug)]
pub enum SettingSpecError {
    ConfigNotFound,
    InvalidToml(String),
    MissingSettingsSection,
    ReservedProfileInOptions(String),
    ReservedKeywordConflict(String),
    InvalidDeclaration(String, String),
    UnknownProfileInSettings(String),
    ActiveProfileNotInOptions(String),
    NoProfileSelected,
    StrictCoverageFailed(String, String), // setting key, missing profile
    InvalidDirective(String, String, String), // setting key, profile, directive
    NullCoexistsWithVal(String, String),  // setting key, profile
    InvalidNullValue(String, String),     // setting key, profile
    InvalidTagsValue(String),             // setting key
    DuplicateTags(String),                // setting key
    InvalidEnvValue(String, String),      // setting key, profile
    MissingRequiredEnvVar(String, String), // setting key, env var name
    RequiredValueMissing(String),         // setting key
    EnvFileNotFound(String),
    DecryptionFailed(String, String), // profile, file
    InvalidExportFormat(String),
    UnresolvedExportUnsupported(String, String), // format, setting key
    UnsupportedType(String, String),             // format, setting key
    InconsistentEnvDirective(String, String),    // setting key, profile
    EnvFallbackUnsupported(String, String),      // format, setting key
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
            Self::ReservedProfileInOptions(name) => write!(
                f,
                "Reserved profile identifier '{}' MUST NOT be in spec.profile.options",
                name
            ),
            Self::ReservedKeywordConflict(seg) => {
                write!(f, "Reserved profile name '{}' cannot be used", seg)
            }
            Self::InvalidDeclaration(decl, reason) => {
                write!(f, "Invalid setting declaration '{}': {}", decl, reason)
            }
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
            Self::InvalidTagsValue(k) => write!(
                f,
                "Directive 'tags' for setting '{}': expected a sequence of strings",
                k
            ),
            Self::DuplicateTags(k) => {
                write!(f, "Setting '{}' has more than one tags declaration", k)
            }
            Self::InvalidEnvValue(k, p) => write!(
                f,
                "Directive 'env' for setting '{}' in profile '{}': expected a string",
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
            Self::DecryptionFailed(profile, file) => write!(
                f,
                "Failed to decrypt age-encrypted environment file '{}' for profile '{}'",
                file, profile
            ),
            Self::InvalidExportFormat(fmt) => write!(f, "Unsupported export format: '{}'", fmt),
            Self::UnresolvedExportUnsupported(fmt, key) => write!(
                f,
                "Setting '{}' reads an environment variable, which '{}' exports cannot reference; set spec.export.resolve = true",
                key, fmt
            ),
            Self::UnsupportedType(fmt, key) => write!(
                f,
                "The type of setting '{}' (a union, optional, or environment-backed value) cannot be expressed in '{}' exports",
                key, fmt
            ),
            Self::InconsistentEnvDirective(k, p) => write!(
                f,
                "Setting '{}' uses an 'env' directive in some profiles but not in profile '{}'; with spec.export.resolve = false every profile must use 'env' (fallbacks allowed)",
                k, p
            ),
            Self::EnvFallbackUnsupported(fmt, key) => write!(
                f,
                "Setting '{}' has an 'env' directive with a 'val'/'null' fallback, which '{}' exports cannot express; set spec.export.resolve = true or remove the fallback",
                key, fmt
            ),
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
