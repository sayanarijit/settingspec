use crate::error::{Result, SettingSpecError};
use crate::model::ProfileConfig;

/// Validates a trimmed, non-empty profile name against the configured
/// options (if any are configured; an empty list means any name is valid).
fn validate_profile(name: String, options: &[String]) -> Result<Option<String>> {
    if !options.is_empty() && !options.contains(&name) {
        return Err(SettingSpecError::ActiveProfileNotInOptions(name));
    }
    Ok(Some(name))
}

pub fn resolve_active_profile(config: &ProfileConfig) -> Result<Option<String>> {
    // 1. Environment variable
    if let Ok(val) = std::env::var(&config.key) {
        let trimmed = val.trim().to_string();
        if !trimmed.is_empty() {
            return validate_profile(trimmed, &config.options);
        }
    }

    // 2. Specification default
    if let Some(ref def) = config.default {
        let trimmed = def.trim().to_string();
        if !trimmed.is_empty() {
            return validate_profile(trimmed, &config.options);
        }
    }

    // 3. Resolution Failure / Default-only
    if !config.options.is_empty() {
        Err(SettingSpecError::NoProfileSelected)
    } else {
        Ok(None)
    }
}
