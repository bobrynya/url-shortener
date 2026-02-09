//! URL normalization and sanitization utilities.
//!
//! Ensures consistent URL representation by normalizing hostnames, removing
//! fragments, and handling default ports.

use url::Url;

/// Errors that can occur during URL normalization.
#[derive(Debug, thiserror::Error)]
pub enum UrlNormalizationError {
    #[error("Invalid URL format: {0}")]
    InvalidFormat(String),

    #[error("Only HTTP and HTTPS protocols are allowed")]
    UnsupportedProtocol,

    #[error("Failed to normalize URL: {0}")]
    NormalizationFailed(String),
}

/// Normalizes a URL to a canonical form.
///
/// # Normalization Rules
///
/// 1. **Protocol**: Only HTTP and HTTPS are allowed
/// 2. **Hostname**: Converted to lowercase
/// 3. **Default ports**: Removed (80 for HTTP, 443 for HTTPS)
/// 4. **Fragments**: Removed (e.g., `#section`)
/// 5. **Query parameters**: Preserved as-is
/// 6. **Path**: Preserved with case sensitivity
///
/// # Security
///
/// Rejects potentially dangerous protocols like `javascript:`, `data:`, `file:`, etc.
///
/// # Errors
///
/// Returns [`UrlNormalizationError::InvalidFormat`] for malformed URLs.
/// Returns [`UrlNormalizationError::UnsupportedProtocol`] for non-HTTP(S) schemes.
///
/// # Examples
///
/// ```ignore
/// // Case normalization
/// assert_eq!(
///     normalize_url("HTTPS://EXAMPLE.COM/Path").unwrap(),
///     "https://example.com/Path"
/// );
///
/// // Default port removal
/// assert_eq!(
///     normalize_url("https://example.com:443/path").unwrap(),
///     "https://example.com/path"
/// );
///
/// // Fragment removal
/// assert_eq!(
///     normalize_url("https://example.com/page#section").unwrap(),
///     "https://example.com/page"
/// );
/// ```
pub fn normalize_url(input: &str) -> Result<String, UrlNormalizationError> {
    let mut url =
        Url::parse(input).map_err(|e| UrlNormalizationError::InvalidFormat(e.to_string()))?;

    match url.scheme() {
        "http" | "https" => {}
        _ => return Err(UrlNormalizationError::UnsupportedProtocol),
    }

    if let Some(host) = url.host_str() {
        let host_lowercase = host.to_ascii_lowercase();
        url.set_host(Some(&host_lowercase)).map_err(|_| {
            UrlNormalizationError::NormalizationFailed("Failed to set normalized host".to_string())
        })?;
    }

    url.set_fragment(None);

    let is_default_port = matches!(
        (url.scheme(), url.port()),
        ("http", Some(80)) | ("https", Some(443))
    );
    if is_default_port {
        url.set_port(None).map_err(|_| {
            UrlNormalizationError::NormalizationFailed("Failed to remove default port".to_string())
        })?;
    }

    Ok(url.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_simple_http() {
        let result = normalize_url("http://example.com");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "http://example.com/");
    }

    #[test]
    fn test_normalize_simple_https() {
        let result = normalize_url("https://example.com");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "https://example.com/");
    }

    #[test]
    fn test_normalize_uppercase_host() {
        let result = normalize_url("https://EXAMPLE.COM/path");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "https://example.com/path");
    }

    #[test]
    fn test_normalize_mixed_case_host() {
        let result = normalize_url("https://ExAmPlE.CoM");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "https://example.com/");
    }

    #[test]
    fn test_normalize_remove_default_http_port() {
        let result = normalize_url("http://example.com:80/path");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "http://example.com/path");
    }

    #[test]
    fn test_normalize_remove_default_https_port() {
        let result = normalize_url("https://example.com:443/path");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "https://example.com/path");
    }

    #[test]
    fn test_normalize_keep_custom_port() {
        let result = normalize_url("http://example.com:8080/path");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "http://example.com:8080/path");
    }

    #[test]
    fn test_normalize_remove_fragment() {
        let result = normalize_url("https://example.com/page#section");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "https://example.com/page");
    }

    #[test]
    fn test_normalize_remove_fragment_with_query() {
        let result = normalize_url("https://example.com/page?key=value#section");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "https://example.com/page?key=value");
    }

    #[test]
    fn test_normalize_preserve_query_params() {
        let result = normalize_url("https://example.com/search?q=rust&lang=en");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "https://example.com/search?q=rust&lang=en");
    }

    #[test]
    fn test_normalize_preserve_path() {
        let result = normalize_url("https://example.com/path/to/page");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "https://example.com/path/to/page");
    }

    #[test]
    fn test_normalize_complex_url() {
        let result = normalize_url("HTTPS://EXAMPLE.COM:443/Path?key=VALUE#anchor");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "https://example.com/Path?key=VALUE");
    }

    #[test]
    fn test_normalize_trailing_slash() {
        let result = normalize_url("https://example.com/");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "https://example.com/");
    }

    #[test]
    fn test_normalize_subdomain() {
        let result = normalize_url("https://api.example.com/v1/users");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "https://api.example.com/v1/users");
    }

    #[test]
    fn test_normalize_with_authentication() {
        let result = normalize_url("https://user:pass@example.com/path");
        assert!(result.is_ok());
        assert!(result.unwrap().contains("user:pass"));
    }

    #[test]
    fn test_normalize_ip_address() {
        let result = normalize_url("http://192.168.1.1:8080/api");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "http://192.168.1.1:8080/api");
    }

    #[test]
    fn test_normalize_localhost() {
        let result = normalize_url("http://localhost:3000/test");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "http://localhost:3000/test");
    }

    #[test]
    fn test_normalize_invalid_url() {
        let result = normalize_url("not a valid url");
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            UrlNormalizationError::InvalidFormat(_)
        ));
    }

    #[test]
    fn test_normalize_ftp_protocol() {
        let result = normalize_url("ftp://example.com/file.txt");
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            UrlNormalizationError::UnsupportedProtocol
        ));
    }

    #[test]
    fn test_normalize_file_protocol() {
        let result = normalize_url("file:///home/user/document.txt");
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            UrlNormalizationError::UnsupportedProtocol
        ));
    }

    #[test]
    fn test_normalize_empty_string() {
        let result = normalize_url("");
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            UrlNormalizationError::InvalidFormat(_)
        ));
    }

    #[test]
    fn test_normalize_no_protocol() {
        let result = normalize_url("example.com");
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            UrlNormalizationError::InvalidFormat(_)
        ));
    }

    #[test]
    fn test_normalize_invalid_characters() {
        let result = normalize_url("https://example.com/<invalid>");
        let _ = result;
    }

    #[test]
    fn test_normalize_javascript_protocol() {
        let result = normalize_url("javascript:alert('xss')");
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            UrlNormalizationError::UnsupportedProtocol
        ));
    }

    #[test]
    fn test_normalize_data_protocol() {
        let result = normalize_url("data:text/plain,Hello");
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            UrlNormalizationError::UnsupportedProtocol
        ));
    }

    #[test]
    fn test_normalize_mailto_protocol() {
        let result = normalize_url("mailto:test@example.com");
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            UrlNormalizationError::UnsupportedProtocol
        ));
    }

    #[test]
    fn test_normalize_very_long_url() {
        let long_path = "a".repeat(2000);
        let url = format!("https://example.com/{}", long_path);
        let result = normalize_url(&url);
        assert!(result.is_ok());
        assert!(result.unwrap().len() > 2000);
    }

    #[test]
    fn test_normalize_multiple_query_params() {
        let result = normalize_url("https://example.com/search?a=1&b=2&c=3&d=4");
        assert!(result.is_ok());
        let normalized = result.unwrap();
        assert!(normalized.contains("a=1"));
        assert!(normalized.contains("b=2"));
    }

    #[test]
    fn test_normalize_encoded_characters() {
        let result = normalize_url("https://example.com/path%20with%20spaces");
        assert!(result.is_ok());
        assert!(result.unwrap().contains("path%20with%20spaces"));
    }

    #[test]
    fn test_normalize_unicode_domain() {
        let result = normalize_url("https://münchen.de");
        assert!(result.is_ok());
    }
}
