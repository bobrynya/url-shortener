//! Link entity representing a shortened URL mapping.

use chrono::{DateTime, Utc};

/// A shortened URL link with metadata.
///
/// Represents the mapping between a short code and a long URL within a specific domain.
/// The `domain` field is optional for backward compatibility and joins.
#[derive(Debug, Clone)]
pub struct Link {
    pub id: i64,
    pub code: String,
    pub long_url: String,
    pub domain: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl Link {
    /// Creates a new Link instance.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let link = Link::new(
    ///     1,
    ///     "abc123".to_string(),
    ///     "https://example.com".to_string(),
    ///     Some("s.example.com".to_string()),
    ///     Utc::now(),
    /// );
    /// ```
    pub fn new(
        id: i64,
        code: String,
        long_url: String,
        domain: Option<String>,
        created_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            code,
            long_url,
            domain,
            created_at,
        }
    }
}

/// Input data for creating a new link.
///
/// Used when creating a shortened URL. The `code` should be pre-generated
/// or validated before passing to the repository.
#[derive(Debug, Clone)]
pub struct NewLink {
    pub code: String,
    pub long_url: String,
    pub domain_id: i64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_link_creation() {
        let now = Utc::now();
        let link = Link::new(
            1,
            "abc123".to_string(),
            "https://example.com".to_string(),
            None,
            now,
        );

        assert_eq!(link.id, 1);
        assert_eq!(link.code, "abc123");
        assert_eq!(link.long_url, "https://example.com");
        assert!(link.domain.is_none());
        assert_eq!(link.created_at, now);
    }

    #[test]
    fn test_link_with_domain() {
        let link = Link::new(
            5,
            "test".to_string(),
            "https://example.com".to_string(),
            Some("s.example.com".to_string()),
            Utc::now(),
        );

        assert_eq!(link.code, "test");
        assert!(link.domain.is_some());
        assert_eq!(link.domain.unwrap(), "s.example.com");
    }

    #[test]
    fn test_new_link_creation() {
        let new_link = NewLink {
            code: "xyz789".to_string(),
            long_url: "https://rust-lang.org".to_string(),
            domain_id: 42,
        };

        assert_eq!(new_link.code, "xyz789");
        assert_eq!(new_link.long_url, "https://rust-lang.org");
        assert_eq!(new_link.domain_id, 42);
    }
}
