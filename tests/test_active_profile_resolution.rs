use assert_cmd::Command;
use assert_fs::prelude::*;
use predicates::prelude::*;

#[test]
fn test_resolve_from_default_env_var() {
    let temp = assert_fs::TempDir::new().unwrap();
    let config = temp.child("settingspec.toml");
    config
        .write_str(
            r#"
[spec]
profile.options = ["dev", "prod"]
profile.default = "dev"

[settings]
env_name.default.val = "base"
env_name.prod.val = "production"
"#,
        )
        .unwrap();

    let mut cmd = Command::cargo_bin("settingspec").unwrap();
    cmd.current_dir(temp.path())
        .env("SETTINGSPEC_PROFILE", "prod")
        .arg("export")
        .assert()
        .success();

    let exported = temp.child("settings.toml");
    exported.assert(predicate::str::contains("env_name = \"production\""));
}

#[test]
fn test_resolve_from_custom_env_var_key() {
    let temp = assert_fs::TempDir::new().unwrap();
    let config = temp.child("settingspec.toml");
    config
        .write_str(
            r#"
[spec]
profile.key = "CUSTOM_ENV_KEY"
profile.options = ["dev", "staging"]
profile.default = "dev"

[settings]
mode.default.val = "development"
mode.staging.val = "staging_mode"
"#,
        )
        .unwrap();

    let mut cmd = Command::cargo_bin("settingspec").unwrap();
    cmd.current_dir(temp.path())
        .env("CUSTOM_ENV_KEY", "staging")
        .arg("export")
        .assert()
        .success();

    let exported = temp.child("settings.toml");
    exported.assert(predicate::str::contains("mode = \"staging_mode\""));
}

#[test]
fn test_fallback_to_spec_profile_default() {
    let temp = assert_fs::TempDir::new().unwrap();
    let config = temp.child("settingspec.toml");
    config
        .write_str(
            r#"
[spec]
profile.options = ["dev", "prod"]
profile.default = "dev"

[settings]
mode.default.val = "base"
mode.dev.val = "dev_active"
mode.prod.val = "prod_active"
"#,
        )
        .unwrap();

    let mut cmd = Command::cargo_bin("settingspec").unwrap();
    cmd.current_dir(temp.path())
        .env_remove("SETTINGSPEC_PROFILE")
        .arg("export")
        .assert()
        .success();

    let exported = temp.child("settings.toml");
    exported.assert(predicate::str::contains("mode = \"dev_active\""));
}

#[test]
fn test_env_var_precedence_over_spec_profile_default() {
    let temp = assert_fs::TempDir::new().unwrap();
    let config = temp.child("settingspec.toml");
    config
        .write_str(
            r#"
[spec]
profile.options = ["dev", "prod"]
profile.default = "dev"

[settings]
mode.dev.val = "dev_val"
mode.prod.val = "prod_val"
"#,
        )
        .unwrap();

    let mut cmd = Command::cargo_bin("settingspec").unwrap();
    cmd.current_dir(temp.path())
        .env("SETTINGSPEC_PROFILE", "prod")
        .arg("export")
        .assert()
        .success();

    let exported = temp.child("settings.toml");
    exported.assert(predicate::str::contains("mode = \"prod_val\""));
}

#[test]
fn test_resolution_failure_when_no_profile_selected_and_options_defined() {
    let temp = assert_fs::TempDir::new().unwrap();
    let config = temp.child("settingspec.toml");
    config
        .write_str(
            r#"
[spec]
profile.options = ["dev", "prod"]
# profile.default is omitted, env var is not set

[settings]
key1.dev.val = "val1"
key1.prod.val = "val2"
"#,
        )
        .unwrap();

    let mut cmd = Command::cargo_bin("settingspec").unwrap();
    cmd.current_dir(temp.path())
        .env_remove("SETTINGSPEC_PROFILE")
        .arg("check")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "No profile was selected and spec.profile.options is defined",
        ));
}

#[test]
fn test_default_only_resolution_when_options_unset_and_no_profile_selected() {
    let temp = assert_fs::TempDir::new().unwrap();
    let config = temp.child("settingspec.toml");
    config
        .write_str(
            r#"
[settings]
key1.default.val = "only_default"
key2.prod.val = "prod_only"
"#,
        )
        .unwrap();

    let mut cmd = Command::cargo_bin("settingspec").unwrap();
    cmd.current_dir(temp.path())
        .env_remove("SETTINGSPEC_PROFILE")
        .arg("export")
        .assert()
        .success();

    let exported = temp.child("settings.toml");
    exported.assert(predicate::str::contains("key1 = \"only_default\""));
    // key2 is for prod, which is omitted during default-only resolution
    exported.assert(predicate::str::contains("key2").not());
}
