use assert_cmd::Command;
use assert_fs::prelude::*;
use predicates::prelude::*;

#[test]
fn test_valid_profile_option_succeeds() {
    let temp = assert_fs::TempDir::new().unwrap();
    let config = temp.child("settingspec.toml");
    config
        .write_str(
            r#"
[spec]
profile.options = ["dev", "stage", "prod"]
profile.default = "dev"

[settings]
app.name.default.val = "MyApp"
app.port.dev.val = 8080
app.port.stage.val = 8081
app.port.prod.val = 80
"#,
        )
        .unwrap();

    let mut cmd = Command::cargo_bin("settingspec").unwrap();
    cmd.current_dir(temp.path()).arg("check").assert().success();
}

#[test]
fn test_active_profile_not_in_options_fails() {
    let temp = assert_fs::TempDir::new().unwrap();
    let config = temp.child("settingspec.toml");
    config
        .write_str(
            r#"
[spec]
profile.options = ["dev", "prod"]
profile.default = "test_env"

[settings]
app.name.default.val = "MyApp"
"#,
        )
        .unwrap();

    let mut cmd = Command::cargo_bin("settingspec").unwrap();
    cmd.current_dir(temp.path())
        .arg("check")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "not an element of spec.profile.options",
        ));
}

#[test]
fn test_unknown_profile_name_in_settings_fails() {
    let temp = assert_fs::TempDir::new().unwrap();
    let config = temp.child("settingspec.toml");
    config
        .write_str(
            r#"
[spec]
profile.options = ["dev", "prod"]
profile.default = "dev"

[settings]
key1.default.val = "val1"
key1.unknown_profile.val = "bad"
"#,
        )
        .unwrap();

    let mut cmd = Command::cargo_bin("settingspec").unwrap();
    cmd.current_dir(temp.path())
        .arg("check")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "Unknown profile 'unknown_profile'",
        ));
}

#[test]
fn test_reserved_identifier_default_in_profile_options_fails() {
    let temp = assert_fs::TempDir::new().unwrap();
    let config = temp.child("settingspec.toml");
    config
        .write_str(
            r#"
[spec]
profile.options = ["dev", "default", "prod"]
profile.default = "dev"

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
            "Reserved profile identifier 'default' MUST NOT be in spec.profile.options",
        ));
}

#[test]
fn test_strict_coverage_failure_when_profile_unmapped_and_no_default() {
    let temp = assert_fs::TempDir::new().unwrap();
    let config = temp.child("settingspec.toml");
    // key2 is missing declaration for "prod", and has no default
    config
        .write_str(
            r#"
[spec]
profile.options = ["dev", "stage", "prod"]
profile.default = "dev"

[settings]
key1.default.val = "val1"
key2.dev.val = "dev2"
key2.stage.val = "stage2"
"#,
        )
        .unwrap();

    let mut cmd = Command::cargo_bin("settingspec").unwrap();
    cmd.current_dir(temp.path())
        .arg("check")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "Strict coverage failure: setting 'key2' is missing a declaration for profile 'prod'",
        ));
}

#[test]
fn test_strict_coverage_satisfied_with_explicit_declarations() {
    let temp = assert_fs::TempDir::new().unwrap();
    let config = temp.child("settingspec.toml");
    config
        .write_str(
            r#"
[spec]
profile.options = ["dev", "stage", "prod"]
profile.default = "dev"

[settings]
key2.dev.val = "dev2"
key2.stage.val = "stage2"
key2.prod.val = "prod2"
"#,
        )
        .unwrap();

    let mut cmd = Command::cargo_bin("settingspec").unwrap();
    cmd.current_dir(temp.path()).arg("check").assert().success();
}

#[test]
fn test_profile_options_unset_allows_arbitrary_profiles() {
    let temp = assert_fs::TempDir::new().unwrap();
    let config = temp.child("settingspec.toml");
    config
        .write_str(
            r#"
[spec]
profile.default = "custom_profile"

[settings]
key1.default.val = "default_val"
key1.custom_profile.val = "custom_val"
key1.arbitrary.val = "arbitrary_val"
"#,
        )
        .unwrap();

    let mut cmd = Command::cargo_bin("settingspec").unwrap();
    cmd.current_dir(temp.path()).arg("check").assert().success();
}
