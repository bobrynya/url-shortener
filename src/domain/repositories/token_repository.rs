//! Repository trait for API token authentication.

use crate::error::AppError;
use async_trait::async_trait;
use chrono::{DateTime, Utc};

/// API token entity with metadata.
///
/// Tokens are stored as SHA-256 hashes for security.
#[derive(Debug, Clone)]
pub struct ApiToken {
    pub id: i64,
    pub name: String,
    pub token_hash: String,
    pub created_at: DateTime<Utc>,
    pub revoked_at: Option<DateTime<Utc>>,
}

/// Repository interface for API token management.
///
/// Handles token validation, creation, and revocation for API authentication.
/// Tokens are hashed using SHA-256 before storage.
///
/// # Implementations
///
/// - [`crate::infrastructure::persistence::PgTokenRepository`] - PostgreSQL implementation
/// - Test mocks available with `cfg(test)`
///
/// # Examples
///
/// See integration tests: `tests/repository_token.rs`
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait TokenRepository: Send + Sync {
    /// Validates a token hash against stored credentials.
    ///
    /// # Returns
    ///
    /// - `Ok(true)` if the token is valid and not revoked
    /// - `Ok(false)` if the token is invalid or revoked
    ///
    /// # Errors
    ///
    /// Returns [`AppError::Internal`] on database errors.
    async fn validate_token(&self, token_hash: &str) -> Result<bool, AppError>;

    /// Updates the last_used timestamp for a token.
    ///
    /// Called after successful authentication to track token usage.
    ///
    /// # Errors
    ///
    /// Returns [`AppError::Internal`] on database errors.
    async fn update_last_used(&self, token_hash: &str) -> Result<(), AppError>;

    /// Creates a new API token.
    ///
    /// # Arguments
    ///
    /// - `name` - Human-readable token identifier
    /// - `token_hash` - SHA-256 hash of the raw token
    ///
    /// # Errors
    ///
    /// Returns [`AppError::Conflict`] if a token with the same hash already exists.
    /// Returns [`AppError::Internal`] on database errors.
    async fn create_token(&self, name: &str, token_hash: &str) -> Result<ApiToken, AppError>;

    /// Lists all tokens in the system.
    ///
    /// # Errors
    ///
    /// Returns [`AppError::Internal`] on database errors.
    async fn list_tokens(&self) -> Result<Vec<ApiToken>, AppError>;

    /// Finds a token by its database ID.
    ///
    /// # Errors
    ///
    /// Returns [`AppError::Internal`] on database errors.
    async fn find_by_id(&self, id: i64) -> Result<Option<ApiToken>, AppError>;

    /// Finds a token by its name.
    ///
    /// # Errors
    ///
    /// Returns [`AppError::Internal`] on database errors.
    async fn find_by_name(&self, name: &str) -> Result<Option<ApiToken>, AppError>;

    /// Revokes a token, preventing further authentication.
    ///
    /// Sets the `revoked_at` timestamp to the current time.
    ///
    /// # Errors
    ///
    /// Returns [`AppError::NotFound`] if the token does not exist.
    /// Returns [`AppError::Internal`] on database errors.
    async fn revoke_token(&self, id: i64) -> Result<(), AppError>;
}
