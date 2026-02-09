//! Domain extraction from HTTP request headers.

use crate::AppError;
use axum::http::{HeaderMap, header};

/// Extracts the domain name from HTTP request headers.
///
/// Parses the `Host` header to extract the domain name, handling:
/// - IPv4 addresses (e.g., `192.168.1.1`)
/// - IPv6 addresses (e.g., `[::1]`)
/// - Hostnames with ports (e.g., `example.com:3000`)
/// - Plain hostnames (e.g., `example.com`)
///
/// Port numbers are stripped from the result.
///
/// # Errors
///
/// Returns [`AppError::Validation`] if:
/// - The `Host` header is missing
/// - The header value contains invalid UTF-8
///
/// # Examples
///
/// ```ignore
/// let mut headers = HeaderMap::new();
/// headers.insert(header::HOST, "example.com:8080".parse().unwrap());
///
/// let domain = extract_domain_from_headers(&headers).unwrap();
/// assert_eq!(domain, "example.com");
/// ```
pub fn extract_domain_from_headers(headers: &HeaderMap) -> Result<String, AppError> {
    let host = headers
        .get(header::HOST)
        .ok_or_else(|| AppError::bad_request("Missing Host header", serde_json::json!({})))?
        .to_str()
        .map_err(|_| AppError::bad_request("Invalid Host header", serde_json::json!({})))?;

    let domain = if host.starts_with('[') {
        // IPv6 address (e.g., [::1] or [::1]:8080)
        if let Some(end_bracket) = host.find(']') {
            host[..=end_bracket].to_string()
        } else {
            host.to_string()
        }
    } else {
        // IPv4, hostname, or localhost - strip port if present
        host.split(':').next().unwrap_or(host).to_string()
    };

    Ok(domain)
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::{HeaderMap, HeaderValue, header};

    #[test]
    fn test_extract_domain_simple() {
        let mut headers = HeaderMap::new();
        headers.insert(header::HOST, HeaderValue::from_static("example.com"));

        let result = extract_domain_from_headers(&headers);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "example.com");
    }

    #[test]
    fn test_extract_domain_with_port() {
        let mut headers = HeaderMap::new();
        headers.insert(header::HOST, HeaderValue::from_static("example.com:3000"));

        let result = extract_domain_from_headers(&headers);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "example.com");
    }

    #[test]
    fn test_extract_domain_localhost() {
        let mut headers = HeaderMap::new();
        headers.insert(header::HOST, HeaderValue::from_static("localhost"));

        let result = extract_domain_from_headers(&headers);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "localhost");
    }

    #[test]
    fn test_extract_domain_localhost_with_port() {
        let mut headers = HeaderMap::new();
        headers.insert(header::HOST, HeaderValue::from_static("localhost:8080"));

        let result = extract_domain_from_headers(&headers);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "localhost");
    }

    #[test]
    fn test_extract_domain_ip_address() {
        let mut headers = HeaderMap::new();
        headers.insert(header::HOST, HeaderValue::from_static("192.168.1.1"));

        let result = extract_domain_from_headers(&headers);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "192.168.1.1");
    }

    #[test]
    fn test_extract_domain_ip_with_port() {
        let mut headers = HeaderMap::new();
        headers.insert(header::HOST, HeaderValue::from_static("192.168.1.1:9000"));

        let result = extract_domain_from_headers(&headers);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "192.168.1.1");
    }

    #[test]
    fn test_extract_domain_subdomain() {
        let mut headers = HeaderMap::new();
        headers.insert(header::HOST, HeaderValue::from_static("api.example.com"));

        let result = extract_domain_from_headers(&headers);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "api.example.com");
    }

    #[test]
    fn test_extract_domain_subdomain_with_port() {
        let mut headers = HeaderMap::new();
        headers.insert(
            header::HOST,
            HeaderValue::from_static("api.example.com:443"),
        );

        let result = extract_domain_from_headers(&headers);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "api.example.com");
    }

    #[test]
    fn test_extract_domain_missing_host_header() {
        let headers = HeaderMap::new();

        let result = extract_domain_from_headers(&headers);
        assert!(result.is_err());
    }

    #[test]
    fn test_extract_domain_invalid_utf8() {
        let mut headers = HeaderMap::new();
        let invalid_bytes = vec![0xFF, 0xFE, 0xFD];
        if let Ok(header_value) = HeaderValue::from_bytes(&invalid_bytes) {
            headers.insert(header::HOST, header_value);

            let result = extract_domain_from_headers(&headers);
            assert!(result.is_err());
        }
    }

    #[test]
    fn test_extract_domain_empty_headers() {
        let headers = HeaderMap::new();

        let result = extract_domain_from_headers(&headers);
        assert!(result.is_err());
    }

    #[test]
    fn test_extract_domain_ipv6_with_port() {
        let mut headers = HeaderMap::new();
        headers.insert(header::HOST, HeaderValue::from_static("[::1]:8080"));

        let result = extract_domain_from_headers(&headers);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "[::1]");
    }
}
