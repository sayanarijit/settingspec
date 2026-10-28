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
    out.assert(predicate::str::contains("top_key: str = \"top_val\""));
    out.assert(predicate::str::contains("top_null: str | None = None"));
    out.assert(predicate::str::contains("class database:"));
    out.assert(predicate::str::contains("host: str = \"localhost\""));
    out.assert(predicate::str::contains("port: int = 5432"));
    out.assert(predicate::str::contains("password: str | None = None"));
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
    out.assert(predicate::str::contains("debug: string | null;"));
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
    out.assert(predicate::str::contains("Key2 *string = nil"));
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
    out.assert(predicate::str::contains(
        "pub const key2: ?[]const u8 = null;",
    ));
    out.assert(predicate::str::contains("pub const database = struct {"));
    out.assert(predicate::str::contains("pub const host = \"localhost\";"));
    out.assert(predicate::str::contains("pub const port = 5432;"));
    out.assert(predicate::str::contains(
        "pub const password: ?[]const u8 = null;",
    ));
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
            "inline const std::optional<std::string> key2 = std::nullopt;",
        ));
        out.assert(predicate::str::contains("namespace database {"));
        out.assert(predicate::str::contains(
            "inline constexpr const char* host = \"localhost\";",
        ));
        out.assert(predicate::str::contains(
            "inline constexpr long long port = 5432;",
        ));
        out.assert(predicate::str::contains(
            "inline const std::optional<std::string> password = std::nullopt;",
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
        "public static final java.util.Optional<String> key2 = java.util.Optional.empty();",
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
        "public static final java.util.Optional<String> password = java.util.Optional.empty();",
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

fn export_unresolved(files: &str, settings: &str, env: &[(&str, &str)]) -> assert_fs::TempDir {
    let temp = assert_fs::TempDir::new().unwrap();
    temp.child("settingspec.toml")
        .write_str(&format!(
            "[spec]\nexport.file = {{ {} }}\n\n[settings]\n{}",
            files, settings
        ))
        .unwrap();
    let mut cmd = Command::cargo_bin("settingspec").unwrap();
    cmd.current_dir(temp.path()).arg("export");
    for (k, v) in env {
        cmd.env(k, v);
    }
    cmd.assert().success();
    temp
}

#[test]
fn test_resolve_false_static_formats_contain_env_name_only() {
    let temp = export_unresolved(
        r#""s.json" = true, "s.toml" = true"#,
        "pw._.default.env = \"DB_PASSWORD\"\nhost._.default.val = \"localhost\"\n",
        &[("DB_PASSWORD", "hunter2")],
    );
    temp.child("s.json")
        .assert(predicate::str::contains("\"pw\": \"DB_PASSWORD\""))
        .assert(predicate::str::contains("\"host\": \"localhost\""))
        .assert(predicate::str::contains("hunter2").not());
    temp.child("s.toml")
        .assert(predicate::str::contains("pw = \"DB_PASSWORD\""))
        .assert(predicate::str::contains("hunter2").not());
}

#[test]
fn test_resolve_false_languages_read_env_at_runtime() {
    let temp = export_unresolved(
        r#""s.py" = true, "s.js" = true, "s.rb" = true, "s.lua" = true"#,
        "pw._.default.env = \"DB_PASSWORD\"\nport._.default.env = \"PORT\"\nport._.default.val = 5432\n",
        &[("DB_PASSWORD", "hunter2")],
    );
    temp.child("s.py")
        .assert(predicate::str::contains(
            "from __future__ import annotations",
        ))
        .assert(predicate::str::contains("import os"))
        .assert(predicate::str::contains(
            "pw: str = os.environ[\"DB_PASSWORD\"]",
        ))
        .assert(predicate::str::contains(
            "port: str | int = os.environ.get(\"PORT\", 5432)",
        ))
        .assert(predicate::str::contains("hunter2").not());
    temp.child("s.js")
        .assert(predicate::str::contains("\"pw\": process.env.DB_PASSWORD"))
        .assert(predicate::str::contains(
            "\"port\": (process.env.PORT ?? 5432)",
        ));
    temp.child("s.rb")
        .assert(predicate::str::contains("PW = ENV[\"DB_PASSWORD\"]"))
        .assert(predicate::str::contains("PORT = ENV.fetch(\"PORT\", 5432)"));
    temp.child("s.lua")
        .assert(predicate::str::contains("pw = os.getenv(\"DB_PASSWORD\")"))
        .assert(predicate::str::contains(
            "port = (os.getenv(\"PORT\") or 5432)",
        ));
}

#[test]
fn test_resolve_false_still_requires_env_var() {
    let temp = assert_fs::TempDir::new().unwrap();
    temp.child("settingspec.toml")
        .write_str("[settings]\npw._.default.env = \"MISSING_PW_VAR\"\n")
        .unwrap();
    Command::cargo_bin("settingspec")
        .unwrap()
        .current_dir(temp.path())
        .env_remove("MISSING_PW_VAR")
        .arg("export")
        .assert()
        .failure()
        .stderr(predicate::str::contains("MISSING_PW_VAR"));
}

#[test]
fn test_resolve_false_env_file_uses_shell_reference() {
    let temp = export_unresolved(
        r#"".env" = true"#,
        "pw._.default.env = \"DB_PASSWORD\"\n",
        &[("DB_PASSWORD", "hunter2")],
    );
    temp.child(".env")
        .assert(predicate::str::contains("PW=${DB_PASSWORD}"))
        .assert(predicate::str::contains("hunter2").not());
}

#[test]
fn test_resolve_false_typed_languages_use_string_for_env_only_settings() {
    let temp = export_unresolved(
        r#""s.ts" = true, "s.rs" = true, "s.go" = true, "s.java" = true, "s.scala" = true, "s.cpp" = true, "s.c" = true, "s.zig" = true, "s.hs" = true"#,
        "pw._.default.env = \"DB_PASSWORD\"\n",
        &[("DB_PASSWORD", "hunter2")],
    );
    let expected = [
        ("s.ts", "pw: string;"),
        ("s.ts", "\"pw\": process.env.DB_PASSWORD as string"),
        ("s.rs", "pub fn pw() -> String"),
        ("s.rs", "std::env::var(\"DB_PASSWORD\").expect("),
        ("s.go", "os.Getenv(\"DB_PASSWORD\")"),
        ("s.java", "public static final String pw = "),
        ("s.scala", "lazy val pw: String = sys.env(\"DB_PASSWORD\")"),
        ("s.cpp", "inline const std::string pw = "),
        ("s.c", "const char* (*pw)(void);"),
        ("s.zig", "return std.posix.getenv(\"DB_PASSWORD\").?;"),
        ("s.hs", "pw :: IO String"),
        ("s.hs", "getEnv \"DB_PASSWORD\""),
    ];
    for (file, text) in expected {
        temp.child(file).assert(predicate::str::contains(text));
    }
    for file in [
        "s.ts", "s.rs", "s.go", "s.java", "s.scala", "s.cpp", "s.c", "s.zig", "s.hs",
    ] {
        temp.child(file)
            .assert(predicate::str::contains("hunter2").not());
    }
}

#[test]
fn test_resolve_false_rejects_formats_without_env_access() {
    for file in ["s.elm", "s.tf"] {
        let temp = assert_fs::TempDir::new().unwrap();
        temp.child("settingspec.toml")
            .write_str(&format!(
                "[spec]\nexport.file = {{ \"{}\" = true }}\n\n[settings]\npw._.default.env = \"PW\"\n",
                file
            ))
            .unwrap();
        Command::cargo_bin("settingspec")
            .unwrap()
            .current_dir(temp.path())
            .env("PW", "x")
            .arg("export")
            .assert()
            .failure()
            .stderr(predicate::str::contains("spec.export.resolve = true"));
    }
}

fn export_profiles(files: &[&str], settings: &str, env: &[(&str, &str)]) -> assert_fs::TempDir {
    let temp = assert_fs::TempDir::new().unwrap();
    let file_list: Vec<String> = files.iter().map(|f| format!("\"{}\" = true", f)).collect();
    temp.child("settingspec.toml")
        .write_str(&format!(
            "[spec]\nprofile.options = [\"dev\", \"prod\"]\nprofile.default = \"dev\"\nexport.file = {{ {} }}\n\n[settings]\n{}",
            file_list.join(", "),
            settings
        ))
        .unwrap();
    let mut cmd = Command::cargo_bin("settingspec").unwrap();
    cmd.current_dir(temp.path()).arg("export");
    for (k, v) in env {
        cmd.env(k, v);
    }
    cmd.assert().success();
    temp
}

fn assert_contains(temp: &assert_fs::TempDir, expected: &[(&str, &str)]) {
    for (file, text) in expected {
        temp.child(file).assert(predicate::str::contains(*text));
    }
}

#[test]
fn test_typed_nullable_env_setting_is_optional_string() {
    let temp = export_profiles(
        &[
            "s.ts", "s.rs", "s.go", "s.java", "s.scala", "s.hs", "s.zig", "s.cpp",
        ],
        "pw._.default.env = \"DB_PASSWORD\"\npw._.prod.env = \"DB_PASSWORD\"\npw._.prod.null = true\n",
        &[("DB_PASSWORD", "hunter2")],
    );
    assert_contains(
        &temp,
        &[
            ("s.ts", "pw: string | null;"),
            ("s.ts", "\"pw\": (process.env.DB_PASSWORD ?? null)"),
            ("s.rs", "pub fn pw() -> Option<String>"),
            ("s.go", "Pw *string = func() *string {"),
            (
                "s.java",
                "java.util.Optional<String> pw = java.util.Optional.ofNullable(",
            ),
            (
                "s.scala",
                "lazy val pw: Option[String] = sys.env.get(\"DB_PASSWORD\")",
            ),
            ("s.hs", "pw :: IO (Maybe String)"),
            ("s.zig", "pub fn pw() ?[]const u8"),
            ("s.cpp", "inline const std::optional<std::string> pw ="),
        ],
    );
}

#[test]
fn test_typed_union_of_profile_value_kinds() {
    // int in default, string from the environment in dev, null in prod.
    let temp = export_profiles(
        &[
            "s.ts", "s.rs", "s.go", "s.java", "s.scala", "s.hs", "s.zig", "s.cpp", "s.json",
        ],
        "port._.default.val = 5432\nport._.default.env = \"PORT\"\nport._.dev.env = \"PORT\"\nport._.prod.env = \"PORT\"\nport._.prod.null = true\n",
        &[("PORT", "8080")],
    );
    assert_contains(
        &temp,
        &[
            ("s.ts", "port: string | number | null;"),
            ("s.ts", "\"port\": (process.env.PORT ?? null)"),
            ("s.rs", "pub enum PortValue {"),
            ("s.rs", "    Str(String),"),
            ("s.rs", "    Int(i64),"),
            ("s.rs", "    Null,"),
            ("s.rs", "pub fn port() -> PortValue {"),
            ("s.go", "Port any = func() any {"),
            (
                "s.java",
                "public static final Object port = System.getenv(\"PORT\");",
            ),
            (
                "s.scala",
                "lazy val port: Option[Any] = sys.env.get(\"PORT\")",
            ),
            (
                "s.cpp",
                "std::variant<std::nullptr_t, std::string, long long>",
            ),
            ("s.zig", "pub const PortValue = union(enum) {"),
            ("s.zig", "    str: []const u8,"),
            ("s.zig", "    none: void,"),
            (
                "s.hs",
                "data SettingsPortValue = SettingsPortValueStr String",
            ),
            ("s.hs", "port :: IO SettingsPortValue"),
            ("s.json", "\"port\": \"PORT\""),
        ],
    );
}

#[test]
fn test_typed_union_with_resolved_values() {
    // The same setting, resolved: the active value is a string from the
    // environment, but the type is still the union.
    let temp = assert_fs::TempDir::new().unwrap();
    temp.child("settingspec.toml")
        .write_str(
            r#"
[spec]
profile.options = ["dev", "prod"]
profile.default = "dev"
export.resolve = true
export.file = { "s.ts" = true, "s.rs" = true }

[settings]
port._.default.val = 5432
port._.dev.env = "PORT"
port._.prod.null = true
"#,
        )
        .unwrap();
    Command::cargo_bin("settingspec")
        .unwrap()
        .current_dir(temp.path())
        .env("PORT", "8080")
        .arg("export")
        .assert()
        .success();
    assert_contains(
        &temp,
        &[
            ("s.ts", "port: string | number | null;"),
            // The environment value was parsed as an integer, like the `val`.
            ("s.ts", "\"port\": 8080"),
            ("s.rs", "PortValue::Int(8080)"),
        ],
    );
}

#[test]
fn test_typed_optional_value_keeps_its_kind() {
    let temp = export_profiles(
        &[
            "s.ts", "s.rs", "s.go", "s.java", "s.scala", "s.hs", "s.zig", "s.cpp", "s.elm",
            "s.json",
        ],
        "port._.default.val = 5432\nport._.prod.null = true\n",
        &[],
    );
    assert_contains(
        &temp,
        &[
            ("s.ts", "port: number | null;"),
            ("s.rs", "pub const port: Option<i64> = Some(5432);"),
            ("s.go", "Port *int = func() *int { v := 5432; return &v }()"),
            (
                "s.java",
                "java.util.Optional<Long> port = java.util.Optional.of(5432L);",
            ),
            ("s.scala", "final val port: Option[Long] = Some(5432L)"),
            ("s.hs", "port :: Maybe Int"),
            ("s.hs", "port = Just (5432)"),
            ("s.zig", "pub const port: ?i64 = 5432;"),
            (
                "s.cpp",
                "inline const std::optional<long long> port = 5432;",
            ),
            ("s.elm", "port_ = Just (5432)"),
            // Data formats keep the native type.
            ("s.json", "\"port\": 5432"),
        ],
    );
}

#[test]
fn test_typed_unsupported_shapes_are_rejected() {
    // C only supports string settings, and Rust has no type for a nullable list.
    for (file, settings) in [
        (
            "s.c",
            "port._.default.val = 5432\nport._.prod.null = true\n",
        ),
        (
            "s.rs",
            "list._.default.val = [1, 2]\nlist._.prod.null = true\n",
        ),
    ] {
        let temp = assert_fs::TempDir::new().unwrap();
        temp.child("settingspec.toml")
            .write_str(&format!(
                "[spec]\nprofile.options = [\"dev\", \"prod\"]\nprofile.default = \"dev\"\nexport.file = {{ \"{}\" = true }}\n\n[settings]\n{}",
                file, settings
            ))
            .unwrap();
        Command::cargo_bin("settingspec")
            .unwrap()
            .current_dir(temp.path())
            .arg("export")
            .assert()
            .failure()
            .stderr(predicate::str::contains("cannot be expressed"));
    }
}

#[test]
fn test_python_type_hints_are_the_union_of_profile_types() {
    let temp = export_profiles(
        &["s.py"],
        "port._.default.val = 5432\nport._.default.env = \"PORT\"\nport._.dev.env = \"PORT\"\nport._.prod.env = \"PORT\"\nport._.prod.null = true\nname._.default.val = \"x\"\ntags._.default.val = [\"a\"]\n",
        &[("PORT", "8080")],
    );
    assert_contains(
        &temp,
        &[
            (
                "s.py",
                "port: str | int | None = os.environ[\"PORT\"]",
            ),
            ("s.py", "name: str = \"x\""),
            ("s.py", "tags: list = [\"a\"]"),
        ],
    );
}

fn run_export(settings: &str, resolve: bool, file: &str) -> assert_cmd::assert::Assert {
    let temp = assert_fs::TempDir::new().unwrap();
    temp.child("settingspec.toml")
        .write_str(&format!(
            "[spec]\nprofile.options = [\"dev\", \"prod\"]\nprofile.default = \"dev\"\nexport.resolve = {}\nexport.file = {{ \"{}\" = true }}\n\n[settings]\n{}",
            resolve, file, settings
        ))
        .unwrap();
    let mut cmd = Command::cargo_bin("settingspec").unwrap();
    cmd.current_dir(temp.path())
        .env("KEY", "v")
        .arg("export")
        .assert()
}

#[test]
fn test_unresolved_rejects_mixed_env_and_val_profiles() {
    let settings = "key._.dev.val = \"a\"\nkey._.prod.env = \"KEY\"\n";
    run_export(settings, false, "s.json")
        .failure()
        .stderr(predicate::str::contains("must use 'env' too"));
    run_export(settings, true, "s.json").success();
}

#[test]
fn test_unresolved_static_formats_reject_env_with_fallback() {
    let settings = "key._.dev.env = \"KEY\"\nkey._.dev.val = \"a\"\nkey._.prod.env = \"KEY\"\n";
    for file in ["s.json", "s.toml", "s.yaml", "s.env"] {
        run_export(settings, false, file)
            .failure()
            .stderr(predicate::str::contains("cannot express"));
    }
    run_export(settings, false, "s.py").success();
    run_export(settings, true, "s.json").success();
}
