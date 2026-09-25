use assert_cmd::Command;
use assert_fs::prelude::*;
use predicates::prelude::*;

#[test]
fn test_boolean_filter_true_exports_all() {
    let temp = assert_fs::TempDir::new().unwrap();
    let config = temp.child("settingspec.toml");
    config
        .write_str(
            r#"
[spec.export.file]
"all_settings.toml" = true

[settings]
k1.default.val = "v1"
k2.default.val = "v2"
"#,
        )
        .unwrap();

    let mut cmd = Command::cargo_bin("settingspec").unwrap();
    cmd.current_dir(temp.path())
        .arg("export")
        .assert()
        .success();

    let target = temp.child("all_settings.toml");
    target.assert(predicate::path::exists());
    target.assert(predicate::str::contains("k1 = \"v1\""));
    target.assert(predicate::str::contains("k2 = \"v2\""));
}

#[test]
fn test_boolean_filter_false_disables_export() {
    let temp = assert_fs::TempDir::new().unwrap();
    let config = temp.child("settingspec.toml");
    config
        .write_str(
            r#"
[spec.export.file]
"disabled.toml" = false
"enabled.toml" = true

[settings]
k1.default.val = "v1"
"#,
        )
        .unwrap();

    let mut cmd = Command::cargo_bin("settingspec").unwrap();
    cmd.current_dir(temp.path())
        .arg("export")
        .assert()
        .success();

    temp.child("disabled.toml")
        .assert(predicate::path::missing());
    temp.child("enabled.toml").assert(predicate::path::exists());
}

#[test]
fn test_default_export_target_when_omitted() {
    let temp = assert_fs::TempDir::new().unwrap();
    let config = temp.child("settingspec.toml");
    config
        .write_str(
            r#"
[settings]
k1.default.val = "v1"
"#,
        )
        .unwrap();

    let mut cmd = Command::cargo_bin("settingspec").unwrap();
    cmd.current_dir(temp.path())
        .arg("export")
        .assert()
        .success();

    let default_file = temp.child("settings.toml");
    default_file.assert(predicate::path::exists());
    default_file.assert(predicate::str::contains("k1 = \"v1\""));
}

#[test]
fn test_default_export_target_omitted_when_export_env_specified() {
    let temp = assert_fs::TempDir::new().unwrap();
    let config = temp.child("settingspec.toml");
    config
        .write_str(
            r#"
[spec]
export.env = true

[settings]
k1.default.val = "v1"
"#,
        )
        .unwrap();

    let mut cmd = Command::cargo_bin("settingspec").unwrap();
    cmd.current_dir(temp.path())
        .arg("export")
        .assert()
        .success();

    let default_file = temp.child("settings.toml");
    default_file.assert(predicate::path::missing());

    // Also test with prefix string
    config
        .write_str(
            r#"
[spec.export]
env = "PREFIX_"

[settings]
k1.default.val = "v1"
"#,
        )
        .unwrap();

    let mut cmd2 = Command::cargo_bin("settingspec").unwrap();
    cmd2.current_dir(temp.path())
        .arg("export")
        .assert()
        .success();

    let default_file2 = temp.child("settings.toml");
    default_file2.assert(predicate::path::missing());
}

#[test]
fn test_group_and_subgroup_inclusion() {
    let temp = assert_fs::TempDir::new().unwrap();
    let config = temp.child("settingspec.toml");
    config
        .write_str(
            r#"
[spec.export.file]
"target.toml" = { group1 = true, "group2.subgroup1" = true }

[settings]
group1.k1.default.val = "g1_k1"
group1.k2.default.val = "g1_k2"
group2.subgroup1.k1.default.val = "g2_sg1_k1"
group2.subgroup2.k1.default.val = "g2_sg2_k1"
other.k1.default.val = "other_val"
"#,
        )
        .unwrap();

    let mut cmd = Command::cargo_bin("settingspec").unwrap();
    cmd.current_dir(temp.path())
        .arg("export")
        .assert()
        .success();

    let target = temp.child("target.toml");
    target.assert(predicate::str::contains("g1_k1"));
    target.assert(predicate::str::contains("g1_k2"));
    target.assert(predicate::str::contains("g2_sg1_k1"));
    // group2.subgroup2 and other should not be included
    target.assert(predicate::str::contains("g2_sg2_k1").not());
    target.assert(predicate::str::contains("other_val").not());
}

#[test]
fn test_exact_key_inclusion() {
    let temp = assert_fs::TempDir::new().unwrap();
    let config = temp.child("settingspec.toml");
    config
        .write_str(
            r#"
[spec.export.file]
"target.toml" = { "group.subgroup.k1" = true }

[settings]
group.subgroup.k1.default.val = "included_k1"
group.subgroup.k2.default.val = "excluded_k2"
"#,
        )
        .unwrap();

    let mut cmd = Command::cargo_bin("settingspec").unwrap();
    cmd.current_dir(temp.path())
        .arg("export")
        .assert()
        .success();

    let target = temp.child("target.toml");
    target.assert(predicate::str::contains("included_k1"));
    target.assert(predicate::str::contains("excluded_k2").not());
}

