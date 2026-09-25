use crate::envfile::source_env_file;
use crate::error::{Result, SettingSpecError};
use crate::filter::matches_filter;
use crate::generator::{generate_by_format, generate_by_path};
use crate::model::*;
use crate::parser::parse_config_str;
use crate::profile::resolve_active_profile;
use crate::resolver::resolve_settings;
use clap::{Parser, Subcommand};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Parser, Debug)]
#[command(
    name = "settingspec",
    version,
    about = "One file for all environments, all languages, all submodules."
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Generates all configured export files and/or writes formatted output to stdout
    Export,
    /// Validates configuration syntax, profile completeness, and environment variables
    Check,
    /// Executes a command with resolved configuration available as exported files
    Run {
        #[arg(last = true, required = true)]
        command: Vec<String>,
    },
}

pub struct ExecutionContext {
    pub doc: SettingSpecDocument,
    pub active_profile: Option<String>,
    pub resolved_settings: Vec<ResolvedSetting>,
    pub sourced_env: HashMap<String, String>,
}

pub fn load_and_resolve() -> Result<ExecutionContext> {
    let config_path = Path::new("settingspec.toml");
    if !config_path.exists() {
        return Err(SettingSpecError::ConfigNotFound);
    }

    let content = std::fs::read_to_string(config_path)?;
    let doc = parse_config_str(&content)?;

    let active_profile = resolve_active_profile(&doc.spec.profile)?;
    let sourced_env = source_env_file(&doc.spec.envfile, active_profile.as_deref())?;
    let resolved_settings = resolve_settings(
        &doc.settings,
        &doc.spec,
        active_profile.as_deref(),
        &sourced_env,
    )?;

    Ok(ExecutionContext {
        doc,
        active_profile,
        resolved_settings,
        sourced_env,
    })
}

pub fn run_cli() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Check => {
            // Check only validates config syntax, profiles, env vars without creating files
            load_and_resolve()?;
            Ok(())
        }
        Commands::Export => {
            let ctx = load_and_resolve()?;
            perform_export(&ctx)?;
            Ok(())
        }
        Commands::Run { command } => {
            let ctx = load_and_resolve()?;
            execute_run(&ctx, &command)
        }
    }
}

fn perform_export(ctx: &ExecutionContext) -> Result<Vec<PathBuf>> {
    auto_append_gitignore(ctx)?;

    let mut generated_files = Vec::new();

    // 1. Export files
    for (file_path_str, filter) in &ctx.doc.spec.export.file {
        if *filter == ExportFilter::Disabled {
            continue;
        }

        let filtered_settings: Vec<ResolvedSetting> = ctx
            .resolved_settings
            .iter()
            .filter(|s| matches_filter(filter, s))
            .cloned()
            .collect();

        let path = Path::new(file_path_str);
        let output = generate_by_path(path, &filtered_settings)?;
        write_export_file(path, &output, ctx.doc.spec.export.mode)?;
        generated_files.push(path.to_path_buf());
    }

    // 2. Stdout export if configured
    if let Some(ref stdout_fmt) = ctx.doc.spec.export.stdout {
        let output = generate_by_format(stdout_fmt, &ctx.resolved_settings)?;
        print!("{}", output);
    }

    Ok(generated_files)
}

fn execute_run(ctx: &ExecutionContext, command_args: &[String]) -> Result<()> {
    if command_args.is_empty() {
        return Err(SettingSpecError::InvalidToml(
            "No command specified to run".into(),
        ));
    }

    // Export all files
    let generated_files = perform_export(ctx)?;

    // Setup cleanup guard
    struct CleanupGuard {
        files: Vec<PathBuf>,
        keep: bool,
    }

    impl Drop for CleanupGuard {
        fn drop(&mut self) {
            if !self.keep {
                for f in &self.files {
                    let _ = std::fs::remove_file(f);
                }
            }
        }
    }

    let _guard = CleanupGuard {
        files: generated_files,
        keep: ctx.doc.spec.export.keep,
    };

    let stdin_payload = if let Some(ref stdin_fmt) = ctx.doc.spec.export.stdin {
        Some(generate_by_format(stdin_fmt, &ctx.resolved_settings)?)
    } else {
        None
    };

    let prog = &command_args[0];
    let args = &command_args[1..];

    let mut cmd = std::process::Command::new(prog);
    cmd.args(args);
    for (k, v) in &ctx.sourced_env {
        cmd.env(k, v);
    }

    if let Some(ref prefix) = ctx.doc.spec.export.env {
        for setting in &ctx.resolved_settings {
            if let SettingValue::Value(ref val) = setting.value {
                let env_key = format!(
                    "{}{}",
                    prefix,
                    crate::generator::env::format_env_key(&setting.key)
                );
                let env_val = crate::generator::env::format_env_value(val);
                cmd.env(env_key, env_val);
            }
        }
    }

    let status = if let Some(payload) = stdin_payload {
        cmd.stdin(std::process::Stdio::piped());
        let mut child = cmd.spawn().map_err(|e| {
            SettingSpecError::IoError(format!("Failed to execute '{}': {}", prog, e))
        })?;

        let child_stdin = child.stdin.take();
        let writer = std::thread::spawn(move || {
            if let Some(mut stdin) = child_stdin {
                use std::io::Write;
                let _ = stdin.write_all(payload.as_bytes());
            }
        });

        let status = child.wait().map_err(|e| {
            SettingSpecError::IoError(format!("Failed to wait for '{}': {}", prog, e))
        })?;

        let _ = writer.join();
        status
    } else {
        cmd.status().map_err(|e| {
            SettingSpecError::IoError(format!("Failed to execute '{}': {}", prog, e))
        })?
    };

    if !status.success() {
        let code = status.code().unwrap_or_else(|| {
            #[cfg(unix)]
            {
                use std::os::unix::process::ExitStatusExt;
                if let Some(sig) = status.signal() {
                    return 128 + sig;
                }
            }
            1
        });
        return Err(SettingSpecError::CommandFailed(code));
    }

    Ok(())
}

