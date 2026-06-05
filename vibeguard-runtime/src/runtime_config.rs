use std::io::ErrorKind;
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeConfigError {
    pub message: String,
    pub exit_code: i32,
}

impl RuntimeConfigError {
    fn config_parse_error(message: String) -> Self {
        Self {
            message,
            exit_code: 30,
        }
    }

    fn policy_error(message: String) -> Self {
        Self {
            message,
            exit_code: 20,
        }
    }
}

pub fn validate_runtime_config_file(path_text: &str) -> Result<(), RuntimeConfigError> {
    if path_text.is_empty() {
        return Ok(());
    }

    let path = Path::new(path_text);
    if !path.is_file() {
        return Ok(());
    }

    let text = std::fs::read_to_string(path).map_err(|err| {
        if err.kind() == ErrorKind::InvalidData {
            RuntimeConfigError::config_parse_error(format!(
                "VibeGuard runtime config invalid UTF-8: {}: {err}",
                path.display()
            ))
        } else {
            RuntimeConfigError::policy_error(format!(
                "VibeGuard runtime config cannot be read: {}: {err}",
                path.display()
            ))
        }
    })?;

    serde_json::from_str::<serde_json::Value>(&text).map_err(|err| {
        RuntimeConfigError::config_parse_error(format!(
            "VibeGuard runtime config invalid JSON: {}: {err}",
            path.display()
        ))
    })?;

    Ok(())
}
