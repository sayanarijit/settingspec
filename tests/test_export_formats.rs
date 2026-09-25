use assert_cmd::Command;
use assert_fs::prelude::*;
use predicates::prelude::*;

#[test]
fn test_export_toml_omits_nulls() {
    let temp = assert_fs::TempDir::new().unwrap();
    let config = temp.child("settingspec.toml");
    config
        .write_str(
            r#"
[spec.export.file]
"out.toml" = true

[settings]
active_feature.default.val = "feature_1"
nullable_feature.default.null = true
db.host.default.val = "127.0.0.1"
db.password.default.null = true
"#,
        )
        .unwrap();

    let mut cmd = Command::cargo_bin("settingspec").unwrap();
    cmd.current_dir(temp.path())
        .arg("export")
        .assert()
        .success();

    let out = temp.child("out.toml");
    out.assert(predicate::str::contains("active_feature = \"feature_1\""));
    out.assert(predicate::str::contains("host = \"127.0.0.1\""));
    out.assert(predicate::str::contains("nullable_feature").not());
    out.assert(predicate::str::contains("password").not());
}

#[test]
fn test_export_json_serializes_nulls() {
    let temp = assert_fs::TempDir::new().unwrap();
    let config = temp.child("settingspec.toml");
    config
        .write_str(
            r#"
[spec.export.file]
"out.json" = true

[settings]
key1.default.val = "val1"
key2.default.null = true
database.host.default.val = "localhost"
database.secret.default.null = true
"#,
        )
        .unwrap();

    let mut cmd = Command::cargo_bin("settingspec").unwrap();
    cmd.current_dir(temp.path())
        .arg("export")
        .assert()
        .success();

    let out = temp.child("out.json");
    out.assert(predicate::str::contains("\"key1\": \"val1\""));
    out.assert(predicate::str::contains("\"key2\": null"));
    out.assert(predicate::str::contains("\"host\": \"localhost\""));
    out.assert(predicate::str::contains("\"secret\": null"));
}

#[test]
fn test_export_yaml_and_yml() {
    let temp = assert_fs::TempDir::new().unwrap();
    let config = temp.child("settingspec.toml");
    config
        .write_str(
            r#"
[spec.export.file]
"out.yaml" = true
"out.yml" = true

[settings]
service.name.default.val = "auth-service"
service.port.default.val = 3000
service.fallback.default.null = true
"#,
        )
        .unwrap();

    let mut cmd = Command::cargo_bin("settingspec").unwrap();
    cmd.current_dir(temp.path())
        .arg("export")
        .assert()
        .success();

    for ext in ["out.yaml", "out.yml"] {
        let out = temp.child(ext);
        out.assert(predicate::str::contains("name: auth-service"));
        out.assert(predicate::str::contains("port: 3000"));
        out.assert(predicate::str::contains("fallback: null"));
    }
}

#[test]
fn test_export_python_nested_classes_and_none() {
    let temp = assert_fs::TempDir::new().unwrap();
    let config = temp.child("settingspec.toml");
    config
        .write_str(
            r#"
[spec.export.file]
"settings.py" = true

[settings]
top_key.default.val = "top_val"
top_null.default.null = true
database.host.default.val = "localhost"
database.port.default.val = 5432
database.password.default.null = true
"#,
        )
        .unwrap();

    let mut cmd = Command::cargo_bin("settingspec").unwrap();
    cmd.current_dir(temp.path())
        .arg("export")
        .assert()
        .success();

    let out = temp.child("settings.py");
    out.assert(predicate::str::contains("top_key = \"top_val\""));
    out.assert(predicate::str::contains("top_null = None"));
    out.assert(predicate::str::contains("class database:"));
    out.assert(predicate::str::contains("host = \"localhost\""));
    out.assert(predicate::str::contains("port = 5432"));
    out.assert(predicate::str::contains("password = None"));
}

#[test]
fn test_export_javascript_and_mjs() {
    let temp = assert_fs::TempDir::new().unwrap();
    let config = temp.child("settingspec.toml");
    config
        .write_str(
            r#"
[spec.export.file]
"settings.js" = true
"settings.mjs" = true

[settings]
app.name.default.val = "demo"
app.debug.default.null = true
"#,
        )
        .unwrap();

    let mut cmd = Command::cargo_bin("settingspec").unwrap();
    cmd.current_dir(temp.path())
        .arg("export")
        .assert()
        .success();

    for filename in ["settings.js", "settings.mjs"] {
        let out = temp.child(filename);
        out.assert(predicate::str::contains("export default {"));
        out.assert(predicate::str::contains("\"name\": \"demo\""));
        out.assert(predicate::str::contains("\"debug\": null"));
    }
}

#[test]
fn test_export_typescript_interfaces_and_default_export() {
    let temp = assert_fs::TempDir::new().unwrap();
    let config = temp.child("settingspec.toml");
    config
        .write_str(
            r#"
[spec.export.file]
"settings.ts" = true

[settings]
app.name.default.val = "demo"
app.port.default.val = 8080
app.debug.default.null = true
"#,
        )
        .unwrap();

    let mut cmd = Command::cargo_bin("settingspec").unwrap();
    cmd.current_dir(temp.path())
        .arg("export")
        .assert()
        .success();

    let out = temp.child("settings.ts");
    out.assert(predicate::str::contains("export interface Settings"));
    out.assert(predicate::str::contains("name: string;"));
    out.assert(predicate::str::contains("port: number;"));
    out.assert(predicate::str::contains("debug: null;"));
    out.assert(predicate::str::contains("export default settings;"));
}

