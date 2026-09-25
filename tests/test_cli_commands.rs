use assert_cmd::Command;
use assert_fs::prelude::*;
use predicates::prelude::*;

#[test]
fn test_cli_help_and_version() {
    let mut cmd_help = Command::cargo_bin("settingspec").unwrap();
    cmd_help
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Usage:"))
        .stdout(predicate::str::contains("export"))
        .stdout(predicate::str::contains("check"))
        .stdout(predicate::str::contains("run"));

    let mut cmd_ver = Command::cargo_bin("settingspec").unwrap();
    cmd_ver
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("settingspec"));
}

#[test]
fn test_cli_check_command_does_not_create_files() {
    let temp = assert_fs::TempDir::new().unwrap();
    let config = temp.child("settingspec.toml");
    config
        .write_str(
            r#"
[spec.export.file]
"settings.toml" = true
"settings.json" = true

[settings]
key1.default.val = "val1"
"#,
        )
        .unwrap();

    let mut cmd = Command::cargo_bin("settingspec").unwrap();
    cmd.current_dir(temp.path()).arg("check").assert().success();

    // Verify check does NOT create any files
    temp.child("settings.toml")
        .assert(predicate::path::missing());
    temp.child("settings.json")
        .assert(predicate::path::missing());
}

#[test]
fn test_cli_run_executes_child_and_cleans_up_by_default() {
    let temp = assert_fs::TempDir::new().unwrap();
    let config = temp.child("settingspec.toml");
    config
        .write_str(
            r#"
[spec]
export.keep = false
export.file."settings.toml" = true

[settings]
greeting.default.val = "hello_from_run"
"#,
        )
        .unwrap();

    // The child command should see settings.toml during execution
    let mut cmd = Command::cargo_bin("settingspec").unwrap();
    cmd.current_dir(temp.path())
        .args([
            "run",
            "--",
            "sh",
            "-c",
            "test -f settings.toml && grep 'greeting = \"hello_from_run\"' settings.toml",
        ])
        .assert()
        .success();

    // After run completes, settings.toml MUST be cleaned up (deleted)
    temp.child("settings.toml")
        .assert(predicate::path::missing());
}

#[test]
fn test_cli_run_retains_files_when_keep_true() {
    let temp = assert_fs::TempDir::new().unwrap();
    let config = temp.child("settingspec.toml");
    config
        .write_str(
            r#"
[spec]
export.keep = true
export.file."settings.toml" = true

[settings]
greeting.default.val = "retained_value"
"#,
        )
        .unwrap();

    let mut cmd = Command::cargo_bin("settingspec").unwrap();
    cmd.current_dir(temp.path())
        .args(["run", "--", "echo", "done"])
        .assert()
        .success();

    // With keep = true, file must still exist
    let exported = temp.child("settings.toml");
    exported.assert(predicate::path::exists());
    exported.assert(predicate::str::contains("retained_value"));
}

#[test]
fn test_cli_run_propagates_child_exit_code() {
    let temp = assert_fs::TempDir::new().unwrap();
    let config = temp.child("settingspec.toml");
    config
        .write_str(
            r#"
[settings]
key1.default.val = "val1"
"#,
        )
        .unwrap();

    // Command exiting with 42
    let mut cmd = Command::cargo_bin("settingspec").unwrap();
    cmd.current_dir(temp.path())
        .args(["run", "--", "sh", "-c", "exit 42"])
        .assert()
        .code(42);

    // Command exiting with 1
    let mut cmd2 = Command::cargo_bin("settingspec").unwrap();
    cmd2.current_dir(temp.path())
        .args(["run", "--", "sh", "-c", "exit 1"])
        .assert()
        .code(1);
}

#[test]
fn test_cli_run_export_env_true() {
    let temp = assert_fs::TempDir::new().unwrap();
    let config = temp.child("settingspec.toml");
    config
        .write_str(
            r#"
[spec.export]
env = true

[settings]
server.host.default.val = "127.0.0.1"
server.port.default.val = 8080
secret.default.null = true
"#,
        )
        .unwrap();

    let mut cmd = Command::cargo_bin("settingspec").unwrap();
    cmd.current_dir(temp.path())
        .args([
            "run",
            "--",
            "sh",
            "-c",
            "test ! -e settings.toml && test \"$SERVER_HOST\" = \"127.0.0.1\" && test \"$SERVER_PORT\" = \"8080\" && test -z \"$SECRET\"",
        ])
        .assert()
        .success();
}

