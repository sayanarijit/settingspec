use assert_cmd::Command;
use assert_fs::prelude::*;
use predicates::prelude::*;

#[test]
fn test_envfile_default_loaded_and_used() {
    let temp = assert_fs::TempDir::new().unwrap();
    let config = temp.child("settingspec.toml");
    config
        .write_str(
            r#"
[spec]
profile.options = ["dev", "prod"]
profile.default = "dev"
envfile.default = ".env"

[settings]
secret.default.env = "MY_SECRET"
"#,
        )
        .unwrap();

    let env_file = temp.child(".env");
    env_file
        .write_str("MY_SECRET=super_secret_value\n")
        .unwrap();

    let mut cmd = Command::cargo_bin("settingspec").unwrap();
    cmd.current_dir(temp.path())
        .arg("export")
        .assert()
        .success();

    let exported = temp.child("settings.toml");
    exported.assert(predicate::str::contains("secret = \"super_secret_value\""));
}

#[test]
fn test_profile_specific_envfile_takes_precedence() {
    let temp = assert_fs::TempDir::new().unwrap();
    let config = temp.child("settingspec.toml");
    config
        .write_str(
            r#"
[spec]
profile.options = ["dev", "prod"]
profile.default = "prod"
envfile.default = ".env"
envfile.prod = ".env.prod"

[settings]
db_pass.default.env = "DB_PASSWORD"
"#,
        )
        .unwrap();

    temp.child(".env")
        .write_str("DB_PASSWORD=default_password\n")
        .unwrap();
    temp.child(".env.prod")
        .write_str("DB_PASSWORD=prod_password\n")
        .unwrap();

    let mut cmd = Command::cargo_bin("settingspec").unwrap();
    cmd.current_dir(temp.path())
        .arg("export")
        .assert()
        .success();

    let exported = temp.child("settings.toml");
    exported.assert(predicate::str::contains("db_pass = \"prod_password\""));
}

#[test]
fn test_envfile_stdin_sourcing() {
    let temp = assert_fs::TempDir::new().unwrap();
    let config = temp.child("settingspec.toml");
    config
        .write_str(
            r#"
[spec]
profile.options = ["dev", "prod"]
profile.default = "prod"
envfile.prod = "-"

[settings]
api_token.default.env = "API_TOKEN"
"#,
        )
        .unwrap();

    let mut cmd = Command::cargo_bin("settingspec").unwrap();
    cmd.current_dir(temp.path())
        .write_stdin("API_TOKEN=stdin_secret_token_12345\n")
        .arg("export")
        .assert()
        .success();

    let exported = temp.child("settings.toml");
    exported.assert(predicate::str::contains(
        "api_token = \"stdin_secret_token_12345\"",
    ));
}

#[test]
fn test_missing_envfile_fails_with_error() {
    let temp = assert_fs::TempDir::new().unwrap();
    let config = temp.child("settingspec.toml");
    config
        .write_str(
            r#"
[spec]
envfile.default = "non_existent.env"

[settings]
key1.default.val = "val1"
"#,
        )
        .unwrap();

    let mut cmd = Command::cargo_bin("settingspec").unwrap();
    cmd.current_dir(temp.path())
        .arg("check")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "Environment file 'non_existent.env' not found",
        ));
}

#[test]
fn test_sourced_envfile_passed_to_child_command_in_run() {
    let temp = assert_fs::TempDir::new().unwrap();
    let config = temp.child("settingspec.toml");
    config
        .write_str(
            r#"
[spec]
envfile.default = ".env"

[settings]
dummy.default.val = "1"
"#,
        )
        .unwrap();

    temp.child(".env")
        .write_str("CHILD_CUSTOM_VAR=passed_through_to_child\n")
        .unwrap();

    let mut cmd = Command::cargo_bin("settingspec").unwrap();
    cmd.current_dir(temp.path())
        .args([
            "run",
            "--",
            "sh",
            "-c",
            "test \"$CHILD_CUSTOM_VAR\" = \"passed_through_to_child\"",
        ])
        .assert()
        .success();
}

