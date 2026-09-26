use crate::error::{Result, SettingSpecError};
use crate::model::{DecryptionConfig, EnvFileConfig};
use std::collections::{HashMap, HashSet};
use std::io::Read;
use std::path::{Path, PathBuf};

const AGE_BINARY_HEADER: &[u8] = b"age-encryption.org/";
const AGE_ARMOR_HEADER: &[u8] = b"-----BEGIN AGE ENCRYPTED FILE-----";

pub fn is_age_encrypted_payload(data: &[u8]) -> bool {
    let trimmed = match std::str::from_utf8(data) {
        Ok(s) => s.trim_start().as_bytes(),
        Err(_) => data,
    };
    data.starts_with(AGE_BINARY_HEADER)
        || data.starts_with(AGE_ARMOR_HEADER)
        || trimmed.starts_with(AGE_ARMOR_HEADER)
}

pub fn is_age_encrypted_file(path_str: &str, data: &[u8]) -> bool {
    path_str.ends_with(".age") || is_age_encrypted_payload(data)
}

enum CandidateIdentity {
    X25519(age::x25519::Identity),
    Ssh(age::ssh::Identity),
}

impl CandidateIdentity {
    fn as_identity(&self) -> &dyn age::Identity {
        match self {
            Self::X25519(id) => id,
            Self::Ssh(id) => id,
        }
    }
}

fn try_decrypt(payload: &[u8], identity: &CandidateIdentity) -> Option<Vec<u8>> {
    let decryptor = age::Decryptor::new(age::armor::ArmoredReader::new(payload)).ok()?;
    let mut reader = decryptor
        .decrypt(std::iter::once(identity.as_identity()))
        .ok()?;
    let mut output = Vec::new();
    if reader.read_to_end(&mut output).is_ok() {
        Some(output)
    } else {
        None
    }
}

fn expand_tilde(path_str: &str) -> PathBuf {
    if path_str == "~" {
        if let Ok(home) = std::env::var("HOME").or_else(|_| std::env::var("USERPROFILE")) {
            return PathBuf::from(home);
        }
    } else if let Some(rest) = path_str
        .strip_prefix("~/")
        .or_else(|| path_str.strip_prefix("~\\"))
        && let Ok(home) = std::env::var("HOME").or_else(|_| std::env::var("USERPROFILE"))
    {
        return PathBuf::from(home).join(rest);
    }
    PathBuf::from(path_str)
}

fn extract_identities_from_file(path: &Path) -> Vec<CandidateIdentity> {
    let Ok(bytes) = std::fs::read(path) else {
        return Vec::new();
    };

    // Passphrase restriction: reject/skip passphrase-protected age files
    if is_age_encrypted_payload(&bytes) {
        return Vec::new();
    }

    let Ok(content) = std::str::from_utf8(&bytes) else {
        return Vec::new();
    };

    // Check for age identity file (x25519)
    let mut x25519_identities = Vec::new();
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("AGE-SECRET-KEY-1")
            && let Ok(id) = trimmed.parse::<age::x25519::Identity>()
        {
            x25519_identities.push(CandidateIdentity::X25519(id));
        }
    }
    if !x25519_identities.is_empty() {
        return x25519_identities;
    }

    // Check for SSH private key
    if (content.contains("-----BEGIN OPENSSH PRIVATE KEY-----")
        || content.contains("-----BEGIN RSA PRIVATE KEY-----"))
        && let Ok(identity) = age::ssh::Identity::from_buffer(
            content.as_bytes(),
            Some(path.to_string_lossy().into_owned()),
        )
    {
        match identity {
            age::ssh::Identity::Unencrypted(_) => {
                return vec![CandidateIdentity::Ssh(identity)];
            }
            age::ssh::Identity::Encrypted(_) | age::ssh::Identity::Unsupported(_) => {
                return Vec::new();
            }
        }
    }

    Vec::new()
}

fn collect_candidate_files(dir: &Path, files: &mut Vec<PathBuf>, visited: &mut HashSet<PathBuf>) {
    let Ok(canon) = dir.canonicalize() else {
        return;
    };
    if !visited.insert(canon) {
        return;
    }
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    let mut entries: Vec<_> = entries.filter_map(|e| e.ok()).collect();
    entries.sort_by_key(|e| e.path());
    for entry in entries {
        let path = entry.path();
        if path.is_dir() {
            collect_candidate_files(&path, files, visited);
        } else if path.is_file() {
            files.push(path);
        }
    }
}

fn try_path_lookup(payload: &[u8], path: &Path) -> Option<Vec<u8>> {
    if path.is_file() {
        let candidates = extract_identities_from_file(path);
        for candidate in candidates {
            if let Some(decrypted) = try_decrypt(payload, &candidate) {
                return Some(decrypted);
            }
        }
    } else if path.is_dir() {
        let mut candidate_files = Vec::new();
        let mut visited = HashSet::new();
        collect_candidate_files(path, &mut candidate_files, &mut visited);
        for file in candidate_files {
            let candidates = extract_identities_from_file(&file);
            for candidate in candidates {
                if let Some(decrypted) = try_decrypt(payload, &candidate) {
                    return Some(decrypted);
                }
            }
        }
    }
    None
}