#[test]
fn test_cli_run_export_env_empty_string_same_as_true() {
    let temp = assert_fs::TempDir::new().unwrap();
    let config = temp.child("settingspec.toml");
    config
        .write_str(
            r#"
[spec]
export.env = ""

[settings]
greeting.default.val = "hello world"
"#,
        )
        .unwrap();

    let mut cmd = Command::cargo_bin("settingspec").unwrap();
    cmd.current_dir(temp.path())
        .args([
            "run",
            "--",
            "sh",
            "-c",
            "test \"$GREETING\" = \"hello world\"",
        ])
        .assert()
        .success();
}

#[test]
fn test_cli_run_export_env_with_prefix() {
    let temp = assert_fs::TempDir::new().unwrap();
    let config = temp.child("settingspec.toml");
    config
        .write_str(
            r#"
[spec]
export.env = "MYAPP_"

[settings]
db.host.default.val = "localhost"
db.port.default.val = 5432
"#,
        )
        .unwrap();

    let mut cmd = Command::cargo_bin("settingspec").unwrap();
    cmd.current_dir(temp.path())
        .args([
            "run",
            "--",
            "sh",
            "-c",
            "test \"$MYAPP_DB_HOST\" = \"localhost\" && test \"$MYAPP_DB_PORT\" = \"5432\" && test -z \"$DB_HOST\"",
        ])
        .assert()
        .success();
}

#[test]
fn test_cli_run_export_env_false_does_not_export() {
    let temp = assert_fs::TempDir::new().unwrap();
    let config = temp.child("settingspec.toml");
    config
        .write_str(
            r#"
[spec]
export.env = false

[settings]
my_secret.default.val = "do_not_export"
"#,
        )
        .unwrap();

    let mut cmd = Command::cargo_bin("settingspec").unwrap();
    cmd.current_dir(temp.path())
        .args(["run", "--", "sh", "-c", "test -z \"$MY_SECRET\""])
        .assert()
        .success();
}

#[test]
fn test_cli_run_export_env_various_types() {
    let temp = assert_fs::TempDir::new().unwrap();
    let config = temp.child("settingspec.toml");
    config
        .write_str(
            r#"
[spec]
export.env = true

[settings]
flag_bool.default.val = true
items.default.val = [1, 2, 3]
rate.default.val = 3.14
"#,
        )
        .unwrap();

    let mut cmd = Command::cargo_bin("settingspec").unwrap();
    cmd.current_dir(temp.path())
        .args([
            "run",
            "--",
            "sh",
            "-c",
            "test \"$FLAG_BOOL\" = \"true\" && test \"$ITEMS\" = \"[1,2,3]\" && test \"$RATE\" = \"3.14\"",
        ])
        .assert()
        .success();
}

#[test]
fn test_cli_run_export_stdin_true_default_toml() {
    let temp = assert_fs::TempDir::new().unwrap();
    let config = temp.child("settingspec.toml");
    config
        .write_str(
            r#"
[spec]
export.stdin = true

[settings]
server.host.default.val = "127.0.0.1"
server.port.default.val = 8080
"#,
        )
        .unwrap();

    let mut cmd = Command::cargo_bin("settingspec").unwrap();
    cmd.current_dir(temp.path())
        .args([
            "run",
            "--",
            "python3",
            "-c",
            "import sys, tomllib; data = tomllib.loads(sys.stdin.read()); assert data['server']['host'] == '127.0.0.1'; assert data['server']['port'] == 8080",
        ])
        .assert()
        .success();

    // Verify default settings.toml was NOT created
    temp.child("settings.toml")
        .assert(predicate::path::missing());
}

#[test]
fn test_cli_run_export_stdin_empty_string_same_as_true() {
    let temp = assert_fs::TempDir::new().unwrap();
    let config = temp.child("settingspec.toml");
    config
        .write_str(
            r#"
[spec]
export.stdin = ""

[settings]
greeting.default.val = "hello from stdin"
"#,
        )
        .unwrap();

    let mut cmd = Command::cargo_bin("settingspec").unwrap();
    cmd.current_dir(temp.path())
        .args([
            "run",
            "--",
            "python3",
            "-c",
            "import sys, tomllib; data = tomllib.loads(sys.stdin.read()); assert data['greeting'] == 'hello from stdin'",
        ])
        .assert()
        .success();
}

