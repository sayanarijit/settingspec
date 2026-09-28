use assert_cmd::Command;
use assert_fs::prelude::*;
use predicates::prelude::*;

#[test]
fn test_specification_examples_rejected() {
    // 1. _.default.val = "x" (empty key)
    let temp1 = assert_fs::TempDir::new().unwrap();
    temp1
        .child("settingspec.toml")
        .write_str(
            r#"
[settings]
_.default.val = "x"
"#,
        )
        .unwrap();
    let mut cmd1 = Command::cargo_bin("settingspec").unwrap();
    cmd1.current_dir(temp1.path())
        .arg("check")
        .assert()
        .failure()
        .stderr(predicate::str::contains("_.default.val"));

    // 2. key1.default.val = "x" (missing `_` separator)
    let temp2 = assert_fs::TempDir::new().unwrap();
    temp2
        .child("settingspec.toml")
        .write_str(
            r#"
[settings]
key1.default.val = "x"
"#,
        )
        .unwrap();
    let mut cmd2 = Command::cargo_bin("settingspec").unwrap();
    cmd2.current_dir(temp2.path())
        .arg("check")
        .assert()
        .failure()
        .stderr(predicate::str::contains("key1.default.val"));

    // 3. key1._.default._.val = "y" (more than one `_` component)
    let temp3 = assert_fs::TempDir::new().unwrap();
    temp3
        .child("settingspec.toml")
        .write_str(
            r#"
[settings]
key1._.default._.val = "y"
"#,
        )
        .unwrap();
    let mut cmd3 = Command::cargo_bin("settingspec").unwrap();
    cmd3.current_dir(temp3.path())
        .arg("check")
        .assert()
        .failure()
        .stderr(predicate::str::contains("key1._.default._.val"));

    // 4. key1._.tags.val = "z" (`tags` used as a profile)
    let temp4 = assert_fs::TempDir::new().unwrap();
    temp4
        .child("settingspec.toml")
        .write_str(
            r#"
[settings]
key1._.tags.val = "z"
"#,
        )
        .unwrap();
    let mut cmd4 = Command::cargo_bin("settingspec").unwrap();
    cmd4.current_dir(temp4.path())
        .arg("check")
        .assert()
        .failure()
        .stderr(predicate::str::contains("key1._.tags.val"));

    // 5. key1._.default = "w" (missing directive)
    let temp5 = assert_fs::TempDir::new().unwrap();
    temp5
        .child("settingspec.toml")
        .write_str(
            r#"
[settings]
key1._.default = "w"
"#,
        )
        .unwrap();
    let mut cmd5 = Command::cargo_bin("settingspec").unwrap();
    cmd5.current_dir(temp5.path())
        .arg("check")
        .assert()
        .failure()
        .stderr(predicate::str::contains("key1._.default"));
}

#[test]
fn test_setting_keys_may_contain_directive_words() {
    for kw in ["val", "env", "null", "tags"] {
        let temp = assert_fs::TempDir::new().unwrap();
        let config = temp.child("settingspec.toml");
        config
            .write_str(&format!(
                r#"
[settings]
{}._.default.val = "value"
app.{}.host._.default.val = "db.example.com"
"#,
                kw, kw
            ))
            .unwrap();

        let mut cmd = Command::cargo_bin("settingspec").unwrap();
        cmd.current_dir(temp.path()).arg("check").assert().success();
    }
}

#[test]
fn test_setting_keys_with_reserved_underscore_component_rejected() {
    let temp = assert_fs::TempDir::new().unwrap();
    let config = temp.child("settingspec.toml");
    config
        .write_str(
            r#"
[settings]
"app._.host"._.default.val = "x"
"#,
        )
        .unwrap();

    let mut cmd = Command::cargo_bin("settingspec").unwrap();
    cmd.current_dir(temp.path())
        .arg("check")
        .assert()
        .failure()
        .stderr(predicate::str::contains("app._.host"));
}

#[test]
fn test_reserved_profile_tags_in_profile_options_rejected() {
    let temp = assert_fs::TempDir::new().unwrap();
    let config = temp.child("settingspec.toml");
    config
        .write_str(
            r#"
[spec]
profile.options = ["dev", "tags"]
profile.default = "dev"

[settings]
key1._.default.val = "v1"
"#,
        )
        .unwrap();

    let mut cmd = Command::cargo_bin("settingspec").unwrap();
    cmd.current_dir(temp.path())
        .arg("check")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "Reserved profile identifier 'tags' MUST NOT be in spec.profile.options",
        ));
}

#[test]
fn test_reserved_profile_default_in_profile_options_rejected() {
    let temp = assert_fs::TempDir::new().unwrap();
    let config = temp.child("settingspec.toml");
    config
        .write_str(
            r#"
[spec]
profile.options = ["dev", "default"]
profile.default = "dev"

[settings]
key1._.default.val = "v1"
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
fn test_reserved_keyword_in_profile_default_rejected() {
    for kw in ["default", "tags"] {
        let temp = assert_fs::TempDir::new().unwrap();
        let config = temp.child("settingspec.toml");
        config
            .write_str(&format!(
                r#"
[spec]
profile.default = "{}"

[settings]
key1._.default.val = "v1"
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
                "Reserved profile name '{}' cannot be used",
                kw
            )));
    }
}

#[test]
fn test_reserved_keyword_as_active_profile_via_env_var_rejected() {
    for kw in ["default", "tags"] {
        let temp = assert_fs::TempDir::new().unwrap();
        let config = temp.child("settingspec.toml");
        config
            .write_str(
                r#"
[settings]
key1._.default.val = "v1"
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
                "Reserved profile name '{}' cannot be used",
                kw
            )));
    }
}

#[test]
fn test_reserved_keyword_in_envfile_profile_rejected() {
    let kw = "tags";
    let temp = assert_fs::TempDir::new().unwrap();
    let config = temp.child("settingspec.toml");
    config
        .write_str(&format!(
            r#"
[spec.envfile]
default = ".env"
{} = ".env.custom"

[settings]
key1._.default.val = "v1"
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
            "Reserved profile name '{}' cannot be used",
            kw
        )));
}

#[test]
fn test_directive_keywords_allowed_as_profile_names() {
    for kw in ["val", "env", "null"] {
        let temp = assert_fs::TempDir::new().unwrap();
        let config = temp.child("settingspec.toml");
        config
            .write_str(&format!(
                r#"
[spec]
profile.options = ["dev", "{}"]
profile.default = "dev"

[settings]
key1._.default.val = "base"
key1._.{}.val = "custom"
"#,
                kw, kw
            ))
            .unwrap();

        let mut cmd = Command::cargo_bin("settingspec").unwrap();
        cmd.current_dir(temp.path()).arg("check").assert().success();
    }
}

#[test]
fn test_valid_directive_usage_with_all_reserved_keywords() {
    let temp = assert_fs::TempDir::new().unwrap();
    let config = temp.child("settingspec.toml");
    config
        .write_str(
            r#"
[settings]
my_key._.default.val = "hello"
my_key._.default.env = "MY_KEY_VAR"
my_key._.tags = ["tag1", "tag2"]
cleared_key._.default.null = true
table_key._.default.val = { inner = 42 }
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
key1._.default.val = { val = 1 }
key2._.default.val = { null = true }
key3._.default.val = { tags = [1, 2, 3] }
key4._.default.val = { env = "SOME_ENV" }
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
key._.default.val = { default = { val = 1 } }
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