#[test]
fn test_age_decryption_with_env_var_literal_key() {
    use age::secrecy::ExposeSecret;

    let key = age::x25519::Identity::generate();
    let pubkey = key.to_public();
    let key_str = key.to_string();
    let secret = key_str.expose_secret();

    let encrypted = age::encrypt(&pubkey, b"AGE_SECRET=decrypted_value_123\n").unwrap();

    let temp = assert_fs::TempDir::new().unwrap();
    let config = temp.child("settingspec.toml");
    config
        .write_str(
            r#"
[spec]
profile.options = ["dev"]
profile.default = "dev"
envfile.default = ".env.age"

[settings]
secret.default.env = "AGE_SECRET"
"#,
        )
        .unwrap();

    temp.child(".env.age").write_binary(&encrypted).unwrap();

    let mut cmd = Command::cargo_bin("settingspec").unwrap();
    cmd.current_dir(temp.path())
        .env("SETTINGSPEC_DECRYPTION_KEY", secret)
        .arg("export")
        .assert()
        .success();

    let exported = temp.child("settings.toml");
    exported.assert(predicate::str::contains("secret = \"decrypted_value_123\""));
}

#[test]
fn test_age_decryption_with_env_var_path() {
    use age::secrecy::ExposeSecret;

    let key = age::x25519::Identity::generate();
    let pubkey = key.to_public();
    let key_str = key.to_string();
    let secret = key_str.expose_secret();

    let encrypted = age::encrypt(&pubkey, b"AGE_SECRET=from_env_path\n").unwrap();

    let temp = assert_fs::TempDir::new().unwrap();
    let config = temp.child("settingspec.toml");
    config
        .write_str(
            r#"
[spec]
profile.options = ["dev"]
profile.default = "dev"
envfile.default = ".env.age"

[settings]
secret.default.env = "AGE_SECRET"
"#,
        )
        .unwrap();

    temp.child(".env.age").write_binary(&encrypted).unwrap();
    let key_file = temp.child("my_key.txt");
    key_file.write_str(secret).unwrap();

    let mut cmd = Command::cargo_bin("settingspec").unwrap();
    cmd.current_dir(temp.path())
        .env(
            "SETTINGSPEC_DECRYPTION_KEY",
            key_file.path().to_str().unwrap(),
        )
        .arg("export")
        .assert()
        .success();

    let exported = temp.child("settings.toml");
    exported.assert(predicate::str::contains("secret = \"from_env_path\""));
}

#[test]
fn test_age_decryption_with_env_var_colon_delimited_paths() {
    use age::secrecy::ExposeSecret;

    let key = age::x25519::Identity::generate();
    let pubkey = key.to_public();
    let key_str = key.to_string();
    let secret = key_str.expose_secret();

    let encrypted = age::encrypt(&pubkey, b"AGE_SECRET=colon_path_resolved\n").unwrap();

    let temp = assert_fs::TempDir::new().unwrap();
    let config = temp.child("settingspec.toml");
    config
        .write_str(
            r#"
[spec]
envfile.default = ".env.age"

[settings]
secret.default.env = "AGE_SECRET"
"#,
        )
        .unwrap();

    temp.child(".env.age").write_binary(&encrypted).unwrap();
    let keys_dir = temp.child("keys_dir");
    keys_dir.create_dir_all().unwrap();
    keys_dir.child("key.txt").write_str(secret).unwrap();

    let non_existent = temp.path().join("non_existent_path");
    let colon_env = format!("{}:{}", non_existent.display(), keys_dir.path().display());

    let mut cmd = Command::cargo_bin("settingspec").unwrap();
    cmd.current_dir(temp.path())
        .env("SETTINGSPEC_DECRYPTION_KEY", colon_env)
        .arg("export")
        .assert()
        .success();

    let exported = temp.child("settings.toml");
    exported.assert(predicate::str::contains("secret = \"colon_path_resolved\""));
}

