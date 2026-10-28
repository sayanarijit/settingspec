use super::common::is_valid_identifier;
use super::json::{generate_json_with_env, toml_to_json_value};
use crate::error::Result;
use crate::model::{EnvFallback, EnvRef, ResolvedSetting};

pub fn generate_javascript(settings: &[ResolvedSetting]) -> Result<String> {
    Ok(format!(
        "export default {};\n",
        generate_js_object(settings, env_expression)?
    ))
}

/// Renders the settings as a JSON-shaped object literal in which environment
/// variable references are the expressions returned by `env_expr` instead of
/// strings.
pub(super) fn generate_js_object(
    settings: &[ResolvedSetting],
    env_expr: fn(&EnvRef, bool) -> String,
) -> Result<String> {
    let mut exprs = Vec::new();
    let mut out = generate_json_with_env(settings, &mut |r, nullable| {
        let token = format!("__SETTINGSPEC_ENV_{}__", exprs.len());
        exprs.push((token.clone(), env_expr(r, nullable)));
        serde_json::Value::String(token)
    })?;
    for (token, expr) in exprs {
        out = out.replace(&format!("\"{}\"", token), &expr);
    }
    Ok(out)
}

/// `process.env.NAME`, or `process.env["NAME"]` when NAME is not an identifier.
pub(super) fn process_env(name: &str) -> String {
    if is_valid_identifier(name) {
        format!("process.env.{}", name)
    } else {
        format!("process.env[{}]", serde_json::to_string(name).unwrap())
    }
}

pub(super) fn env_expression(r: &EnvRef, _nullable: bool) -> String {
    let access = process_env(&r.name);
    match &r.fallback {
        None => access,
        Some(EnvFallback::Null) => format!("({} ?? null)", access),
        Some(EnvFallback::Value(v)) => {
            let literal = serde_json::to_string(&toml_to_json_value(v)).unwrap();
            format!("({} ?? {})", access, literal)
        }
    }
}
