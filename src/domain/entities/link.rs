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
    pub expires_at: Option<DateTime<Utc>>,
    pub permanent: bool,
    pub deleted_at: Option<DateTime<Utc>>,
}

impl Link {
    /// Creates a new Link instance.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: i64,
        code: String,
        long_url: String,
        domain: Option<String>,
        created_at: DateTime<Utc>,
        expires_at: Option<DateTime<Utc>>,
        permanent: bool,
        deleted_at: Option<DateTime<Utc>>,
    ) -> Self {
        Self {
            id,
            code,
            long_url,
            domain,
            created_at,
            expires_at,
            permanent,
            deleted_at,
        }
    }

    /// Returns true if the link has been soft-deleted.
    pub fn is_deleted(&self) -> bool {
        self.deleted_at.is_some()
    }

    /// Returns true if the link has passed its expiry time.
    pub fn is_expired(&self) -> bool {
        self.expires_at.is_some_and(|e| Utc::now() >= e)
    }
}

/// Input data for creating a new link.
#[derive(Debug, Clone)]
pub struct NewLink {
    pub code: String,
    pub long_url: String,
    pub domain_id: i64,
    pub expires_at: Option<DateTime<Utc>>,
    pub permanent: bool,
}

/// Partial update for an existing link.
///
/// `None` fields are left unchanged.
/// `expires_at: Some(None)` clears the expiry; `Some(Some(t))` sets it.
#[derive(Debug, Clone)]
pub struct LinkPatch {
    pub url: Option<String>,
    pub expires_at: Option<Option<DateTime<Utc>>>,
    pub permanent: Option<bool>,
    /// When `true`, clears `deleted_at` to restore a soft-deleted link.
    pub restore: bool,
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
            None,
            false,
            None,
        );

        assert_eq!(link.id, 1);
        assert_eq!(link.code, "abc123");
        assert_eq!(link.long_url, "https://example.com");
        assert!(link.domain.is_none());
        assert_eq!(link.created_at, now);
        assert!(!link.is_deleted());
        assert!(!link.is_expired());
    }

    #[test]
    fn test_link_with_domain() {
        let link = Link::new(
            5,
            "test".to_string(),
            "https://example.com".to_string(),
            Some("s.example.com".to_string()),
            Utc::now(),
            None,
            false,
            None,
        );

        assert_eq!(link.code, "test");
        assert!(link.domain.is_some());
        assert_eq!(link.domain.unwrap(), "s.example.com");
    }

    #[test]
    fn test_link_is_deleted() {
        let link = Link::new(
            1,
            "code".to_string(),
            "https://example.com".to_string(),
            None,
            Utc::now(),
            None,
            false,
            Some(Utc::now()),
        );
        assert!(link.is_deleted());
    }

    #[test]
    fn test_link_is_expired() {
        use chrono::Duration;
        let link = Link::new(
            1,
            "code".to_string(),
            "https://example.com".to_string(),
            None,
            Utc::now(),
            Some(Utc::now() - Duration::seconds(1)),
            false,
            None,
        );
        assert!(link.is_expired());
    }

    #[test]
    fn test_new_link_creation() {
        let new_link = NewLink {
            code: "xyz789".to_string(),
            long_url: "https://rust-lang.org".to_string(),
            domain_id: 42,
            expires_at: None,
            permanent: false,
        };

        assert_eq!(new_link.code, "xyz789");
        assert_eq!(new_link.long_url, "https://rust-lang.org");
        assert_eq!(new_link.domain_id, 42);
    }
}