#[test]
fn test_age_decryption_fallback_to_key_path_single_file() {
    use age::secrecy::ExposeSecret;

    let key = age::x25519::Identity::generate();
    let pubkey = key.to_public();
    let key_str = key.to_string();
    let secret = key_str.expose_secret();

    let encrypted = age::encrypt(&pubkey, b"AGE_SECRET=single_file_fallback\n").unwrap();

    let temp = assert_fs::TempDir::new().unwrap();
    let key_file = temp.child("key.txt");
    key_file.write_str(secret).unwrap();

    let config = temp.child("settingspec.toml");
    config
        .write_str(
            r#"
[spec]
envfile.default = ".env.age"
decryption.key.path = "key.txt"

[settings]
secret.default.env = "AGE_SECRET"
"#,
        )
        .unwrap();

    temp.child(".env.age").write_binary(&encrypted).unwrap();

    let mut cmd = Command::cargo_bin("settingspec").unwrap();
    cmd.current_dir(temp.path())
        .env_remove("SETTINGSPEC_DECRYPTION_KEY")
        .arg("export")
        .assert()
        .success();

    let exported = temp.child("settings.toml");
    exported.assert(predicate::str::contains(
        "secret = \"single_file_fallback\"",
    ));
}

#[test]
fn test_age_decryption_fallback_to_key_path_directory_recursive() {
    use age::secrecy::ExposeSecret;

    let key = age::x25519::Identity::generate();
    let pubkey = key.to_public();
    let key_str = key.to_string();
    let secret = key_str.expose_secret();

    let encrypted = age::encrypt(&pubkey, b"AGE_SECRET=nested_dir_secret\n").unwrap();

    let temp = assert_fs::TempDir::new().unwrap();
    let sub = temp.child("keys").child("sub").child("deeper");
    sub.create_dir_all().unwrap();
    sub.child("identities.txt")
        .write_str(&format!("# comment line\n{}\n", secret))
        .unwrap();

    let config = temp.child("settingspec.toml");
    config
        .write_str(
            r#"
[spec]
envfile.default = ".env.age"
decryption.key.path = "keys"

[settings]
secret.default.env = "AGE_SECRET"
"#,
        )
        .unwrap();

    temp.child(".env.age").write_binary(&encrypted).unwrap();

    let mut cmd = Command::cargo_bin("settingspec").unwrap();
    cmd.current_dir(temp.path())
        .env_remove("SETTINGSPEC_DECRYPTION_KEY")
        .arg("export")
        .assert()
        .success();

    let exported = temp.child("settings.toml");
    exported.assert(predicate::str::contains("secret = \"nested_dir_secret\""));
}

#[test]
fn test_age_decryption_fallback_to_key_path_array() {
    use age::secrecy::ExposeSecret;

    let key = age::x25519::Identity::generate();
    let pubkey = key.to_public();
    let key_str = key.to_string();
    let secret = key_str.expose_secret();

    let encrypted = age::encrypt(&pubkey, b"AGE_SECRET=array_fallback_success\n").unwrap();

    let temp = assert_fs::TempDir::new().unwrap();
    let key_file = temp.child("actual_key.txt");
    key_file.write_str(secret).unwrap();

    let config = temp.child("settingspec.toml");
    config
        .write_str(
            r#"
[spec]
envfile.default = ".env.age"
decryption.key.path = ["nonexistent_key.txt", "actual_key.txt"]

[settings]
secret.default.env = "AGE_SECRET"
"#,
        )
        .unwrap();

    temp.child(".env.age").write_binary(&encrypted).unwrap();

    let mut cmd = Command::cargo_bin("settingspec").unwrap();
    cmd.current_dir(temp.path())
        .env_remove("SETTINGSPEC_DECRYPTION_KEY")
        .arg("export")
        .assert()
        .success();

    let exported = temp.child("settings.toml");
    exported.assert(predicate::str::contains(
        "secret = \"array_fallback_success\"",
    ));
}