pub fn write_export_file(path: &Path, content: &str, mode: u32) -> Result<()> {
    if let Some(parent) = path.parent()
        && !parent.as_os_str().is_empty()
    {
        std::fs::create_dir_all(parent)?;
    }

    std::fs::write(path, content)?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = std::fs::Permissions::from_mode(mode);
        let _ = std::fs::set_permissions(path, perms);
    }

    Ok(())
}

pub fn find_git_root(start_dir: &Path) -> Option<PathBuf> {
    let mut current = if start_dir.is_relative() {
        match std::env::current_dir() {
            Ok(cwd) => cwd.join(start_dir),
            Err(_) => start_dir.to_path_buf(),
        }
    } else {
        start_dir.to_path_buf()
    };
    if let Ok(canon) = current.canonicalize() {
        current = canon;
    }
    loop {
        let git_entry = current.join(".git");
        if git_entry.exists() {
            return Some(current);
        }
        if !current.pop() {
            break;
        }
    }
    None
}

fn normalize_path(path: &Path) -> PathBuf {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            std::path::Component::Prefix(prefix) => normalized.push(prefix.as_os_str()),
            std::path::Component::RootDir => normalized.push(std::path::MAIN_SEPARATOR.to_string()),
            std::path::Component::CurDir => {}
            std::path::Component::ParentDir => {
                normalized.pop();
            }
            std::path::Component::Normal(c) => normalized.push(c),
        }
    }
    normalized
}

fn is_entry_in_gitignore(existing_content: &str, entry: &str) -> bool {
    let norm_entry = entry.trim_start_matches('/').trim_end_matches('/');
    for line in existing_content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('#') || trimmed.is_empty() {
            continue;
        }
        let norm_line = trimmed.trim_start_matches('/').trim_end_matches('/');
        if norm_line == norm_entry {
            return true;
        }
        // In git, a pattern without '/' matches anywhere in the repository:
        // E.g. if .gitignore has "settings.toml", it matches "subdir/settings.toml".
        if !trimmed.starts_with('/') && !norm_line.contains('/') {
            if let Some((_, file_name)) = norm_entry.rsplit_once('/') {
                if file_name == norm_line {
                    return true;
                }
            }
        }
    }
    false
}

pub fn auto_append_gitignore(ctx: &ExecutionContext) -> Result<()> {
    if ctx.doc.spec.export.skip_gitignore {
        return Ok(());
    }

    let current_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let git_root = match find_git_root(&current_dir) {
        Some(root) => root,
        None => return Ok(()),
    };

    let git_root_canon = git_root.canonicalize().unwrap_or_else(|_| git_root.clone());
    let current_dir_canon = current_dir
        .canonicalize()
        .unwrap_or_else(|_| current_dir.clone());

    let mut entries_to_add = Vec::new();

    for (file_path_str, filter) in &ctx.doc.spec.export.file {
        if *filter == ExportFilter::Disabled {
            continue;
        }

        let target_path = Path::new(file_path_str);
        let full_target = if target_path.is_relative() {
            current_dir_canon.join(target_path)
        } else {
            target_path.to_path_buf()
        };

        let normalized = normalize_path(&full_target);
        let rel_path = match normalized.strip_prefix(&git_root_canon) {
            Ok(rel) => rel,
            Err(_) => continue,
        };

        let mut parts = Vec::new();
        for comp in rel_path.components() {
            if let std::path::Component::Normal(c) = comp {
                parts.push(c.to_string_lossy().to_string());
            }
        }

        if !parts.is_empty() {
            let entry = format!("/{}", parts.join("/"));
            if !entries_to_add.contains(&entry) {
                entries_to_add.push(entry);
            }
        }
    }

    if entries_to_add.is_empty() {
        return Ok(());
    }

    let gitignore_path = git_root.join(".gitignore");
    let existing_content = if gitignore_path.exists() {
        std::fs::read_to_string(&gitignore_path)?
    } else {
        String::new()
    };

    let mut missing_entries = Vec::new();
    for entry in entries_to_add {
        if !is_entry_in_gitignore(&existing_content, &entry) && !missing_entries.contains(&entry) {
            missing_entries.push(entry);
        }
    }

    if missing_entries.is_empty() {
        return Ok(());
    }

    let mut to_append = String::new();
    if !existing_content.is_empty() && !existing_content.ends_with('\n') {
        to_append.push('\n');
    }
    for entry in &missing_entries {
        to_append.push_str(entry);
        to_append.push('\n');
    }

    use std::io::Write;
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&gitignore_path)?;
    file.write_all(to_append.as_bytes())?;

    Ok(())
}