#[test]
fn test_export_lua_table_and_nil() {
    let temp = assert_fs::TempDir::new().unwrap();
    let config = temp.child("settingspec.toml");
    config
        .write_str(
            r#"
[spec.export.file]
"settings.lua" = true

[settings]
key1.default.val = "val1"
key2.default.null = true
db.host.default.val = "localhost"
db.port.default.val = 5432
"#,
        )
        .unwrap();

    let mut cmd = Command::cargo_bin("settingspec").unwrap();
    cmd.current_dir(temp.path())
        .arg("export")
        .assert()
        .success();

    let out = temp.child("settings.lua");
    out.assert(predicate::str::contains("return {"));
    out.assert(predicate::str::contains("key1 = \"val1\""));
    out.assert(predicate::str::contains("key2 = nil"));
    out.assert(predicate::str::contains("host = \"localhost\""));
    out.assert(predicate::str::contains("port = 5432"));
}

#[test]
fn test_export_stdout_toml_json_yaml_env() {
    let temp = assert_fs::TempDir::new().unwrap();
    let config = temp.child("settingspec.toml");
    config
        .write_str(
            r#"
[spec.export]
stdout = "json"

[settings]
api.url.default.val = "https://api.example.com"
api.timeout.default.val = 30
"#,
        )
        .unwrap();

    let mut cmd = Command::cargo_bin("settingspec").unwrap();
    cmd.current_dir(temp.path())
        .arg("export")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "\"url\": \"https://api.example.com\"",
        ))
        .stdout(predicate::str::contains("\"timeout\": 30"));

    // Now test stdout = "env"
    config
        .write_str(
            r#"
[spec.export]
stdout = "env"

[settings]
api.url.default.val = "https://api.example.com"
api.timeout.default.val = 30
"#,
        )
        .unwrap();

    let mut cmd2 = Command::cargo_bin("settingspec").unwrap();
    cmd2.current_dir(temp.path())
        .arg("export")
        .assert()
        .success()
        .stdout(predicate::str::contains("API_URL=https://api.example.com"))
        .stdout(predicate::str::contains("API_TIMEOUT=30"));
}

#[test]
fn test_export_file_env_format() {
    let temp = assert_fs::TempDir::new().unwrap();
    let config = temp.child("settingspec.toml");
    config
        .write_str(
            r#"
[spec.export.file]
".env" = true
".env.local" = true
"settings.env" = true
"env.prod" = true

[settings]
app.name.default.val = "MyApp"
app.port.default.val = 8080
secret.default.null = true
"#,
        )
        .unwrap();

    let mut cmd = Command::cargo_bin("settingspec").unwrap();
    cmd.current_dir(temp.path())
        .arg("export")
        .assert()
        .success();

    let dotenv = temp.child(".env");
    dotenv.assert(predicate::path::exists());
    dotenv.assert(predicate::str::contains("APP_NAME=MyApp"));
    dotenv.assert(predicate::str::contains("APP_PORT=8080"));
    dotenv.assert(predicate::str::contains("SECRET").not());

    let dotenv_local = temp.child(".env.local");
    dotenv_local.assert(predicate::path::exists());
    dotenv_local.assert(predicate::str::contains("APP_NAME=MyApp"));
    dotenv_local.assert(predicate::str::contains("APP_PORT=8080"));
    dotenv_local.assert(predicate::str::contains("SECRET").not());

    let env_file = temp.child("settings.env");
    env_file.assert(predicate::path::exists());
    env_file.assert(predicate::str::contains("APP_NAME=MyApp"));
    env_file.assert(predicate::str::contains("APP_PORT=8080"));
    env_file.assert(predicate::str::contains("SECRET").not());

    let env_prod_file = temp.child("env.prod");
    env_prod_file.assert(predicate::path::exists());
    env_prod_file.assert(predicate::str::contains("APP_NAME=MyApp"));
    env_prod_file.assert(predicate::str::contains("APP_PORT=8080"));
    env_prod_file.assert(predicate::str::contains("SECRET").not());
}

#[test]
fn test_export_file_unrecognizable_format_fails() {
    let temp = assert_fs::TempDir::new().unwrap();
    let config = temp.child("settingspec.toml");
    config
        .write_str(
            r#"
[spec.export.file]
"aaa" = true

[settings]
app.name.default.val = "MyApp"
"#,
        )
        .unwrap();

    let mut cmd = Command::cargo_bin("settingspec").unwrap();
    cmd.current_dir(temp.path())
        .arg("export")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Unsupported export format: 'aaa'"));

    // Test unknown extension
    config
        .write_str(
            r#"
[spec.export.file]
"settings.unknown" = true

[settings]
app.name.default.val = "MyApp"
"#,
        )
        .unwrap();

    let mut cmd2 = Command::cargo_bin("settingspec").unwrap();
    cmd2.current_dir(temp.path())
        .arg("export")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "Unsupported export format: '.unknown'",
        ));
}