#[test]
fn test_age_decryption_armored_and_header_detection() {
    use age::secrecy::ExposeSecret;

    let key = age::x25519::Identity::generate();
    let pubkey = key.to_public();
    let key_str = key.to_string();
    let secret = key_str.expose_secret();

    // Encrypt armored, and save as regular .env without .age extension
    let armored = age::encrypt_and_armor(&pubkey, b"AGE_SECRET=armored_detected\n").unwrap();

    let temp = assert_fs::TempDir::new().unwrap();
    let config = temp.child("settingspec.toml");
    config
        .write_str(
            r#"
[spec]
envfile.default = ".env"
decryption.key.path = "key.txt"

[settings]
secret.default.env = "AGE_SECRET"
"#,
        )
        .unwrap();

    temp.child("key.txt").write_str(secret).unwrap();
    temp.child(".env").write_str(&armored).unwrap();

    let mut cmd = Command::cargo_bin("settingspec").unwrap();
    cmd.current_dir(temp.path())
        .env_remove("SETTINGSPEC_DECRYPTION_KEY")
        .arg("export")
        .assert()
        .success();

    let exported = temp.child("settings.toml");
    exported.assert(predicate::str::contains("secret = \"armored_detected\""));
}

#[test]
fn test_age_decryption_stdin() {
    use age::secrecy::ExposeSecret;

    let key = age::x25519::Identity::generate();
    let pubkey = key.to_public();
    let key_str = key.to_string();
    let secret = key_str.expose_secret();

    let encrypted = age::encrypt(&pubkey, b"STDIN_SECRET=from_stdin_age\n").unwrap();

    let temp = assert_fs::TempDir::new().unwrap();
    let config = temp.child("settingspec.toml");
    config
        .write_str(
            r#"
[spec]
profile.options = ["prod"]
profile.default = "prod"
envfile.prod = "-"

[settings]
secret.default.env = "STDIN_SECRET"
"#,
        )
        .unwrap();

    let mut cmd = Command::cargo_bin("settingspec").unwrap();
    cmd.current_dir(temp.path())
        .env("SETTINGSPEC_DECRYPTION_KEY", secret)
        .write_stdin(encrypted)
        .arg("export")
        .assert()
        .success();

    let exported = temp.child("settings.toml");
    exported.assert(predicate::str::contains("secret = \"from_stdin_age\""));
}

#[test]
fn test_age_decryption_env_var_precedence_over_key_path() {
    use age::secrecy::ExposeSecret;

    let valid_key = age::x25519::Identity::generate();
    let pubkey = valid_key.to_public();
    let valid_key_str = valid_key.to_string();
    let valid_secret = valid_key_str.expose_secret();

    let other_key = age::x25519::Identity::generate();
    let other_key_str = other_key.to_string();
    let other_secret = other_key_str.expose_secret();

    let encrypted = age::encrypt(&pubkey, b"AGE_SECRET=value\n").unwrap();

    let temp = assert_fs::TempDir::new().unwrap();
    let key_file = temp.child("valid_key.txt");
    key_file.write_str(valid_secret).unwrap();

    let config = temp.child("settingspec.toml");
    config
        .write_str(
            r#"
[spec]
envfile.default = ".env.age"
decryption.key.path = "valid_key.txt"

[settings]
secret.default.env = "AGE_SECRET"
"#,
        )
        .unwrap();

    temp.child(".env.age").write_binary(&encrypted).unwrap();

    // When SETTINGSPEC_DECRYPTION_KEY is set to wrong key, fallback path is NOT consulted!
    let mut cmd = Command::cargo_bin("settingspec").unwrap();
    cmd.current_dir(temp.path())
        .env("SETTINGSPEC_DECRYPTION_KEY", other_secret)
        .arg("check")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "Failed to decrypt age-encrypted environment file '.env.age' for profile 'default'",
        ));
}

