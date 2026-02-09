//! Domain entity representing a URL shortening domain.

use chrono::{DateTime, Utc};

/// A domain that serves shortened URLs.
///
/// Each domain acts as a namespace for short links, allowing multiple short codes
/// with the same value across different domains. Only one domain can be marked as
/// the system default at a time.
#[derive(Debug, Clone)]
pub struct Domain {
    pub id: i64,
    pub domain: String,
    pub is_default: bool,
    pub is_active: bool,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Domain {
    /// Creates a new Domain instance.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let domain = Domain::new(
    ///     1,
    ///     "s.example.com".to_string(),
    ///     true,
    ///     true,
    ///     Some("Default shortener domain".to_string()),
    ///     Utc::now(),
    ///     Utc::now(),
    /// );
    /// ```
    pub fn new(
        id: i64,
        domain: String,
        is_default: bool,
        is_active: bool,
        description: Option<String>,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            domain,
            is_default,
            is_active,
            description,
            created_at,
            updated_at,
        }
    }
}

/// Input data for creating a new domain.
///
/// New domains are active by default. Use [`UpdateDomain`] to modify status later.
#[derive(Debug, Clone)]
pub struct NewDomain {
    pub domain: String,
    pub is_default: bool,
    pub description: Option<String>,
}

/// Input data for updating an existing domain.
///
/// All fields are optional to support partial updates. Use `None` to leave
/// a field unchanged.
#[derive(Debug, Clone, Default)]
pub struct UpdateDomain {
    pub is_active: Option<bool>,
    pub description: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_domain_creation_default() {
        let now = Utc::now();
        let domain = Domain::new(
            1,
            "s.example.com".to_string(),
            true,
            true,
            Some("Default shortener domain".to_string()),
            now,
            now,
        );

        assert_eq!(domain.id, 1);
        assert_eq!(domain.domain, "s.example.com");
        assert!(domain.is_default);
        assert!(domain.is_active);
        assert_eq!(
            domain.description,
            Some("Default shortener domain".to_string())
        );
    }

    #[test]
    fn test_domain_creation_inactive() {
        let now = Utc::now();
        let domain = Domain::new(
            2,
            "old.example.com".to_string(),
            false,
            false,
            None,
            now,
            now,
        );

        assert!(!domain.is_default);
        assert!(!domain.is_active);
        assert!(domain.description.is_none());
    }

    #[test]
    fn test_new_domain_creation() {
        let new_domain = NewDomain {
            domain: "new.short.link".to_string(),
            is_default: false,
            description: Some("Secondary domain".to_string()),
        };

        assert_eq!(new_domain.domain, "new.short.link");
        assert!(!new_domain.is_default);
        assert!(new_domain.description.is_some());
    }

    #[test]
    fn test_update_domain_partial() {
        let update = UpdateDomain {
            is_active: Some(false),
            description: None,
        };

        assert_eq!(update.is_active, Some(false));
        assert!(update.description.is_none());
    }

    #[test]
    fn test_update_domain_default() {
        let update = UpdateDomain::default();

        assert!(update.is_active.is_none());
        assert!(update.description.is_none());
    }
}
