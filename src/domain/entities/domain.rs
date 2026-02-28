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
    pub deleted_at: Option<DateTime<Utc>>,
}

impl Domain {
    /// Creates a new Domain instance.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: i64,
        domain: String,
        is_default: bool,
        is_active: bool,
        description: Option<String>,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
        deleted_at: Option<DateTime<Utc>>,
    ) -> Self {
        Self {
            id,
            domain,
            is_default,
            is_active,
            description,
            created_at,
            updated_at,
            deleted_at,
        }
    }

    /// Returns true if the domain has been soft-deleted.
    pub fn is_deleted(&self) -> bool {
        self.deleted_at.is_some()
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
/// All fields are optional to support partial updates. `None` leaves a field unchanged.
///
/// # `description` semantics
///
/// - `None` → leave unchanged
/// - `Some(Some(s))` → set to `s`
/// - `Some(None)` → clear (set to NULL)
///
/// # `is_default` semantics
///
/// - `Some(true)` → make this domain the system default (handled by service via transaction)
/// - `Some(false)` → error; use `Some(true)` on another domain instead
/// - `None` → leave unchanged
#[derive(Debug, Clone, Default)]
pub struct UpdateDomain {
    pub domain: Option<String>,
    pub is_default: Option<bool>,
    pub is_active: Option<bool>,
    pub description: Option<Option<String>>,
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
            None,
        );

        assert_eq!(domain.id, 1);
        assert_eq!(domain.domain, "s.example.com");
        assert!(domain.is_default);
        assert!(domain.is_active);
        assert_eq!(
            domain.description,
            Some("Default shortener domain".to_string())
        );
        assert!(!domain.is_deleted());
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
            None,
        );

        assert!(!domain.is_default);
        assert!(!domain.is_active);
        assert!(domain.description.is_none());
    }

    #[test]
    fn test_domain_is_deleted() {
        let domain = Domain::new(
            3,
            "deleted.example.com".to_string(),
            false,
            true,
            None,
            Utc::now(),
            Utc::now(),
            Some(Utc::now()),
        );
        assert!(domain.is_deleted());
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
            ..Default::default()
        };

        assert_eq!(update.is_active, Some(false));
        assert!(update.description.is_none());
    }

    #[test]
    fn test_update_domain_default() {
        let update = UpdateDomain::default();

        assert!(update.is_active.is_none());
        assert!(update.description.is_none());
        assert!(update.domain.is_none());
        assert!(update.is_default.is_none());
    }
}
