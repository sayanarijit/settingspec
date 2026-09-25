#![allow(deprecated)]

use crate::error::{Result, SettingSpecError};
use crate::model::EnvFileConfig;
use std::collections::HashMap;
use std::io::Read;
use std::path::Path;

/// Loads dotenv-formatted entries from `path` into `map`. Does not mutate
/// the process environment; callers that need these values visible to
/// `std::env::var` (e.g. setting resolution) must pass the returned map
/// through explicitly.
fn load_dotenv_into_map(path: &Path, map: &mut HashMap<String, String>) -> Result<()> {
    for (k, v) in dotenv::from_path_iter(path)
        .map_err(|e| SettingSpecError::IoError(e.to_string()))?
        .flatten()
    {
        map.insert(k, v);
    }
    Ok(())
}

pub fn source_env_file(
    config: &EnvFileConfig,
    active_profile: Option<&str>,
) -> Result<HashMap<String, String>> {
    // 1. Determine which envfile to load: active profile takes precedence over default
    let target = if let Some(profile) = active_profile {
        config
            .profiles
            .get(profile)
            .cloned()
            .or_else(|| config.default.clone())
    } else {
        config.default.clone()
    };

    let Some(envfile_spec) = target else {
        return Ok(HashMap::new());
    };

    let mut map = HashMap::new();

    if envfile_spec == "-" {
        use std::io::Write;
        let mut temp = tempfile::NamedTempFile::new()?;
        let mut buffer = String::new();
        std::io::stdin().read_to_string(&mut buffer)?;
        temp.write_all(buffer.as_bytes())?;
        temp.flush()?;

        load_dotenv_into_map(temp.path(), &mut map)?;
    } else {
        let path = Path::new(&envfile_spec);
        if !path.exists() {
            return Err(SettingSpecError::EnvFileNotFound(envfile_spec));
        }

        load_dotenv_into_map(path, &mut map)?;
    }

    Ok(map)
}

pub fn parse_dotenv(content: &str) -> HashMap<String, String> {
    use std::io::Write;
    let mut map = HashMap::new();
    if let Ok(mut temp) = tempfile::NamedTempFile::new()
        && temp.write_all(content.as_bytes()).is_ok()
        && temp.flush().is_ok()
        && let Ok(iter) = dotenv::from_path_iter(temp.path())
    {
        for (k, v) in iter.flatten() {
            map.insert(k, v);
        }
    }
    map
}
