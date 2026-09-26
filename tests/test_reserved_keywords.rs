use assert_cmd::Command;
use assert_fs::prelude::*;
use predicates::prelude::*;

#[test]
fn test_reserved_keywords_as_standalone_setting_keys_rejected() {
    for kw in ["val", "env", "null", "tags"] {
        let temp = assert_fs::TempDir::new().unwrap();
        let config = temp.child("settingspec.toml");
        config
            .write_str(&format!(
                r#"
[settings]
{}.default.val = "value"
"#,
                kw
            ))
            .unwrap();

        let mut cmd = Command::cargo_bin("settingspec").unwrap();
        cmd.current_dir(temp.path())
            .arg("check")
            .assert()
            .failure()
            .stderr(predicate::str::contains(format!(
                "Reserved directive keyword '{}' cannot be used as a setting key segment or profile name",
                kw
            )));
    }
}

#[test]
fn test_specification_examples_rejected() {
    // 1. env.default.val = "x"
    let temp1 = assert_fs::TempDir::new().unwrap();
    temp1
        .child("settingspec.toml")
        .write_str(
            r#"
[settings]
env.default.val = "x"
"#,
        )
        .unwrap();
    let mut cmd1 = Command::cargo_bin("settingspec").unwrap();
    cmd1.current_dir(temp1.path())
        .arg("check")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "Reserved directive keyword 'env' cannot be used as a setting key segment or profile name",
        ));

    // 2. group1.tags.prod.val = "y"
    let temp2 = assert_fs::TempDir::new().unwrap();
    temp2
        .child("settingspec.toml")
        .write_str(
            r#"
[settings]
group1.tags.prod.val = "y"
"#,
        )
        .unwrap();
    let mut cmd2 = Command::cargo_bin("settingspec").unwrap();
    cmd2.current_dir(temp2.path())
        .arg("check")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "Reserved directive keyword 'tags' cannot be used as a setting key segment or profile name",
        ));

    // 3. val.default.env = "Z"
    let temp3 = assert_fs::TempDir::new().unwrap();
    temp3
        .child("settingspec.toml")
        .write_str(
            r#"
[settings]
val.default.env = "Z"
"#,
        )
        .unwrap();
    let mut cmd3 = Command::cargo_bin("settingspec").unwrap();
    cmd3.current_dir(temp3.path())
        .arg("check")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "Reserved directive keyword 'val' cannot be used as a setting key segment or profile name",
        ));
}

#[test]
fn test_reserved_keywords_as_intermediate_key_segments_rejected() {
    for kw in ["val", "env", "null", "tags"] {
        let temp = assert_fs::TempDir::new().unwrap();
        let config = temp.child("settingspec.toml");
        config
            .write_str(&format!(
                r#"
[settings]
app.{}.host.default.val = "db.example.com"
"#,
                kw
            ))
            .unwrap();

        let mut cmd = Command::cargo_bin("settingspec").unwrap();
        cmd.current_dir(temp.path())
            .arg("check")
            .assert()
            .failure()
            .stderr(predicate::str::contains(format!(
                "Reserved directive keyword '{}' cannot be used as a setting key segment or profile name",
                kw
            )));
    }
}

#[test]
fn test_reserved_keywords_as_profile_name_without_options_rejected() {
    for kw in ["val", "env", "null", "tags"] {
        let temp = assert_fs::TempDir::new().unwrap();
        let config = temp.child("settingspec.toml");
        config
            .write_str(&format!(
                r#"
[settings]
database.host.{}.val = "localhost"
"#,
                kw
            ))
            .unwrap();

        let mut cmd = Command::cargo_bin("settingspec").unwrap();
        cmd.current_dir(temp.path())
            .arg("check")
            .assert()
            .failure()
            .stderr(predicate::str::contains(format!(
                "Reserved directive keyword '{}' cannot be used as a setting key segment or profile name",
                kw
            )));
    }
}

#[test]
fn test_reserved_keywords_as_profile_name_with_options_rejected() {
    for kw in ["val", "env", "null", "tags"] {
        let temp = assert_fs::TempDir::new().unwrap();
        let config = temp.child("settingspec.toml");
        config
            .write_str(&format!(
                r#"
[spec]
profile.options = ["dev", "prod"]
profile.default = "dev"

[settings]
database.host.default.val = "localhost"
database.host.{}.val = "remote"
"#,
                kw
            ))
            .unwrap();

        let mut cmd = Command::cargo_bin("settingspec").unwrap();
        cmd.current_dir(temp.path())
            .arg("check")
            .assert()
            .failure()
            .stderr(predicate::str::contains(format!(
                "Reserved directive keyword '{}' cannot be used as a setting key segment or profile name",
                kw
            )));
    }
}

#[test]
fn test_reserved_keyword_in_profile_options_rejected() {
    for kw in ["val", "env", "null", "tags"] {
        let temp = assert_fs::TempDir::new().unwrap();
        let config = temp.child("settingspec.toml");
        config
            .write_str(&format!(
                r#"
[spec]
profile.options = ["dev", "{}"]
profile.default = "dev"

[settings]
key1.default.val = "v1"
"#,
                kw
            ))
            .unwrap();

        let mut cmd = Command::cargo_bin("settingspec").unwrap();
        cmd.current_dir(temp.path())
            .arg("check")
            .assert()
            .failure()
            .stderr(predicate::str::contains(format!(
                "Reserved directive keyword '{}' cannot be used as a setting key segment or profile name",
                kw
            )));
    }
}

