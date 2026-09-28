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
active_feature._.default.val = "feature_1"
nullable_feature._.default.null = true
db.host._.default.val = "127.0.0.1"
db.password._.default.null = true
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
key1._.default.val = "val1"
key2._.default.null = true
database.host._.default.val = "localhost"
database.secret._.default.null = true
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
service.name._.default.val = "auth-service"
service.port._.default.val = 3000
service.fallback._.default.null = true
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
top_key._.default.val = "top_val"
top_null._.default.null = true
database.host._.default.val = "localhost"
database.port._.default.val = 5432
database.password._.default.null = true
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
app.name._.default.val = "demo"
app.debug._.default.null = true
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
app.name._.default.val = "demo"
app.port._.default.val = 8080
app.debug._.default.null = true
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
key1._.default.val = "val1"
key2._.default.null = true
db.host._.default.val = "localhost"
db.port._.default.val = 5432
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
api.url._.default.val = "https://api.example.com"
api.timeout._.default.val = 30
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
api.url._.default.val = "https://api.example.com"
api.timeout._.default.val = 30
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
fn test_export_stdout_new_languages() {
    let temp = assert_fs::TempDir::new().unwrap();
    let config = temp.child("settingspec.toml");

    for fmt in [
        "rust",
        "rs",
        "go",
        "golang",
        "zig",
        "c",
        "cpp",
        "c++",
        "java",
        "elm",
        "ruby",
        "rb",
        "scala",
        "haskell",
        "hs",
        "terraform",
        "tf",
    ] {
        config
            .write_str(&format!(
                r#"
[spec.export]
stdout = "{}"

[settings]
key1._.default.val = "val1"
"#,
                fmt
            ))
            .unwrap();

        let mut cmd = Command::cargo_bin("settingspec").unwrap();
        cmd.current_dir(temp.path())
            .arg("export")
            .assert()
            .success();
    }
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
app.name._.default.val = "MyApp"
app.port._.default.val = 8080
secret._.default.null = true
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
app.name._.default.val = "MyApp"
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
app.name._.default.val = "MyApp"
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

#[test]
fn test_export_rust() {
    let temp = assert_fs::TempDir::new().unwrap();
    let config = temp.child("settingspec.toml");
    config
        .write_str(
            r#"
[spec.export.file]
"settings.rs" = true

[settings]
top_key._.default.val = "top_val"
top_null._.default.null = true
database.host._.default.val = "localhost"
database.port._.default.val = 5432
database.password._.default.null = true
"#,
        )
        .unwrap();

    let mut cmd = Command::cargo_bin("settingspec").unwrap();
    cmd.current_dir(temp.path())
        .arg("export")
        .assert()
        .success();

    let out = temp.child("settings.rs");
    out.assert(predicate::str::contains(
        "pub const top_key: &'static str = \"top_val\";",
    ));
    out.assert(predicate::str::contains(
        "pub const top_null: Option<&'static str> = None;",
    ));
    out.assert(predicate::str::contains("pub mod database {"));
    out.assert(predicate::str::contains(
        "pub const host: &'static str = \"localhost\";",
    ));
    out.assert(predicate::str::contains("pub const port: i64 = 5432;"));
    out.assert(predicate::str::contains(
        "pub const password: Option<&'static str> = None;",
    ));
}

#[test]
fn test_export_go() {
    let temp = assert_fs::TempDir::new().unwrap();
    let config = temp.child("settingspec.toml");
    config
        .write_str(
            r#"
[spec.export.file]
"settings.go" = true

[settings]
key1._.default.val = "val1"
key2._.default.null = true
database.host._.default.val = "localhost"
database.port._.default.val = 5432
database.password._.default.null = true
"#,
        )
        .unwrap();

    let mut cmd = Command::cargo_bin("settingspec").unwrap();
    cmd.current_dir(temp.path())
        .arg("export")
        .assert()
        .success();

    let out = temp.child("settings.go");
    out.assert(predicate::str::contains("package settings"));
    out.assert(predicate::str::contains("Key1 = \"val1\""));
    out.assert(predicate::str::contains("Key2 any = nil"));
    out.assert(predicate::str::contains("var Database = struct {"));
    out.assert(predicate::str::contains("Host string"));
    out.assert(predicate::str::contains("Port int"));
    out.assert(predicate::str::contains("Host: \"localhost\""));
    out.assert(predicate::str::contains("Port: 5432"));
    out.assert(predicate::str::contains("Password: nil"));
}

#[test]
fn test_export_zig() {
    let temp = assert_fs::TempDir::new().unwrap();
    let config = temp.child("settingspec.toml");
    config
        .write_str(
            r#"
[spec.export.file]
"settings.zig" = true

[settings]
key1._.default.val = "val1"
key2._.default.null = true
database.host._.default.val = "localhost"
database.port._.default.val = 5432
database.password._.default.null = true
"#,
        )
        .unwrap();

    let mut cmd = Command::cargo_bin("settingspec").unwrap();
    cmd.current_dir(temp.path())
        .arg("export")
        .assert()
        .success();

    let out = temp.child("settings.zig");
    out.assert(predicate::str::contains("pub const key1 = \"val1\";"));
    out.assert(predicate::str::contains("pub const key2 = null;"));
    out.assert(predicate::str::contains("pub const database = struct {"));
    out.assert(predicate::str::contains("pub const host = \"localhost\";"));
    out.assert(predicate::str::contains("pub const port = 5432;"));
    out.assert(predicate::str::contains("pub const password = null;"));
}

#[test]
fn test_export_c() {
    let temp = assert_fs::TempDir::new().unwrap();
    let config = temp.child("settingspec.toml");
    config
        .write_str(
            r#"
[spec.export.file]
"settings.h" = true
"settings.c" = true

[settings]
key1._.default.val = "val1"
key2._.default.null = true
database.host._.default.val = "localhost"
database.port._.default.val = 5432
database.password._.default.null = true
"#,
        )
        .unwrap();

    let mut cmd = Command::cargo_bin("settingspec").unwrap();
    cmd.current_dir(temp.path())
        .arg("export")
        .assert()
        .success();

    for file in ["settings.h", "settings.c"] {
        let out = temp.child(file);
        out.assert(predicate::str::contains("#ifndef SETTINGS_H"));
        out.assert(predicate::str::contains("const char* host;"));
        out.assert(predicate::str::contains("long long port;"));
        out.assert(predicate::str::contains(
            "static const Settings settings = {",
        ));
        out.assert(predicate::str::contains(".host = \"localhost\""));
        out.assert(predicate::str::contains(".port = 5432"));
        out.assert(predicate::str::contains(".password = NULL"));
    }
}

#[test]
fn test_export_cpp() {
    let temp = assert_fs::TempDir::new().unwrap();
    let config = temp.child("settingspec.toml");
    config
        .write_str(
            r#"
[spec.export.file]
"settings.hpp" = true
"settings.cpp" = true

[settings]
key1._.default.val = "val1"
key2._.default.null = true
database.host._.default.val = "localhost"
database.port._.default.val = 5432
database.password._.default.null = true
"#,
        )
        .unwrap();

    let mut cmd = Command::cargo_bin("settingspec").unwrap();
    cmd.current_dir(temp.path())
        .arg("export")
        .assert()
        .success();

    for file in ["settings.hpp", "settings.cpp"] {
        let out = temp.child(file);
        out.assert(predicate::str::contains("namespace settings {"));
        out.assert(predicate::str::contains(
            "inline constexpr const char* key1 = \"val1\";",
        ));
        out.assert(predicate::str::contains(
            "inline constexpr std::nullptr_t key2 = nullptr;",
        ));
        out.assert(predicate::str::contains("namespace database {"));
        out.assert(predicate::str::contains(
            "inline constexpr const char* host = \"localhost\";",
        ));
        out.assert(predicate::str::contains(
            "inline constexpr long long port = 5432;",
        ));
        out.assert(predicate::str::contains(
            "inline constexpr std::nullptr_t password = nullptr;",
        ));
    }
}

#[test]
fn test_export_java() {
    let temp = assert_fs::TempDir::new().unwrap();
    let config = temp.child("settingspec.toml");
    config
        .write_str(
            r#"
[spec.export.file]
"Settings.java" = true

[settings]
key1._.default.val = "val1"
key2._.default.null = true
database.host._.default.val = "localhost"
database.port._.default.val = 5432
database.password._.default.null = true
"#,
        )
        .unwrap();

    let mut cmd = Command::cargo_bin("settingspec").unwrap();
    cmd.current_dir(temp.path())
        .arg("export")
        .assert()
        .success();

    let out = temp.child("Settings.java");
    out.assert(predicate::str::contains("public final class Settings {"));
    out.assert(predicate::str::contains(
        "public static final String key1 = \"val1\";",
    ));
    out.assert(predicate::str::contains(
        "public static final Object key2 = null;",
    ));
    out.assert(predicate::str::contains(
        "public static final class database {",
    ));
    out.assert(predicate::str::contains(
        "public static final String host = \"localhost\";",
    ));
    out.assert(predicate::str::contains(
        "public static final long port = 5432L;",
    ));
    out.assert(predicate::str::contains(
        "public static final Object password = null;",
    ));
}

#[test]
fn test_export_elm() {
    let temp = assert_fs::TempDir::new().unwrap();
    let config = temp.child("settingspec.toml");
    config
        .write_str(
            r#"
[spec.export.file]
"Settings.elm" = true

[settings]
key1._.default.val = "val1"
key2._.default.null = true
database.host._.default.val = "localhost"
database.port._.default.val = 5432
database.password._.default.null = true
"#,
        )
        .unwrap();

    let mut cmd = Command::cargo_bin("settingspec").unwrap();
    cmd.current_dir(temp.path())
        .arg("export")
        .assert()
        .success();

    let out = temp.child("Settings.elm");
    out.assert(predicate::str::contains("module Settings exposing (..)"));
    out.assert(predicate::str::contains("settings ="));
    out.assert(predicate::str::contains("key1 = \"val1\""));
    out.assert(predicate::str::contains("key2 = Nothing"));
    out.assert(predicate::str::contains("host = \"localhost\""));
    out.assert(predicate::str::contains("port_ = 5432"));
    out.assert(predicate::str::contains("password = Nothing"));
}

#[test]
fn test_export_ruby() {
    let temp = assert_fs::TempDir::new().unwrap();
    let config = temp.child("settingspec.toml");
    config
        .write_str(
            r#"
[spec.export.file]
"settings.rb" = true

[settings]
key1._.default.val = "val1"
key2._.default.null = true
database.host._.default.val = "localhost"
database.port._.default.val = 5432
database.password._.default.null = true
"#,
        )
        .unwrap();

    let mut cmd = Command::cargo_bin("settingspec").unwrap();
    cmd.current_dir(temp.path())
        .arg("export")
        .assert()
        .success();

    let out = temp.child("settings.rb");
    out.assert(predicate::str::contains("module Settings"));
    out.assert(predicate::str::contains("KEY1 = \"val1\".freeze"));
    out.assert(predicate::str::contains("KEY2 = nil"));
    out.assert(predicate::str::contains("module Database"));
    out.assert(predicate::str::contains("HOST = \"localhost\".freeze"));
    out.assert(predicate::str::contains("PORT = 5432"));
    out.assert(predicate::str::contains("PASSWORD = nil"));
}

#[test]
fn test_export_scala() {
    let temp = assert_fs::TempDir::new().unwrap();
    let config = temp.child("settingspec.toml");
    config
        .write_str(
            r#"
[spec.export.file]
"settings.scala" = true

[settings]
key1._.default.val = "val1"
key2._.default.null = true
database.host._.default.val = "localhost"
database.port._.default.val = 5432
database.password._.default.null = true
"#,
        )
        .unwrap();

    let mut cmd = Command::cargo_bin("settingspec").unwrap();
    cmd.current_dir(temp.path())
        .arg("export")
        .assert()
        .success();

    let out = temp.child("settings.scala");
    out.assert(predicate::str::contains("object Settings {"));
    out.assert(predicate::str::contains(
        "final val key1: String = \"val1\"",
    ));
    out.assert(predicate::str::contains(
        "final val key2: Option[String] = None",
    ));
    out.assert(predicate::str::contains("object database {"));
    out.assert(predicate::str::contains(
        "final val host: String = \"localhost\"",
    ));
    out.assert(predicate::str::contains("final val port: Long = 5432L"));
    out.assert(predicate::str::contains(
        "final val password: Option[String] = None",
    ));
}

#[test]
fn test_export_haskell() {
    let temp = assert_fs::TempDir::new().unwrap();
    let config = temp.child("settingspec.toml");
    config
        .write_str(
            r#"
[spec.export.file]
"settings.hs" = true

[settings]
key1._.default.val = "val1"
key2._.default.null = true
database.host._.default.val = "localhost"
database.port._.default.val = 5432
database.password._.default.null = true
"#,
        )
        .unwrap();

    let mut cmd = Command::cargo_bin("settingspec").unwrap();
    cmd.current_dir(temp.path())
        .arg("export")
        .assert()
        .success();

    let out = temp.child("settings.hs");
    out.assert(predicate::str::contains("module Settings where"));
    out.assert(predicate::str::contains("settings :: Settings"));
    out.assert(predicate::str::contains("key1 = \"val1\""));
    out.assert(predicate::str::contains("key2 = Nothing"));
    out.assert(predicate::str::contains("host = \"localhost\""));
    out.assert(predicate::str::contains("port = 5432"));
    out.assert(predicate::str::contains("password = Nothing"));
}

#[test]
fn test_export_terraform() {
    let temp = assert_fs::TempDir::new().unwrap();
    let config = temp.child("settingspec.toml");
    config
        .write_str(
            r#"
[spec.export.file]
"settings.tf" = true

[settings]
key1._.default.val = "val1"
key2._.default.null = true
database.host._.default.val = "localhost"
database.port._.default.val = 5432
database.password._.default.null = true
"#,
        )
        .unwrap();

    let mut cmd = Command::cargo_bin("settingspec").unwrap();
    cmd.current_dir(temp.path())
        .arg("export")
        .assert()
        .success();

    let out = temp.child("settings.tf");
    out.assert(predicate::str::contains("locals {"));
    out.assert(predicate::str::contains("key1 = \"val1\""));
    out.assert(predicate::str::contains("key2 = null"));
    out.assert(predicate::str::contains("database = {"));
    out.assert(predicate::str::contains("host = \"localhost\""));
    out.assert(predicate::str::contains("port = 5432"));
    out.assert(predicate::str::contains("password = null"));
}
