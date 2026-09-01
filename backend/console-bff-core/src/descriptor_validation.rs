//! Descriptor validation for service/resource presentation metadata.
//!
//! Descriptors are treated as a supply-chain input (see `docs/security/threat-model.md`).
//! This module rejects executable-looking keys and values before they reach the
//! browser, as defense-in-depth in addition to the stricter validation performed
//! by the frontend runtime.

use crate::error::ApiError;

/// Object keys that are always rejected because they look executable or unsafe.
const FORBIDDEN_KEY_PATTERNS: &[&str] = &["eval", "exec"];

/// Object keys that are explicitly rejected regardless of casing.
const FORBIDDEN_KEY_LITERALS: &[&str] = &["$exec", "x-araf-script", "javascript"];

/// Returns true for event-handler script keys such as `onClickScript` or
/// `onLoadScript`. These are never legitimate descriptor fields.
fn is_event_handler_script_key(key: &str) -> bool {
    let lower = key.to_lowercase();
    lower.starts_with("on") && lower.ends_with("script")
}

/// String prefixes/substrings that indicate executable content.
const FORBIDDEN_STRING_PATTERNS: &[&str] = &["javascript:", "data:text/html", "<script"];

/// URL-like prefixes used to detect arbitrary URLs outside documentation fields.
const URL_LIKE_PREFIXES: &[&str] = &["http://", "https://", "//", "data:", "file:"];

/// Documentation fields that are allowed to contain URLs.
const DOCUMENTATION_FIELDS: &[&str] = &["documentationUrl", "documentation_url"];

fn is_forbidden_key(key: &str) -> bool {
    let lower = key.to_lowercase();
    if FORBIDDEN_KEY_LITERALS.iter().any(|l| lower == *l) {
        return true;
    }
    if is_event_handler_script_key(key) {
        return true;
    }
    FORBIDDEN_KEY_PATTERNS.iter().any(|pat| lower.contains(pat))
}

fn contains_forbidden_string(value: &str) -> bool {
    let lower = value.to_lowercase();
    FORBIDDEN_STRING_PATTERNS
        .iter()
        .any(|pat| lower.contains(pat))
}

fn looks_like_url(value: &str) -> bool {
    URL_LIKE_PREFIXES
        .iter()
        .any(|prefix| value.to_lowercase().starts_with(prefix))
}

fn validate_value(value: &serde_json::Value, path: &str, allow_url: bool) -> Result<(), ApiError> {
    match value {
        serde_json::Value::Object(map) => {
            for (key, child) in map {
                if is_forbidden_key(key) {
                    return Err(ApiError::BadRequest(format!(
                        "dangerous descriptor key at {path}.{key}: executable/unsafe keys are not allowed"
                    )));
                }
                let child_allow_url = DOCUMENTATION_FIELDS.contains(&key.as_str());
                let child_path = if path.is_empty() {
                    key.clone()
                } else {
                    format!("{path}.{key}")
                };
                validate_value(child, &child_path, child_allow_url)?;
            }
        }
        serde_json::Value::Array(items) => {
            for (index, item) in items.iter().enumerate() {
                validate_value(item, &format!("{path}[{index}]"), allow_url)?;
            }
        }
        serde_json::Value::String(s) => {
            if contains_forbidden_string(s) {
                return Err(ApiError::BadRequest(format!(
                    "dangerous descriptor value at {path}: executable/unsafe strings are not allowed"
                )));
            }
            if !allow_url && looks_like_url(s) {
                return Err(ApiError::BadRequest(format!(
                    "dangerous descriptor value at {path}: arbitrary URLs are not allowed outside documentation fields"
                )));
            }
        }
        _ => {}
    }
    Ok(())
}

/// Validate a JSON descriptor value for executable or unsafe content.
///
/// Returns `ApiError::BadRequest` with a detailed message when a dangerous key,
/// executable-looking string, or arbitrary URL outside a documentation field is
/// found.
pub fn validate_descriptor_json(value: &serde_json::Value) -> Result<(), ApiError> {
    validate_value(value, "descriptor", false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn accepts_safe_descriptor() {
        let descriptor = json!({
            "id": "compute.server",
            "name": "Server",
            "documentationUrl": "https://docs.example.com/compute"
        });
        assert!(validate_descriptor_json(&descriptor).is_ok());
    }

    #[test]
    fn rejects_executable_string_values() {
        for value in [
            "javascript:alert(1)",
            "<script>alert(1)</script>",
            "data:text/html,<script>alert(1)</script>",
        ] {
            let descriptor = json!({"name": value});
            let err = validate_descriptor_json(&descriptor).expect_err("should reject {value}");
            assert!(
                err.to_string().contains("executable/unsafe strings"),
                "got {err}"
            );
        }
    }

    #[test]
    fn rejects_dangerous_keys() {
        for key in [
            "onClickScript",
            "onLoadScript",
            "evalExpr",
            "execProfile",
            "$exec",
            "x-araf-script",
            "javascript",
        ] {
            let descriptor = json!({key: "value"});
            let err = validate_descriptor_json(&descriptor).expect_err("should reject {key}");
            assert!(
                err.to_string().contains("dangerous descriptor key"),
                "got {err}"
            );
        }
    }

    #[test]
    fn allows_description_and_documentation_url() {
        let descriptor = json!({
            "description": "A perfectly normal service description",
            "documentationUrl": "https://docs.example.com/service"
        });
        assert!(validate_descriptor_json(&descriptor).is_ok());
    }

    #[test]
    fn rejects_arbitrary_urls_outside_documentation() {
        let descriptor = json!({"iconToken": "https://example.com/icon.svg"});
        let err = validate_descriptor_json(&descriptor).expect_err("should reject url");
        assert!(err.to_string().contains("arbitrary URLs"), "got {err}");
    }

    #[test]
    fn allows_documentation_url() {
        let descriptor = json!({
            "documentationUrl": "https://docs.example.com/service",
            "documentation_url": "https://docs.example.com/service"
        });
        assert!(validate_descriptor_json(&descriptor).is_ok());
    }
}