#[test]
fn test_reserved_keyword_in_profile_default_rejected() {
    for kw in ["val", "env", "null", "tags"] {
        let temp = assert_fs::TempDir::new().unwrap();
        let config = temp.child("settingspec.toml");
        config
            .write_str(&format!(
                r#"
[spec]
profile.default = "{}"

[settings]
key1.default.val = "v1"
"#,
                kw
            ))
            .unwrap();

        let mut cmd = Command::cargo_bin("settingspec").unwrap();
        cmd.current_dir(temp.path())
            .arg("check")
            .assert()
            .failure()
            .stderr(predicate::str::contains(format!(
                "Reserved directive keyword '{}' cannot be used as a setting key segment or profile name",
                kw
            )));
    }
}

#[test]
fn test_reserved_keyword_as_active_profile_via_env_var_rejected() {
    for kw in ["val", "env", "null", "tags"] {
        let temp = assert_fs::TempDir::new().unwrap();
        let config = temp.child("settingspec.toml");
        config
            .write_str(
                r#"
[settings]
key1.default.val = "v1"
"#,
            )
            .unwrap();

        let mut cmd = Command::cargo_bin("settingspec").unwrap();
        cmd.current_dir(temp.path())
            .env("SETTINGSPEC_PROFILE", kw)
            .arg("check")
            .assert()
            .failure()
            .stderr(predicate::str::contains(format!(
                "Reserved directive keyword '{}' cannot be used as a setting key segment or profile name",
                kw
            )));
    }
}

#[test]
fn test_reserved_keyword_in_envfile_profile_rejected() {
    for kw in ["val", "env", "null", "tags"] {
        let temp = assert_fs::TempDir::new().unwrap();
        let config = temp.child("settingspec.toml");
        config
            .write_str(&format!(
                r#"
[spec.envfile]
default = ".env"
{} = ".env.custom"

[settings]
key1.default.val = "v1"
"#,
                kw
            ))
            .unwrap();

        let mut cmd = Command::cargo_bin("settingspec").unwrap();
        cmd.current_dir(temp.path())
            .arg("check")
            .assert()
            .failure()
            .stderr(predicate::str::contains(format!(
                "Reserved directive keyword '{}' cannot be used as a setting key segment or profile name",
                kw
            )));
    }
}

#[test]
fn test_similar_names_containing_keywords_are_allowed() {
    let temp = assert_fs::TempDir::new().unwrap();
    let config = temp.child("settingspec.toml");
    config
        .write_str(
            r#"
[spec]
profile.options = ["evaluate", "my_env", "nullable", "tagset"]
profile.default = "evaluate"

[settings]
evaluation.default.val = "base1"
evaluation.my_env.val = "ok1"
environment.var.default.val = "base2"
environment.var.nullable.val = "ok2"
validator.eval.default.val = "base3"
validator.eval.tagset.val = "ok3"
tag_manager.tags_list.default.val = "base4"
tag_manager.tags_list.evaluate.val = "ok4"
"#,
        )
        .unwrap();

    let mut cmd = Command::cargo_bin("settingspec").unwrap();
    cmd.current_dir(temp.path()).arg("check").assert().success();
}

#[test]
fn test_valid_directive_usage_with_all_reserved_keywords() {
    let temp = assert_fs::TempDir::new().unwrap();
    let config = temp.child("settingspec.toml");
    config
        .write_str(
            r#"
[settings]
my_key.default.val = "hello"
my_key.default.env = "MY_KEY_VAR"
my_key.default.tags = ["tag1", "tag2"]
cleared_key.default.null = true
table_key.default.val = { inner = 42 }
"#,
        )
        .unwrap();

    let mut cmd = Command::cargo_bin("settingspec").unwrap();
    cmd.current_dir(temp.path()).arg("check").assert().success();
}

#[test]
fn test_reserved_keywords_as_table_values_in_val() {
    let temp = assert_fs::TempDir::new().unwrap();
    let config = temp.child("settingspec.toml");
    config
        .write_str(
            r#"
[settings]
key1.default.val = { val = 1 }
key2.default.val = { null = true }
key3.default.val = { tags = [1, 2, 3] }
key4.default.val = { env = "SOME_ENV" }
"#,
        )
        .unwrap();

    let mut cmd = Command::cargo_bin("settingspec").unwrap();
    cmd.current_dir(temp.path())
        .arg("export")
        .assert()
        .success();

    let exported = temp.child("settings.toml");
    exported.assert(predicate::str::contains("[key1]"));
    exported.assert(predicate::str::contains("val = 1"));
    exported.assert(predicate::str::contains("[key2]"));
    exported.assert(predicate::str::contains("null = true"));
    exported.assert(predicate::str::contains("[key3]"));
    exported.assert(predicate::str::contains("tags = ["));
    exported.assert(predicate::str::contains("1,"));
    exported.assert(predicate::str::contains("2,"));
    exported.assert(predicate::str::contains("3,"));
    exported.assert(predicate::str::contains("[key4]"));
    exported.assert(predicate::str::contains("env = \"SOME_ENV\""));
}

#[test]
fn test_nested_default_in_table_val() {
    let temp = assert_fs::TempDir::new().unwrap();
    let config = temp.child("settingspec.toml");
    config
        .write_str(
            r#"
[spec.export.file]
"settings.toml" = true
"settings.json" = true

[settings]
key.default.val = { default = { val = 1 } }
"#,
        )
        .unwrap();

    let mut cmd = Command::cargo_bin("settingspec").unwrap();
    cmd.current_dir(temp.path())
        .arg("export")
        .assert()
        .success();

    let exported_toml = temp.child("settings.toml");
    exported_toml.assert(predicate::str::contains("[key.default]"));
    exported_toml.assert(predicate::str::contains("val = 1"));

    let exported_json = temp.child("settings.json");
    exported_json.assert(predicate::str::contains(r#""key""#));
    exported_json.assert(predicate::str::contains(r#""default""#));
    exported_json.assert(predicate::str::contains(r#""val": 1"#));
}
