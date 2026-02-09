//! Short code generation and validation utilities.
//!
//! Provides cryptographically secure random code generation and validation
//! for custom user-provided codes.

use crate::error::AppError;
use base64::Engine as _;
use serde_json::json;

/// Length of random bytes before base64 encoding.
const CODE_LENGTH_BYTES: usize = 9;

/// Reserved codes that cannot be used as short links.
///
/// These codes are reserved for system endpoints to prevent routing conflicts.
const RESERVED_CODES: &[&str] = &["stats", "health", "domains", "admin", "api", "dashboard"];

/// Generates a cryptographically secure random short code.
///
/// Uses `getrandom` for entropy and encodes the result as URL-safe base64
/// without padding, producing a 12-character code.
///
/// # Panics
///
/// Panics if the system random number generator fails (extremely rare).
///
/// # Examples
///
/// ```ignore
/// let code = generate_code();
/// assert_eq!(code.len(), 12);
/// assert!(code.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_'));
/// ```
pub fn generate_code() -> String {
    let mut buffer = [0u8; CODE_LENGTH_BYTES];

    getrandom::fill(&mut buffer).expect("Failed to generate random bytes");

    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(buffer)
}

/// Validates a user-provided custom short code.
///
/// # Rules
///
/// - Length: 8-15 characters
/// - Allowed characters: lowercase letters, digits, hyphens
/// - Cannot start or end with a hyphen
/// - Cannot be a reserved system code
///
/// # Errors
///
/// Returns [`AppError::Validation`] if any validation rule is violated.
///
/// # Examples
///
/// ```ignore
/// // Valid codes
/// assert!(validate_custom_code("my-link-2024").is_ok());
/// assert!(validate_custom_code("promo2025").is_ok());
///
/// // Invalid codes
/// assert!(validate_custom_code("short").is_err());        // Too short
/// assert!(validate_custom_code("MyCode").is_err());       // Uppercase
/// assert!(validate_custom_code("-invalid").is_err());     // Starts with hyphen
/// assert!(validate_custom_code("admin").is_err());        // Reserved
/// ```
pub fn validate_custom_code(code: &str) -> Result<(), AppError> {
    if code.len() < 8 || code.len() > 15 {
        return Err(AppError::bad_request(
            "Custom code must be 8-15 characters",
            json!({ "provided_length": code.len() }),
        ));
    }

    if !code
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
    {
        return Err(AppError::bad_request(
            "Custom code can only contain lowercase letters, digits, and hyphens",
            json!({ "code": code }),
        ));
    }

    if code.starts_with('-') || code.ends_with('-') {
        return Err(AppError::bad_request(
            "Custom code cannot start or end with a hyphen",
            json!({ "code": code }),
        ));
    }

    if RESERVED_CODES.contains(&code) {
        return Err(AppError::bad_request(
            "This code is reserved",
            json!({ "code": code }),
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn test_generate_code_not_empty() {
        let code = generate_code();
        assert!(!code.is_empty());
    }

    #[test]
    fn test_generate_code_has_correct_length() {
        let code = generate_code();
        assert_eq!(code.len(), 12);
    }

    #[test]
    fn test_generate_code_url_safe_characters() {
        let code = generate_code();
        assert!(
            code.chars()
                .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
        );
    }

    #[test]
    fn test_generate_code_produces_unique_codes() {
        let mut codes = HashSet::new();

        for _ in 0..1000 {
            let code = generate_code();
            codes.insert(code);
        }

        assert_eq!(codes.len(), 1000);
    }

    #[test]
    fn test_generate_code_no_padding() {
        let code = generate_code();
        assert!(!code.contains('='));
    }

    #[test]
    fn test_validate_minimum_length() {
        let result = validate_custom_code("abcd1234");
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_maximum_length() {
        let result = validate_custom_code("abcd1234567890x");
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_with_hyphens_in_middle() {
        let result = validate_custom_code("my-cool-link");
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_only_digits() {
        let result = validate_custom_code("12345678");
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_only_lowercase() {
        let result = validate_custom_code("abcdefgh");
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_mixed_valid_chars() {
        let result = validate_custom_code("abc-123-xyz");
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_too_short() {
        let result = validate_custom_code("abc123");
        assert!(result.is_err());

        let err = result.unwrap_err();
        assert!(err.to_string().contains("8-15 characters"));
    }

    #[test]
    fn test_validate_too_long() {
        let result = validate_custom_code("abcd1234567890xyz");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_uppercase_letters() {
        let result = validate_custom_code("MyCode123");
        assert!(result.is_err());

        let err = result.unwrap_err();
        assert!(err.to_string().contains("lowercase"));
    }

    #[test]
    fn test_validate_special_characters() {
        let result = validate_custom_code("my_code@123");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_starts_with_hyphen() {
        let result = validate_custom_code("-mycode123");
        assert!(result.is_err());

        let err = result.unwrap_err();
        assert!(err.to_string().contains("cannot start or end"));
    }

    #[test]
    fn test_validate_ends_with_hyphen() {
        let result = validate_custom_code("mycode123-");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_reserved_code_stats() {
        let result = validate_custom_code("stats");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_reserved_code_admin() {
        let result = validate_custom_code("admin");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_all_reserved_codes() {
        for &reserved in RESERVED_CODES {
            let result = validate_custom_code(reserved);
            assert!(
                result.is_err(),
                "Reserved code '{}' should be invalid",
                reserved
            );
        }
    }

    #[test]
    fn test_validate_spaces_not_allowed() {
        let result = validate_custom_code("my code 123");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_empty_string() {
        let result = validate_custom_code("");
        assert!(result.is_err());
    }
}
