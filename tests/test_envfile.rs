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