#[test]
fn test_age_decryption_empty_env_var_falls_back_to_key_path() {
    use age::secrecy::ExposeSecret;

    let key = age::x25519::Identity::generate();
    let pubkey = key.to_public();
    let key_str = key.to_string();
    let secret = key_str.expose_secret();

    let encrypted = age::encrypt(&pubkey, b"AGE_SECRET=fallback_on_empty\n").unwrap();

    let temp = assert_fs::TempDir::new().unwrap();
    let key_file = temp.child("valid_key.txt");
    key_file.write_str(secret).unwrap();

    let config = temp.child("settingspec.toml");
    config
        .write_str(
            r#"
[spec]
envfile.default = ".env.age"
decryption.key.path = "valid_key.txt"

[settings]
secret.default.env = "AGE_SECRET"
"#,
        )
        .unwrap();

    temp.child(".env.age").write_binary(&encrypted).unwrap();

    // When SETTINGSPEC_DECRYPTION_KEY is empty, it falls back to decryption.key.path!
    let mut cmd = Command::cargo_bin("settingspec").unwrap();
    cmd.current_dir(temp.path())
        .env("SETTINGSPEC_DECRYPTION_KEY", "")
        .arg("export")
        .assert()
        .success();

    let exported = temp.child("settings.toml");
    exported.assert(predicate::str::contains("secret = \"fallback_on_empty\""));
}

#[test]
fn test_age_decryption_failure_identifies_profile_and_file() {
    let temp = assert_fs::TempDir::new().unwrap();
    let config = temp.child("settingspec.toml");
    config
        .write_str(
            r#"
[spec]
profile.options = ["dev", "stage"]
profile.default = "stage"
envfile.stage = "stage_secrets.age"

[settings]
secret.default.val = "default_val"
"#,
        )
        .unwrap();

    temp.child("stage_secrets.age")
        .write_str("not_a_valid_age_payload")
        .unwrap();

    let mut cmd = Command::cargo_bin("settingspec").unwrap();
    cmd.current_dir(temp.path())
        .env_remove("SETTINGSPEC_DECRYPTION_KEY")
        .arg("check")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "Failed to decrypt age-encrypted environment file 'stage_secrets.age' for profile 'stage'",
        ));
}

#[test]
fn test_check_validates_age_decryption_for_active_profile() {
    use age::secrecy::ExposeSecret;

    let key = age::x25519::Identity::generate();
    let pubkey = key.to_public();
    let key_str = key.to_string();
    let secret = key_str.expose_secret();

    let encrypted = age::encrypt(&pubkey, b"STAGE_VAR=ok\n").unwrap();

    let temp = assert_fs::TempDir::new().unwrap();
    let config = temp.child("settingspec.toml");
    config
        .write_str(
            r#"
[spec]
profile.options = ["dev", "stage"]
profile.default = "stage"
envfile.stage = ".env.stage.age"
decryption.key.path = "key.txt"

[settings]
key.default.env = "STAGE_VAR"
"#,
        )
        .unwrap();

    temp.child(".env.stage.age")
        .write_binary(&encrypted)
        .unwrap();
    temp.child("key.txt").write_str(secret).unwrap();

    let mut cmd = Command::cargo_bin("settingspec").unwrap();
    cmd.current_dir(temp.path())
        .env_remove("SETTINGSPEC_DECRYPTION_KEY")
        .arg("check")
        .assert()
        .success();
}

