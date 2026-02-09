//! Click event model for asynchronous click tracking.

/// An in-memory representation of a click event for async processing.
///
/// Used to pass click information from HTTP handlers to the background worker
/// via a channel. This decouples the HTTP response from database writes,
/// allowing fast redirects without blocking.
///
/// # Design
///
/// - Contains denormalized data (domain name + code) to avoid lookups in handlers
/// - All client metadata is optional to handle missing headers gracefully
/// - Cloneable for sending across async boundaries
///
/// # Usage Flow
///
/// 1. Created in redirect handler with request metadata
/// 2. Sent to channel (non-blocking)
/// 3. Processed by [`crate::domain::click_worker::run_click_worker`]
/// 4. Converted to [`crate::domain::entities::NewClick`] for persistence
#[derive(Debug, Clone)]
pub struct ClickEvent {
    pub domain: String,
    pub code: String,
    pub user_agent: Option<String>,
    pub referer: Option<String>,
    pub ip: Option<String>,
}

impl ClickEvent {
    /// Creates a new click event.
    ///
    /// # Arguments
    ///
    /// - `domain` - The domain name serving the short link (e.g., "s.example.com")
    /// - `code` - The short code that was accessed
    /// - `ip` - Optional client IP address
    /// - `user_agent` - Optional User-Agent header
    /// - `referer` - Optional Referer header
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let event = ClickEvent::new(
    ///     "s.example.com".to_string(),
    ///     "abc123".to_string(),
    ///     Some("192.168.1.1".to_string()),
    ///     Some("Mozilla/5.0"),
    ///     Some("https://google.com"),
    /// );
    /// ```
    pub fn new(
        domain: String,
        code: String,
        ip: Option<String>,
        user_agent: Option<&str>,
        referer: Option<&str>,
    ) -> Self {
        Self {
            domain,
            code,
            ip,
            user_agent: user_agent.map(|s| s.to_string()),
            referer: referer.map(|s| s.to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_click_event_creation_full() {
        let event = ClickEvent::new(
            "s.example.com".to_string(),
            "abc123".to_string(),
            Some("192.168.1.1".to_string()),
            Some("Mozilla/5.0"),
            Some("https://google.com"),
        );

        assert_eq!(event.domain, "s.example.com");
        assert_eq!(event.code, "abc123");
        assert_eq!(event.ip, Some("192.168.1.1".to_string()));
        assert_eq!(event.user_agent, Some("Mozilla/5.0".to_string()));
        assert_eq!(event.referer, Some("https://google.com".to_string()));
    }

    #[test]
    fn test_click_event_creation_minimal() {
        let event = ClickEvent::new(
            "short.link".to_string(),
            "xyz".to_string(),
            None,
            None,
            None,
        );

        assert_eq!(event.domain, "short.link");
        assert_eq!(event.code, "xyz");
        assert!(event.ip.is_none());
        assert!(event.user_agent.is_none());
        assert!(event.referer.is_none());
    }

    #[test]
    fn test_click_event_str_conversion() {
        let user_agent = "Chrome/120";
        let referer = "https://example.com";

        let event = ClickEvent::new(
            "d.com".to_string(),
            "test".to_string(),
            Some("10.0.0.1".to_string()),
            Some(user_agent),
            Some(referer),
        );

        assert_eq!(event.user_agent.unwrap(), user_agent.to_string());
        assert_eq!(event.referer.unwrap(), referer.to_string());
    }

    #[test]
    fn test_click_event_clone() {
        let event = ClickEvent::new(
            "s.com".to_string(),
            "code1".to_string(),
            Some("1.1.1.1".to_string()),
            Some("Safari"),
            None,
        );

        let cloned = event.clone();

        assert_eq!(cloned.domain, event.domain);
        assert_eq!(cloned.code, event.code);
        assert_eq!(cloned.ip, event.ip);
        assert_eq!(cloned.user_agent, event.user_agent);
    }
}