#[test]
fn test_cli_run_export_stdin_json() {
    let temp = assert_fs::TempDir::new().unwrap();
    let config = temp.child("settingspec.toml");
    config
        .write_str(
            r#"
[spec.export]
stdin = "json"

[settings]
api.endpoint.default.val = "https://example.com"
api.retries.default.val = 5
flag.default.val = true
"#,
        )
        .unwrap();

    let mut cmd = Command::cargo_bin("settingspec").unwrap();
    cmd.current_dir(temp.path())
        .args([
            "run",
            "--",
            "python3",
            "-c",
            "import sys, json; data = json.load(sys.stdin); assert data['api']['endpoint'] == 'https://example.com'; assert data['api']['retries'] == 5; assert data['flag'] is True",
        ])
        .assert()
        .success();
}

#[test]
fn test_cli_run_export_stdin_yaml() {
    let temp = assert_fs::TempDir::new().unwrap();
    let config = temp.child("settingspec.toml");
    config
        .write_str(
            r#"
[spec.export]
stdin = "yaml"

[settings]
app.name.default.val = "my_app"
"#,
        )
        .unwrap();

    let mut cmd = Command::cargo_bin("settingspec").unwrap();
    cmd.current_dir(temp.path())
        .args(["run", "--", "sh", "-c", "grep -q 'name: my_app'"])
        .assert()
        .success();
}

#[test]
fn test_cli_run_export_stdin_env() {
    let temp = assert_fs::TempDir::new().unwrap();
    let config = temp.child("settingspec.toml");
    config
        .write_str(
            r#"
[spec.export]
stdin = "env"

[settings]
db.host.default.val = "db.internal"
db.port.default.val = 3306
"#,
        )
        .unwrap();

    let mut cmd = Command::cargo_bin("settingspec").unwrap();
    cmd.current_dir(temp.path())
        .args([
            "run",
            "--",
            "sh",
            "-c",
            "OUT=$(cat); echo \"$OUT\" | grep -q 'DB_HOST=db.internal' && echo \"$OUT\" | grep -q 'DB_PORT=3306'",
        ])
        .assert()
        .success();
}

#[test]
fn test_cli_run_export_stdin_false_does_not_pass_settings() {
    let temp = assert_fs::TempDir::new().unwrap();
    let config = temp.child("settingspec.toml");
    config
        .write_str(
            r#"
[spec]
export.stdin = false

[settings]
secret.default.val = "should_not_be_on_stdin"
"#,
        )
        .unwrap();

    let mut cmd = Command::cargo_bin("settingspec").unwrap();
    cmd.current_dir(temp.path())
        .args(["run", "--", "sh", "-c", "test -z \"$(cat)\""])
        .assert()
        .success();
}

#[test]
fn test_cli_run_export_stdin_combined_with_files_and_env() {
    let temp = assert_fs::TempDir::new().unwrap();
    let config = temp.child("settingspec.toml");
    config
        .write_str(
            r#"
[spec]
export.stdin = "json"
export.env = "MYAPP_"
export.file."config.toml" = true

[settings]
service.name.default.val = "auth-service"
service.port.default.val = 9000
"#,
        )
        .unwrap();

    let mut cmd = Command::cargo_bin("settingspec").unwrap();
    cmd.current_dir(temp.path())
        .args([
            "run",
            "--",
            "sh",
            "-c",
            "test -f config.toml && test \"$MYAPP_SERVICE_NAME\" = \"auth-service\" && python3 -c \"import sys, json; d = json.load(sys.stdin); assert d['service']['port'] == 9000\"",
        ])
        .assert()
        .success();

    // After run completes, config.toml must be cleaned up
    temp.child("config.toml").assert(predicate::path::missing());
}

