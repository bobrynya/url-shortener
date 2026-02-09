//! Repository trait for domain management.

use crate::domain::entities::{Domain, NewDomain, UpdateDomain};
use crate::error::AppError;
use async_trait::async_trait;

/// Repository interface for managing domains.
///
/// Handles CRUD operations for domains that serve as namespaces for short links.
/// Each domain can have one default domain for the system.
///
/// # Implementations
///
/// - [`crate::infrastructure::persistence::PgDomainRepository`] - PostgreSQL implementation
/// - Test mocks available with `cfg(test)`
///
/// # Examples
///
/// See integration tests: `tests/repository_domain.rs`
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait DomainRepository: Send + Sync {
    /// Creates a new domain.
    ///
    /// # Errors
    ///
    /// Returns [`AppError::Conflict`] if a domain with the same name already exists.
    /// Returns [`AppError::Internal`] on database errors.
    async fn create(&self, new_domain: NewDomain) -> Result<Domain, AppError>;

    /// Finds a domain by its database ID.
    ///
    /// # Errors
    ///
    /// Returns [`AppError::Internal`] on database errors.
    async fn find_by_id(&self, id: i64) -> Result<Option<Domain>, AppError>;

    /// Finds a domain by its name (e.g., "s.example.com").
    ///
    /// # Errors
    ///
    /// Returns [`AppError::Internal`] on database errors.
    async fn find_by_name(&self, domain: &str) -> Result<Option<Domain>, AppError>;

    /// Retrieves the default domain for the system.
    ///
    /// # Errors
    ///
    /// Returns [`AppError::NotFound`] if no default domain is configured.
    /// Returns [`AppError::Internal`] on database errors.
    async fn get_default(&self) -> Result<Domain, AppError>;

    /// Lists all domains, optionally filtered by active status.
    ///
    /// # Arguments
    ///
    /// - `only_active` - If true, returns only active domains
    ///
    /// # Errors
    ///
    /// Returns [`AppError::Internal`] on database errors.
    async fn list(&self, only_active: bool) -> Result<Vec<Domain>, AppError>;

    /// Updates an existing domain.
    ///
    /// # Errors
    ///
    /// Returns [`AppError::NotFound`] if the domain does not exist.
    /// Returns [`AppError::Internal`] on database errors.
    async fn update(&self, id: i64, update: UpdateDomain) -> Result<Domain, AppError>;

    /// Deletes a domain.
    ///
    /// # Errors
    ///
    /// Returns [`AppError::NotFound`] if the domain does not exist.
    /// Returns [`AppError::Conflict`] if the domain has associated links.
    /// Returns [`AppError::Internal`] on database errors.
    async fn delete(&self, id: i64) -> Result<(), AppError>;

    /// Sets a domain as the system default.
    ///
    /// Only one domain can be marked as default at a time.
    ///
    /// # Errors
    ///
    /// Returns [`AppError::NotFound`] if the domain does not exist.
    /// Returns [`AppError::Internal`] on database errors.
    async fn set_default(&self, id: i64) -> Result<(), AppError>;

    /// Counts the number of links associated with a domain.
    ///
    /// # Errors
    ///
    /// Returns [`AppError::Internal`] on database errors.
    async fn count_links(&self, domain_id: i64) -> Result<i64, AppError>;
}