#[test]
fn test_subgroup_exclusion_implicit_parent_inclusion() {
    let temp = assert_fs::TempDir::new().unwrap();
    let config = temp.child("settingspec.toml");
    config
        .write_str(
            r#"
[spec.export.file]
"target.toml" = { "group3.subgroup" = false }

[settings]
group3.subgroup.k1.default.val = "excluded_subgroup_val"
group3.other_sub.k1.default.val = "included_other_val"
unrelated.k1.default.val = "unrelated_val"
"#,
        )
        .unwrap();

    let mut cmd = Command::cargo_bin("settingspec").unwrap();
    cmd.current_dir(temp.path())
        .arg("export")
        .assert()
        .success();

    let target = temp.child("target.toml");
    target.assert(predicate::str::contains("included_other_val"));
    target.assert(predicate::str::contains("excluded_subgroup_val").not());
    target.assert(predicate::str::contains("unrelated_val").not());
}

#[test]
fn test_key_exclusion_implicit_parent_inclusion() {
    let temp = assert_fs::TempDir::new().unwrap();
    let config = temp.child("settingspec.toml");
    config
        .write_str(
            r#"
[spec.export.file]
"target.toml" = { "group2.subgroup2.k2" = false }

[settings]
group2.subgroup2.k1.default.val = "included_k1"
group2.subgroup2.k2.default.val = "excluded_k2"
"#,
        )
        .unwrap();

    let mut cmd = Command::cargo_bin("settingspec").unwrap();
    cmd.current_dir(temp.path())
        .arg("export")
        .assert()
        .success();

    let target = temp.child("target.toml");
    target.assert(predicate::str::contains("included_k1"));
    target.assert(predicate::str::contains("excluded_k2").not());
}

#[test]
fn test_mixed_inclusion_and_exclusion_in_same_scope() {
    let temp = assert_fs::TempDir::new().unwrap();
    let config = temp.child("settingspec.toml");
    config
        .write_str(
            r#"
[spec.export.file."target.toml"]
"group2.subgroup2.k1" = true
"group2.subgroup2.k2" = false

[settings]
group2.subgroup2.k1.default.val = "included_k1"
group2.subgroup2.k2.default.val = "excluded_k2"
group2.subgroup2.k3.default.val = "unmentioned_k3"
"#,
        )
        .unwrap();

    let mut cmd = Command::cargo_bin("settingspec").unwrap();
    cmd.current_dir(temp.path())
        .arg("export")
        .assert()
        .success();

    let target = temp.child("target.toml");
    target.assert(predicate::str::contains("included_k1"));
    target.assert(predicate::str::contains("excluded_k2").not());
    target.assert(predicate::str::contains("unmentioned_k3").not());
}

#[test]
fn test_tag_inclusion_and_tag_exclusion_override() {
    let temp = assert_fs::TempDir::new().unwrap();
    let config = temp.child("settingspec.toml");
    config
        .write_str(
            r##"
[spec.export.file."target.toml"]
group1 = true
"#sensitive" = false
"#special" = true

[settings]
group1.public.default.val = "public_val"
group1.secret.default.val = "secret_val"
group1.secret.default.tags = ["sensitive"]

group2.tagged.default.val = "tagged_special_val"
group2.tagged.default.tags = ["special"]

group2.untagged.default.val = "untagged_val"
"##,
        )
        .unwrap();

    let mut cmd = Command::cargo_bin("settingspec").unwrap();
    cmd.current_dir(temp.path())
        .arg("export")
        .assert()
        .success();

    let target = temp.child("target.toml");
    target.assert(predicate::str::contains("public_val"));
    target.assert(predicate::str::contains("tagged_special_val"));
    // sensitive is excluded even though parent group1 = true
    target.assert(predicate::str::contains("secret_val").not());
    // group2.untagged is not included
    target.assert(predicate::str::contains("untagged_val").not());
}

#[test]
fn test_pure_exclusion_mode() {
    let temp = assert_fs::TempDir::new().unwrap();
    let config = temp.child("settingspec.toml");
    config
        .write_str(
            r##"
[spec.export.file."target.toml"]
group1 = false
"#draft" = false

[settings]
group1.k1.default.val = "g1_val"
group2.k1.default.val = "g2_val"
group3.draft.default.val = "draft_val"
group3.draft.default.tags = ["draft"]
group3.published.default.val = "pub_val"
"##,
        )
        .unwrap();

    let mut cmd = Command::cargo_bin("settingspec").unwrap();
    cmd.current_dir(temp.path())
        .arg("export")
        .assert()
        .success();

    let target = temp.child("target.toml");
    target.assert(predicate::str::contains("g2_val"));
    target.assert(predicate::str::contains("pub_val"));
    target.assert(predicate::str::contains("g1_val").not());
    target.assert(predicate::str::contains("draft_val").not());
}

#[test]
fn test_default_export_target_omitted_when_export_stdin_specified() {
    let temp = assert_fs::TempDir::new().unwrap();
    let config = temp.child("settingspec.toml");
    config
        .write_str(
            r#"
[spec]
export.stdin = true

[settings]
k1.default.val = "v1"
"#,
        )
        .unwrap();

    let mut cmd = Command::cargo_bin("settingspec").unwrap();
    cmd.current_dir(temp.path())
        .arg("export")
        .assert()
        .success();

    let default_file = temp.child("settings.toml");
    default_file.assert(predicate::path::missing());

    // Also test with format string
    config
        .write_str(
            r#"
[spec.export]
stdin = "json"

[settings]
k1.default.val = "v1"
"#,
        )
        .unwrap();

    let mut cmd2 = Command::cargo_bin("settingspec").unwrap();
    cmd2.current_dir(temp.path())
        .arg("export")
        .assert()
        .success();

    let default_file2 = temp.child("settings.toml");
    default_file2.assert(predicate::path::missing());
}