#[test]
fn test_cli_run_export_stdin_unsupported_format_fails() {
    let temp = assert_fs::TempDir::new().unwrap();
    let config = temp.child("settingspec.toml");
    config
        .write_str(
            r#"
[spec.export]
stdin = "unsupported_fmt"

[settings]
key.default.val = "val"
"#,
        )
        .unwrap();

    let mut cmd = Command::cargo_bin("settingspec").unwrap();
    cmd.current_dir(temp.path())
        .args(["run", "--", "echo", "hi"])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "Unsupported export format: 'unsupported_fmt'",
        ));
}

#[test]
fn test_cli_export_files_rejected() {
    let temp = assert_fs::TempDir::new().unwrap();
    let config = temp.child("settingspec.toml");
    config
        .write_str(
            r#"
[spec.export.files]
"out.json" = true

[settings]
msg.default.val = "should_fail"
"#,
        )
        .unwrap();

    let mut cmd = Command::cargo_bin("settingspec").unwrap();
    cmd.current_dir(temp.path())
        .arg("check")
        .assert()
        .failure()
        .stderr(predicate::str::contains("unknown field `files`"));
}

#[test]
fn test_cli_export_stdout_true_defaults_to_toml() {
    let temp = assert_fs::TempDir::new().unwrap();
    let config = temp.child("settingspec.toml");
    config
        .write_str(
            r#"
[spec.export]
stdout = true

[settings]
server.host.default.val = "127.0.0.1"
server.port.default.val = 8080
"#,
        )
        .unwrap();

    let mut cmd = Command::cargo_bin("settingspec").unwrap();
    cmd.current_dir(temp.path())
        .arg("export")
        .assert()
        .success()
        .stdout(predicate::str::contains("host = \"127.0.0.1\""))
        .stdout(predicate::str::contains("port = 8080"));
}

#[test]
fn test_cli_export_file_true_defaults_to_settings_toml() {
    let temp = assert_fs::TempDir::new().unwrap();
    let config = temp.child("settingspec.toml");
    config
        .write_str(
            r#"
[spec.export]
file = true

[settings]
key1.default.val = "file_val"
"#,
        )
        .unwrap();

    let mut cmd = Command::cargo_bin("settingspec").unwrap();
    cmd.current_dir(temp.path())
        .arg("export")
        .assert()
        .success();

    let exported = temp.child("settings.toml");
    exported.assert(predicate::path::exists());
    exported.assert(predicate::str::contains("key1 = \"file_val\""));
}

#[test]
fn test_cli_export_file_true_combined_with_stdout_true() {
    let temp = assert_fs::TempDir::new().unwrap();
    let config = temp.child("settingspec.toml");
    config
        .write_str(
            r#"
[spec.export]
stdout = true
file = true

[settings]
app.name.default.val = "both_outputs"
"#,
        )
        .unwrap();

    let mut cmd = Command::cargo_bin("settingspec").unwrap();
    cmd.current_dir(temp.path())
        .arg("export")
        .assert()
        .success()
        .stdout(predicate::str::contains("name = \"both_outputs\""));

    let exported = temp.child("settings.toml");
    exported.assert(predicate::path::exists());
    exported.assert(predicate::str::contains("name = \"both_outputs\""));
}

#[test]
fn test_gitignore_auto_append_on_export_in_git_repo() {
    let temp = assert_fs::TempDir::new().unwrap();
    temp.child(".git").create_dir_all().unwrap();

    let config = temp.child("settingspec.toml");
    config
        .write_str(
            r#"
[spec.export]
file = true

[settings]
key1.default.val = "val1"
"#,
        )
        .unwrap();

    let mut cmd = Command::cargo_bin("settingspec").unwrap();
    cmd.current_dir(temp.path())
        .arg("export")
        .assert()
        .success();

    let gitignore = temp.child(".gitignore");
    gitignore.assert(predicate::path::exists());
    gitignore.assert(predicate::str::contains("settings.toml\n"));
}

