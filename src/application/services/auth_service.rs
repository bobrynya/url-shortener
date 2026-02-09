//! Authentication service for API token validation.

use sha2::{Digest, Sha256};
use std::sync::Arc;

use crate::domain::repositories::TokenRepository;
use crate::error::AppError;
use serde_json::json;

/// Service for authenticating API requests via Bearer tokens.
///
/// Tokens are hashed using SHA-256 before storage and comparison to prevent
/// leakage of raw credentials.
pub struct AuthService<R: TokenRepository> {
    repository: Arc<R>,
}

impl<R: TokenRepository> AuthService<R> {
    /// Creates a new authentication service.
    pub fn new(repository: Arc<R>) -> Self {
        Self { repository }
    }

    /// Hashes a raw token using SHA-256.
    ///
    /// Returns a 64-character hex-encoded hash.
    fn hash_token(&self, token: &str) -> String {
        let mut h = Sha256::new();
        h.update(token.as_bytes());
        hex::encode(h.finalize())
    }

    /// Authenticates a raw token against stored credentials.
    ///
    /// On successful authentication, updates the `last_used` timestamp for
    /// monitoring and audit purposes.
    ///
    /// # Errors
    ///
    /// Returns [`AppError::Unauthorized`] if:
    /// - Token hash does not match any stored credentials
    /// - Token has been revoked
    ///
    /// Returns [`AppError::Internal`] on database errors.
    pub async fn authenticate(&self, token: &str) -> Result<(), AppError> {
        let token_hash = self.hash_token(token);

        let is_valid = self.repository.validate_token(&token_hash).await?;

        if !is_valid {
            return Err(AppError::unauthorized(
                "Unauthorized",
                json!({"reason": "Invalid or revoked token"}),
            ));
        }

        let _ = self.repository.update_last_used(&token_hash).await;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::repositories::MockTokenRepository;

    #[tokio::test]
    async fn test_authenticate_success() {
        let mut mock_repo = MockTokenRepository::new();

        let token = "valid-token";
        let expected_hash = {
            let mut h = Sha256::new();
            h.update(token.as_bytes());
            hex::encode(h.finalize())
        };

        mock_repo
            .expect_validate_token()
            .withf(move |hash| hash == &expected_hash)
            .times(1)
            .returning(|_| Ok(true));

        mock_repo
            .expect_update_last_used()
            .times(1)
            .returning(|_| Ok(()));

        let service = AuthService::new(Arc::new(mock_repo));

        let result = service.authenticate(token).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_authenticate_invalid_token() {
        let mut mock_repo = MockTokenRepository::new();

        mock_repo
            .expect_validate_token()
            .times(1)
            .returning(|_| Ok(false));

        let service = AuthService::new(Arc::new(mock_repo));

        let result = service.authenticate("invalid-token").await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AppError::Unauthorized { .. }));
    }

    #[tokio::test]
    async fn test_hash_token_consistency() {
        let mock_repo = MockTokenRepository::new();
        let service = AuthService::new(Arc::new(mock_repo));

        let hash1 = service.hash_token("test-token");
        let hash2 = service.hash_token("test-token");

        assert_eq!(hash1, hash2);
        assert_eq!(hash1.len(), 64);
    }

    #[tokio::test]
    async fn test_hash_token_different_inputs() {
        let mock_repo = MockTokenRepository::new();
        let service = AuthService::new(Arc::new(mock_repo));

        let hash1 = service.hash_token("token1");
        let hash2 = service.hash_token("token2");

        assert_ne!(hash1, hash2);
    }
}
