use super::json::generate_json;
use crate::error::Result;
use crate::model::ResolvedSetting;

pub fn generate_javascript(settings: &[ResolvedSetting]) -> Result<String> {
    let json_str = generate_json(settings)?;
    Ok(format!("export default {};\n", json_str))
}