#[test]
fn test_gitignore_auto_append_on_run_command_in_git_repo() {
    let temp = assert_fs::TempDir::new().unwrap();
    temp.child(".git").create_dir_all().unwrap();

    let config = temp.child("settingspec.toml");
    config
        .write_str(
            r#"
[spec.export]
file = true
keep = false

[settings]
key1.default.val = "val1"
"#,
        )
        .unwrap();

    let mut cmd = Command::cargo_bin("settingspec").unwrap();
    cmd.current_dir(temp.path())
        .args(["run", "--", "echo", "hello"])
        .assert()
        .success();

    // Exported file cleaned up
    temp.child("settings.toml")
        .assert(predicate::path::missing());
    // .gitignore still created and contains settings.toml
    let gitignore = temp.child(".gitignore");
    gitignore.assert(predicate::path::exists());
    gitignore.assert(predicate::str::contains("settings.toml\n"));
}

#[test]
fn test_gitignore_auto_append_disabled_by_skip_gitignore() {
    let temp = assert_fs::TempDir::new().unwrap();
    temp.child(".git").create_dir_all().unwrap();

    let config = temp.child("settingspec.toml");
    config
        .write_str(
            r#"
[spec]
export.skip_gitignore = true
export.file = true

[settings]
key1.default.val = "val1"
"#,
        )
        .unwrap();

    let mut cmd = Command::cargo_bin("settingspec").unwrap();
    cmd.current_dir(temp.path())
        .arg("export")
        .assert()
        .success();

    let gitignore = temp.child(".gitignore");
    gitignore.assert(predicate::path::missing());
}

#[test]
fn test_gitignore_no_duplicate_when_entry_exists() {
    let temp = assert_fs::TempDir::new().unwrap();
    temp.child(".git").create_dir_all().unwrap();

    let gitignore = temp.child(".gitignore");
    gitignore.write_str("target/\nsettings.toml\n").unwrap();

    let config = temp.child("settingspec.toml");
    config
        .write_str(
            r#"
[spec.export]
file = true

[settings]
key1.default.val = "val1"
"#,
        )
        .unwrap();

    let mut cmd = Command::cargo_bin("settingspec").unwrap();
    cmd.current_dir(temp.path())
        .arg("export")
        .assert()
        .success();

    let content = std::fs::read_to_string(gitignore.path()).unwrap();
    assert_eq!(content.matches("settings.toml").count(), 1);
}

#[test]
fn test_gitignore_preserves_content_without_newline() {
    let temp = assert_fs::TempDir::new().unwrap();
    temp.child(".git").create_dir_all().unwrap();

    let gitignore = temp.child(".gitignore");
    gitignore.write_str("target/").unwrap();

    let config = temp.child("settingspec.toml");
    config
        .write_str(
            r#"
[spec.export]
file = true

[settings]
key1.default.val = "val1"
"#,
        )
        .unwrap();

    let mut cmd = Command::cargo_bin("settingspec").unwrap();
    cmd.current_dir(temp.path())
        .arg("export")
        .assert()
        .success();

    let content = std::fs::read_to_string(gitignore.path()).unwrap();
    assert_eq!(content, "target/\n/settings.toml\n");
}

#[test]
fn test_gitignore_auto_append_subdirectory_to_git_root() {
    let temp = assert_fs::TempDir::new().unwrap();
    temp.child(".git").create_dir_all().unwrap();

    let sub = temp.child("subapp");
    sub.create_dir_all().unwrap();

    let config = sub.child("settingspec.toml");
    config
        .write_str(
            r#"
[spec.export.file]
"output.json" = true

[settings]
key1.default.val = "val1"
"#,
        )
        .unwrap();

    let mut cmd = Command::cargo_bin("settingspec").unwrap();
    cmd.current_dir(sub.path()).arg("export").assert().success();

    let root_gitignore = temp.child(".gitignore");
    root_gitignore.assert(predicate::path::exists());
    root_gitignore.assert(predicate::str::contains("subapp/output.json\n"));

    let sub_gitignore = sub.child(".gitignore");
    sub_gitignore.assert(predicate::path::missing());
}

#[test]
fn test_check_does_not_create_gitignore() {
    let temp = assert_fs::TempDir::new().unwrap();
    temp.child(".git").create_dir_all().unwrap();

    let config = temp.child("settingspec.toml");
    config
        .write_str(
            r#"
[settings]
key1.default.val = "val1"
"#,
        )
        .unwrap();

    let mut cmd = Command::cargo_bin("settingspec").unwrap();
    cmd.current_dir(temp.path()).arg("check").assert().success();

    temp.child(".gitignore").assert(predicate::path::missing());
}
