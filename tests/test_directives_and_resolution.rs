use assert_cmd::Command;
use assert_fs::prelude::*;
use predicates::prelude::*;

#[test]
fn test_all_supported_val_types() {
    let temp = assert_fs::TempDir::new().unwrap();
    let config = temp.child("settingspec.toml");
    config
        .write_str(
            r#"
[settings]
str_val.default.val = "hello world"
int_val.default.val = 42
float_val.default.val = 3.1415
bool_val.default.val = true
datetime_val.default.val = 2026-09-25T12:00:00Z
array_val.default.val = [1, 2, 3]
table_val.default.val = { host = "localhost", port = 5432 }
"#,
        )
        .unwrap();

    let mut cmd = Command::cargo_bin("settingspec").unwrap();
    cmd.current_dir(temp.path())
        .arg("export")
        .assert()
        .success();

    let exported = temp.child("settings.toml");
    exported.assert(predicate::str::contains("str_val = \"hello world\""));
    exported.assert(predicate::str::contains("int_val = 42"));
    exported.assert(predicate::str::contains("float_val = 3.1415"));
    exported.assert(predicate::str::contains("bool_val = true"));
    exported.assert(predicate::str::contains(
        "datetime_val = 2026-09-25T12:00:00Z",
    ));
    exported.assert(predicate::str::contains("array_val = ["));
    exported.assert(predicate::str::contains("1,"));
    exported.assert(predicate::str::contains("host = \"localhost\""));
}

#[test]
fn test_env_override_with_val_fallback() {
    let temp = assert_fs::TempDir::new().unwrap();
    let config = temp.child("settingspec.toml");
    config
        .write_str(
            r#"
[settings]
secret.default.val = "default_secret"
secret.default.env = "MY_SECRET_VAR"
"#,
        )
        .unwrap();

    // 1. Without env var: falls back to val
    let mut cmd1 = Command::cargo_bin("settingspec").unwrap();
    cmd1.current_dir(temp.path())
        .env_remove("MY_SECRET_VAR")
        .arg("export")
        .assert()
        .success();
    temp.child("settings.toml")
        .assert(predicate::str::contains("secret = \"default_secret\""));

    // 2. With env var: uses env var
    let mut cmd2 = Command::cargo_bin("settingspec").unwrap();
    cmd2.current_dir(temp.path())
        .env("MY_SECRET_VAR", "env_provided_secret")
        .arg("export")
        .assert()
        .success();
    temp.child("settings.toml")
        .assert(predicate::str::contains("secret = \"env_provided_secret\""));
}

#[test]
fn test_env_override_with_null_fallback() {
    let temp = assert_fs::TempDir::new().unwrap();
    let config = temp.child("settingspec.toml");
    config
        .write_str(
            r#"
[spec]
export.file."settings.json" = true

[settings]
optional_key.default.null = true
optional_key.default.env = "OPTIONAL_VAR"
"#,
        )
        .unwrap();

    // 1. Without env var: resolves to null
    let mut cmd1 = Command::cargo_bin("settingspec").unwrap();
    cmd1.current_dir(temp.path())
        .env_remove("OPTIONAL_VAR")
        .arg("export")
        .assert()
        .success();
    temp.child("settings.json")
        .assert(predicate::str::contains("\"optional_key\": null"));

    // 2. With env var: resolves to env var value
    let mut cmd2 = Command::cargo_bin("settingspec").unwrap();
    cmd2.current_dir(temp.path())
        .env("OPTIONAL_VAR", "provided_value")
        .arg("export")
        .assert()
        .success();
    temp.child("settings.json").assert(predicate::str::contains(
        "\"optional_key\": \"provided_value\"",
    ));
}