#[test]
fn test_age_decryption_with_ssh_key() {
    const SSH_PUBKEY: &str =
        "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIHsKLqeplhpW+uObz5dvMgjz1OxfM/XXUB+VHtZ6isGN";
    const SSH_PRIVKEY: &str = "-----BEGIN OPENSSH PRIVATE KEY-----
b3BlbnNzaC1rZXktdjEAAAAABG5vbmUAAAAEbm9uZQAAAAAAAAABAAAAMwAAAAtzc2gtZW
QyNTUxOQAAACB7Ci6nqZYaVvrjm8+XbzII89TsXzP111AflR7WeorBjQAAAJCfEwtqnxML
agAAAAtzc2gtZWQyNTUxOQAAACB7Ci6nqZYaVvrjm8+XbzII89TsXzP111AflR7WeorBjQ
AAAEADBJvjZT8X6JRJI8xVq/1aU8nMVgOtVnmdwqWwrSlXG3sKLqeplhpW+uObz5dvMgjz
1OxfM/XXUB+VHtZ6isGNAAAADHN0cjRkQGNhcmJvbgE=
-----END OPENSSH PRIVATE KEY-----";

    let pubkey: age::ssh::Recipient = SSH_PUBKEY.parse().unwrap();
    let encrypted = age::encrypt(&pubkey, b"SSH_SECRET=ssh_decrypted_value\n").unwrap();

    let temp = assert_fs::TempDir::new().unwrap();
    let config = temp.child("settingspec.toml");
    config
        .write_str(
            r#"
[spec]
envfile.default = ".env.age"
decryption.key.path = "~/.ssh/"

[settings]
secret.default.env = "SSH_SECRET"
"#,
        )
        .unwrap();

    temp.child(".env.age").write_binary(&encrypted).unwrap();

    // Mock HOME directory so ~/.ssh/ expands to temp.path()/.ssh
    let fake_home = temp.child("home");
    fake_home.create_dir_all().unwrap();
    let ssh_dir = fake_home.child(".ssh");
    ssh_dir.create_dir_all().unwrap();
    ssh_dir.child("id_ed25519").write_str(SSH_PRIVKEY).unwrap();

    let mut cmd = Command::cargo_bin("settingspec").unwrap();
    cmd.current_dir(temp.path())
        .env("HOME", fake_home.path())
        .env_remove("SETTINGSPEC_DECRYPTION_KEY")
        .arg("export")
        .assert()
        .success();

    let exported = temp.child("settings.toml");
    exported.assert(predicate::str::contains("secret = \"ssh_decrypted_value\""));
}

#[test]
fn test_age_decryption_passphrase_protected_identity_skipped() {
    const SSH_PUBKEY: &str =
        "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIHsKLqeplhpW+uObz5dvMgjz1OxfM/XXUB+VHtZ6isGN";
    const ENCRYPTED_SSH_KEY: &str = "-----BEGIN OPENSSH PRIVATE KEY-----
b3BlbnNzaC1rZXktdjEAAAAACmFlczI1Ni1jYmMAAAAGYmNyeXB0AAAAGAAAABC0OgNmiw
QW/kJ8kCmmTA2TAAAAEAAAAAEAAAAzAAAAC3NzaC1lZDI1NTE5AAAAIHsKLqeplhpW+uOb
z5dvMgjz1OxfM/XXUB+VHtZ6isGNAAAAkPhBKsZoNmaeuWYJQxOl+ofEmue/sFJnW+4IOt
oTrS/orMBJ4b/phQcv/ejWYJ4RYYVhSLiI6hf0KwNGefxI90E8iG/yDOKcrxb34tqDEYrY
FARDaJVRd9QtWLEqoP7pgdBR2BTP7aK1y6Mx3eFDgiQI9f/0Sjxd8V0apOPXv4i4kuQ1Nt
LF7kNlDznn/nyZlg==
-----END OPENSSH PRIVATE KEY-----";

    let pubkey: age::ssh::Recipient = SSH_PUBKEY.parse().unwrap();
    let encrypted = age::encrypt(&pubkey, b"SECRET=value\n").unwrap();

    let temp = assert_fs::TempDir::new().unwrap();
    let config = temp.child("settingspec.toml");
    config
        .write_str(
            r#"
[spec]
envfile.default = ".env.age"
decryption.key.path = "keys"

[settings]
secret.default.env = "SECRET"
"#,
        )
        .unwrap();

    temp.child(".env.age").write_binary(&encrypted).unwrap();
    let keys_dir = temp.child("keys");
    keys_dir.create_dir_all().unwrap();
    keys_dir
        .child("encrypted_key")
        .write_str(ENCRYPTED_SSH_KEY)
        .unwrap();

    // The encrypted identity MUST be skipped, and decryption fails with error identifying profile and file
    let mut cmd = Command::cargo_bin("settingspec").unwrap();
    cmd.current_dir(temp.path())
        .env_remove("SETTINGSPEC_DECRYPTION_KEY")
        .arg("check")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "Failed to decrypt age-encrypted environment file '.env.age' for profile 'default'",
        ));
}