pub fn decrypt_envfile_payload(
    payload: &[u8],
    decryption: &DecryptionConfig,
    profile: &str,
    file_name: &str,
) -> Result<Vec<u8>> {
    // 1. Environment Variable:
    // SettingSpec reads the environment variable named by decryption.key.env (default SETTINGSPEC_DECRYPTION_KEY).
    // If set and non-empty, its value is interpreted per the Value Format rules below, and decryption.key.path is NOT consulted.
    let env_val = std::env::var(&decryption.key.env).ok();
    if let Some(ref val) = env_val
        && !val.is_empty()
    {
        let trimmed = val.trim();
        if trimmed.starts_with("AGE-SECRET-KEY-1") {
            // A single literal age identity string
            if let Ok(id) = trimmed.parse::<age::x25519::Identity>() {
                let candidate = CandidateIdentity::X25519(id);
                if let Some(decrypted) = try_decrypt(payload, &candidate) {
                    return Ok(decrypted);
                }
            }
            // Decryption failed with the literal identity
            return Err(SettingSpecError::DecryptionFailed(
                profile.to_string(),
                file_name.to_string(),
            ));
        } else {
            // One or more filesystem paths, delimited by colons (:)
            for path_segment in val.split(':') {
                let seg = path_segment.trim();
                if seg.is_empty() {
                    continue;
                }
                let expanded = expand_tilde(seg);
                if let Some(decrypted) = try_path_lookup(payload, &expanded) {
                    return Ok(decrypted);
                }
            }
            // None succeeded across all supplied paths
            return Err(SettingSpecError::DecryptionFailed(
                profile.to_string(),
                file_name.to_string(),
            ));
        }
    }

    // 2. Fallback Path(s):
    // If the environment variable is unset or empty, SettingSpec falls back to decryption.key.path
    for path_str in &decryption.key.path {
        let trimmed = path_str.trim();
        if trimmed.is_empty() {
            continue;
        }
        let expanded = expand_tilde(trimmed);
        if let Some(decrypted) = try_path_lookup(payload, &expanded) {
            return Ok(decrypted);
        }
    }

    // 3. Resolution Failure:
    // If an envfile.* entry is age-encrypted and no decryption identity resolves via steps 1-2,
    // decryption MUST fail and SettingSpec MUST terminate with an error identifying the affected profile and file.
    Err(SettingSpecError::DecryptionFailed(
        profile.to_string(),
        file_name.to_string(),
    ))
}

fn load_dotenv_from_reader<R: Read>(reader: R, map: &mut HashMap<String, String>) -> Result<()> {
    for item in dotenvy::from_read_iter(reader) {
        let (k, v) = item.map_err(|e| SettingSpecError::IoError(e.to_string()))?;
        map.insert(k, v);
    }
    Ok(())
}

pub fn resolve_envfile_target<'a>(
    config: &'a EnvFileConfig,
    active_profile: Option<&str>,
) -> Option<(&'a str, String)> {
    let profile_name = active_profile.unwrap_or("default").to_string();

    if let Some(active) = active_profile
        && let Some(spec) = config.profiles.get(active)
    {
        return Some((spec.as_str(), profile_name));
    }

    if let Some(ref spec) = config.default {
        return Some((spec.as_str(), profile_name));
    }

    None
}

pub fn source_env_file(
    config: &EnvFileConfig,
    decryption: &DecryptionConfig,
    active_profile: Option<&str>,
) -> Result<HashMap<String, String>> {
    let Some((envfile_spec, profile_name)) = resolve_envfile_target(config, active_profile) else {
        return Ok(HashMap::new());
    };

    let mut map = HashMap::new();

    if envfile_spec == "-" {
        let mut buffer = Vec::new();
        std::io::stdin().read_to_end(&mut buffer)?;

        if is_age_encrypted_payload(&buffer) {
            let decrypted = decrypt_envfile_payload(&buffer, decryption, &profile_name, "-")?;
            load_dotenv_from_reader(decrypted.as_slice(), &mut map)?;
        } else {
            load_dotenv_from_reader(buffer.as_slice(), &mut map)?;
        }
    } else {
        let path = Path::new(&envfile_spec);
        if !path.exists() {
            return Err(SettingSpecError::EnvFileNotFound(envfile_spec.to_string()));
        }

        let bytes = std::fs::read(path)?;
        if is_age_encrypted_file(envfile_spec, &bytes) {
            let decrypted =
                decrypt_envfile_payload(&bytes, decryption, &profile_name, envfile_spec)?;
            load_dotenv_from_reader(decrypted.as_slice(), &mut map)?;
        } else {
            load_dotenv_from_reader(bytes.as_slice(), &mut map)?;
        }
    }

    Ok(map)
}

pub fn parse_dotenv(content: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    for (k, v) in dotenvy::from_read_iter(content.as_bytes()).flatten() {
        map.insert(k, v);
    }
    map
}