#[test]
fn test_env_without_val_or_null_fails_when_absent() {
    let temp = assert_fs::TempDir::new().unwrap();
    let config = temp.child("settingspec.toml");
    config
        .write_str(
            r#"
[settings]
required_secret.default.env = "UNSET_REQUIRED_VAR"
"#,
        )
        .unwrap();

    let mut cmd = Command::cargo_bin("settingspec").unwrap();
    cmd.current_dir(temp.path())
        .env_remove("UNSET_REQUIRED_VAR")
        .arg("check")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "Missing required environment variable 'UNSET_REQUIRED_VAR'",
        ));
}

#[test]
fn test_null_cannot_coexist_with_val() {
    let temp = assert_fs::TempDir::new().unwrap();
    let config = temp.child("settingspec.toml");
    config
        .write_str(
            r#"
[settings]
bad_key.default.val = "some_val"
bad_key.default.null = true
"#,
        )
        .unwrap();

    let mut cmd = Command::cargo_bin("settingspec").unwrap();
    cmd.current_dir(temp.path())
        .arg("check")
        .assert()
        .failure()
        .stderr(predicate::str::contains("cannot coexist with 'val'"));
}

#[test]
fn test_null_false_is_rejected() {
    let temp = assert_fs::TempDir::new().unwrap();
    let config = temp.child("settingspec.toml");
    config
        .write_str(
            r#"
[settings]
bad_null.default.null = false
"#,
        )
        .unwrap();

    let mut cmd = Command::cargo_bin("settingspec").unwrap();
    cmd.current_dir(temp.path())
        .arg("check")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Directive 'null' must be true"));
}

#[test]
fn test_tags_are_not_exported_as_values() {
    let temp = assert_fs::TempDir::new().unwrap();
    let config = temp.child("settingspec.toml");
    config
        .write_str(
            r#"
[settings]
api_key.default.val = "key123"
api_key.default.tags = ["sensitive", "auth"]
"#,
        )
        .unwrap();

    let mut cmd = Command::cargo_bin("settingspec").unwrap();
    cmd.current_dir(temp.path())
        .arg("export")
        .assert()
        .success();

    let exported = temp.child("settings.toml");
    exported.assert(predicate::str::contains("api_key = \"key123\""));
    exported.assert(predicate::str::contains("sensitive").not());
    exported.assert(predicate::str::contains("auth").not());
}

#[test]
fn test_unknown_directive_rejected() {
    let temp = assert_fs::TempDir::new().unwrap();
    let config = temp.child("settingspec.toml");
    config
        .write_str(
            r#"
[settings]
key1.default.value = "val1"
"#,
        )
        .unwrap();

    let mut cmd = Command::cargo_bin("settingspec").unwrap();
    cmd.current_dir(temp.path())
        .arg("check")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Invalid directive 'value'"));
}

#[test]
fn test_resolution_precedence_profile_env_over_profile_val_over_default() {
    let temp = assert_fs::TempDir::new().unwrap();
    let config = temp.child("settingspec.toml");
    config
        .write_str(
            r#"
[spec]
profile.options = ["dev", "prod"]
profile.default = "prod"

[settings]
tier.default.val = "tier_default"
tier.prod.val = "tier_prod_val"
tier.prod.env = "TIER_PROD_ENV"
"#,
        )
        .unwrap();

    // 1. With TIER_PROD_ENV set: returns env var
    let mut cmd1 = Command::cargo_bin("settingspec").unwrap();
    cmd1.current_dir(temp.path())
        .env("TIER_PROD_ENV", "tier_prod_env_active")
        .arg("export")
        .assert()
        .success();
    temp.child("settings.toml")
        .assert(predicate::str::contains("tier = \"tier_prod_env_active\""));

    // 2. Without TIER_PROD_ENV: falls back to tier.prod.val
    let mut cmd2 = Command::cargo_bin("settingspec").unwrap();
    cmd2.current_dir(temp.path())
        .env_remove("TIER_PROD_ENV")
        .arg("export")
        .assert()
        .success();
    temp.child("settings.toml")
        .assert(predicate::str::contains("tier = \"tier_prod_val\""));
}