#[test]
fn test_default_envfile_not_overwritten_by_active_profile() {
    let temp = assert_fs::TempDir::new().unwrap();
    let config = temp.child("settingspec.toml");
    config
        .write_str(
            r#"
[spec]
profile.options = ["dev", "prod"]
profile.default = "dev"
envfile.default = ".env"
envfile.prod = ".env.prod"

[settings]
shared.default.env = "VAR_SHARED"
default_only.default.env = "VAR_DEFAULT_ONLY"
"#,
        )
        .unwrap();

    temp.child(".env")
        .write_str("VAR_DEFAULT_ONLY=from_default\nVAR_SHARED=default\n")
        .unwrap();
    temp.child(".env.prod")
        .write_str("VAR_SHARED=prod\n")
        .unwrap();

    // When running with prod, .env MUST NOT be loaded (so VAR_DEFAULT_ONLY is absent and fails)
    let mut cmd = Command::cargo_bin("settingspec").unwrap();
    cmd.current_dir(temp.path())
        .env("SETTINGSPEC_PROFILE", "prod")
        .arg("check")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "Missing required environment variable 'VAR_DEFAULT_ONLY'",
        ));
}

#[test]
fn test_default_envfile_loaded_as_fallback_when_profile_envfile_declaration_is_missing() {
    let temp = assert_fs::TempDir::new().unwrap();
    let config = temp.child("settingspec.toml");
    config
        .write_str(
            r#"
[spec]
profile.options = ["dev", "prod"]
profile.default = "dev"
envfile.default = ".env"

[settings]
key.prod.env = "MY_VAR"
key.dev.val = "dev_val"
"#,
        )
        .unwrap();

    temp.child(".env")
        .write_str("MY_VAR=from_default\n")
        .unwrap();

    // When running with prod, envfile.prod is not declared, so envfile.default MUST be loaded as fallback!
    let mut cmd = Command::cargo_bin("settingspec").unwrap();
    cmd.current_dir(temp.path())
        .env("SETTINGSPEC_PROFILE", "prod")
        .arg("check")
        .assert()
        .success();
}

#[test]
fn test_profile_envfile_declared_but_file_missing_raises_error() {
    let temp = assert_fs::TempDir::new().unwrap();
    let config = temp.child("settingspec.toml");
    config
        .write_str(
            r#"
[spec]
profile.options = ["dev", "prod"]
profile.default = "dev"
envfile.default = ".env"
envfile.prod = ".env.prod"

[settings]
key.prod.env = "MY_VAR"
key.dev.val = "dev_val"
"#,
        )
        .unwrap();

    // .env exists, but .env.prod is NOT created
    temp.child(".env")
        .write_str("MY_VAR=from_default\n")
        .unwrap();

    // When running with prod, envfile.prod is declared but missing, so it MUST fail and not fall back to .env
    let mut cmd = Command::cargo_bin("settingspec").unwrap();
    cmd.current_dir(temp.path())
        .env("SETTINGSPEC_PROFILE", "prod")
        .arg("check")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "Environment file '.env.prod' not found",
        ));
}
