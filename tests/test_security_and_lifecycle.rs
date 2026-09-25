use assert_cmd::Command;
use assert_fs::prelude::*;
use predicates::prelude::*;

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

#[test]
fn test_exported_file_permission_mode_0600() {
    let temp = assert_fs::TempDir::new().unwrap();
    let config = temp.child("settingspec.toml");
    config
        .write_str(
            r#"
[spec]
export.mode = 0x600
export.file."secret_settings.toml" = true

[settings]
secret_key.default.val = "very_secret_value"
"#,
        )
        .unwrap();

    let mut cmd = Command::cargo_bin("settingspec").unwrap();
    cmd.current_dir(temp.path())
        .arg("export")
        .assert()
        .success();

    let exported = temp.child("secret_settings.toml");
    exported.assert(predicate::path::exists());

    #[cfg(unix)]
    {
        let metadata = std::fs::metadata(exported.path()).unwrap();
        let mode = metadata.permissions().mode() & 0o777;
        assert_eq!(mode, 0o600, "File permissions should be 0600 (rw-------)");
    }
}

#[test]
fn test_exported_file_permission_mode_0640() {
    let temp = assert_fs::TempDir::new().unwrap();
    let config = temp.child("settingspec.toml");
    config
        .write_str(
            r#"
[spec]
export.mode = 0x640
export.file."secret_settings.toml" = true

[settings]
secret_key.default.val = "very_secret_value"
"#,
        )
        .unwrap();

    let mut cmd = Command::cargo_bin("settingspec").unwrap();
    cmd.current_dir(temp.path())
        .arg("export")
        .assert()
        .success();

    let exported = temp.child("secret_settings.toml");
    exported.assert(predicate::path::exists());

    #[cfg(unix)]
    {
        let metadata = std::fs::metadata(exported.path()).unwrap();
        let mode = metadata.permissions().mode() & 0o777;
        assert_eq!(mode, 0o640, "File permissions should be 0640 (rw-r-----)");
    }
}

#[test]
fn test_ephemeral_secrets_cleaned_up_on_child_failure() {
    let temp = assert_fs::TempDir::new().unwrap();
    let config = temp.child("settingspec.toml");
    config
        .write_str(
            r#"
[spec]
export.keep = false
export.file."secret.json" = true

[settings]
token.default.val = "super_secret"
"#,
        )
        .unwrap();

    // Child command exits with non-zero failure code (exit 12)
    let mut cmd = Command::cargo_bin("settingspec").unwrap();
    cmd.current_dir(temp.path())
        .args(["run", "--", "sh", "-c", "exit 12"])
        .assert()
        .code(12);

    // secret.json MUST still be cleaned up even though child failed
    temp.child("secret.json").assert(predicate::path::missing());
}

#[test]
fn test_missing_settingspec_toml_fails_with_clear_error() {
    let temp = assert_fs::TempDir::new().unwrap();

    let mut cmd = Command::cargo_bin("settingspec").unwrap();
    cmd.current_dir(temp.path())
        .arg("check")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "settingspec.toml' not found in project root",
        ));
}

#[test]
fn test_missing_settings_section_fails() {
    let temp = assert_fs::TempDir::new().unwrap();
    let config = temp.child("settingspec.toml");
    config
        .write_str(
            r#"
[spec]
profile.options = ["dev", "prod"]
"#,
        )
        .unwrap();

    let mut cmd = Command::cargo_bin("settingspec").unwrap();
    cmd.current_dir(temp.path())
        .arg("check")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "Missing required '[settings]' section",
        ));
}
