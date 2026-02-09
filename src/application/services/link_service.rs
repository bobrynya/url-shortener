//! Link creation and retrieval service.

use std::sync::Arc;

use crate::domain::entities::{Link, NewLink};
use crate::domain::repositories::{DomainRepository, LinkRepository};
use crate::error::AppError;
use crate::utils::code_generator::{generate_code, validate_custom_code};
use crate::utils::url_normalizer::normalize_url;
use serde_json::json;

/// Service for creating and retrieving shortened links.
///
/// Handles URL normalization, code generation/validation, and deduplication
/// to ensure consistent and collision-free short URLs.
pub struct LinkService<L: LinkRepository, D: DomainRepository> {
    link_repository: Arc<L>,
    domain_repository: Arc<D>,
}

impl<L: LinkRepository, D: DomainRepository> LinkService<L, D> {
    /// Creates a new link service.
    pub fn new(link_repository: Arc<L>, domain_repository: Arc<D>) -> Self {
        Self {
            link_repository,
            domain_repository,
        }
    }

    /// Creates a short link using the default domain.
    ///
    /// # Arguments
    ///
    /// - `long_url` - The original URL to shorten
    /// - `custom_code` - Optional custom short code (validated if provided)
    ///
    /// # Errors
    ///
    /// Returns [`AppError::Validation`] if:
    /// - URL is invalid
    /// - Custom code is invalid
    /// - Domain doesn't exist
    ///
    /// Returns [`AppError::Conflict`] if custom code already exists.
    pub async fn create_short_link(
        &self,
        long_url: String,
        custom_code: Option<String>,
    ) -> Result<Link, AppError> {
        let default_domain = self.domain_repository.get_default().await?;
        self.create_short_link_for_domain(long_url, default_domain.id, custom_code)
            .await
    }

    /// Creates a short link for a specific domain.
    ///
    /// # Deduplication
    ///
    /// If a link for the same normalized URL and domain already exists, returns
    /// the existing link instead of creating a duplicate.
    ///
    /// # Code Generation
    ///
    /// - If `custom_code` is provided, validates and uses it (or returns conflict error)
    /// - Otherwise, generates a cryptographically secure random 12-character code
    /// - Retries up to 10 times on collision before failing
    ///
    /// # Errors
    ///
    /// See [`Self::create_short_link`] for error cases.
    pub async fn create_short_link_for_domain(
        &self,
        long_url: String,
        domain_id: i64,
        custom_code: Option<String>,
    ) -> Result<Link, AppError> {
        let normalized_url = normalize_url(&long_url).map_err(|e| {
            AppError::bad_request("Invalid URL format", json!({ "reason": e.to_string() }))
        })?;

        if let Some(existing_link) = self
            .link_repository
            .find_by_long_url(&normalized_url, domain_id)
            .await?
        {
            return Ok(existing_link);
        }

        let code = if let Some(custom) = custom_code {
            validate_custom_code(&custom)?;

            if self
                .link_repository
                .find_by_code(&custom, domain_id)
                .await?
                .is_some()
            {
                return Err(AppError::conflict(
                    "Custom code already exists for this domain",
                    json!({ "code": custom, "domain_id": domain_id }),
                ));
            }

            custom
        } else {
            self.generate_unique_code(domain_id).await?
        };

        let new_link = NewLink {
            code,
            long_url: normalized_url,
            domain_id,
        };

        self.link_repository.create(new_link).await
    }

    /// Retrieves a link by its short code and domain.
    ///
    /// # Errors
    ///
    /// Returns [`AppError::NotFound`] if no link matches the code and domain.
    /// Returns [`AppError::Internal`] on database errors.
    pub async fn get_link_by_code(&self, code: &str, domain_id: i64) -> Result<Link, AppError> {
        self.link_repository
            .find_by_code(code, domain_id)
            .await?
            .ok_or_else(|| {
                AppError::not_found(
                    "Short link not found",
                    json!({ "code": code, "domain_id": domain_id }),
                )
            })
    }

    /// Constructs the full short URL from a domain and code.
    ///
    /// Always uses HTTPS protocol.
    pub fn get_short_url(&self, domain: &str, code: &str) -> String {
        format!("https://{}/{}", domain.trim_end_matches('/'), code)
    }

    /// Generates a unique short code for a domain with collision retry.
    ///
    /// Attempts up to 10 times before failing.
    async fn generate_unique_code(&self, domain_id: i64) -> Result<String, AppError> {
        const MAX_ATTEMPTS: usize = 10;

        for _ in 0..MAX_ATTEMPTS {
            let code = generate_code();

            if self
                .link_repository
                .find_by_code(&code, domain_id)
                .await?
                .is_none()
            {
                return Ok(code);
            }
        }

        Err(AppError::internal(
            "Failed to generate unique code",
            json!({ "reason": "Too many collisions" }),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entities::Domain;
    use crate::domain::repositories::{MockDomainRepository, MockLinkRepository};
    use chrono::Utc;

    fn create_test_domain(id: i64) -> Domain {
        Domain::new(
            id,
            "s.example.com".to_string(),
            true,
            true,
            None,
            Utc::now(),
            Utc::now(),
        )
    }

    fn create_test_link(id: i64, code: &str, url: &str, _domain_id: i64) -> Link {
        Link::new(
            id,
            code.to_string(),
            url.to_string(),
            Some("s.example.com".to_string()),
            Utc::now(),
        )
    }

    #[tokio::test]
    async fn test_create_short_link_success() {
        let mut mock_link_repo = MockLinkRepository::new();
        let mut mock_domain_repo = MockDomainRepository::new();

        let domain = create_test_domain(1);
        mock_domain_repo
            .expect_get_default()
            .times(1)
            .returning(move || Ok(domain.clone()));

        mock_link_repo
            .expect_find_by_long_url()
            .times(1)
            .returning(|_, _| Ok(None));

        mock_link_repo
            .expect_find_by_code()
            .times(1)
            .returning(|_, _| Ok(None));

        let created_link = create_test_link(10, "abc123", "https://example.com", 1);
        mock_link_repo
            .expect_create()
            .times(1)
            .returning(move |_| Ok(created_link.clone()));

        let service = LinkService::new(Arc::new(mock_link_repo), Arc::new(mock_domain_repo));

        let result = service
            .create_short_link("https://example.com".to_string(), None)
            .await;

        assert!(result.is_ok());
        let link = result.unwrap();
        assert_eq!(link.long_url, "https://example.com");
    }

    #[tokio::test]
    async fn test_create_short_link_normalizes_url() {
        let mut mock_link_repo = MockLinkRepository::new();
        let mut mock_domain_repo = MockDomainRepository::new();

        let domain = create_test_domain(1);
        mock_domain_repo
            .expect_get_default()
            .times(1)
            .returning(move || Ok(domain.clone()));

        mock_link_repo
            .expect_find_by_long_url()
            .withf(|url, _| url == "https://example.com/path")
            .times(1)
            .returning(|_, _| Ok(None));

        mock_link_repo
            .expect_find_by_code()
            .times(1)
            .returning(|_, _| Ok(None));

        let created_link = create_test_link(10, "abc123", "https://example.com/path", 1);
        mock_link_repo
            .expect_create()
            .times(1)
            .returning(move |_| Ok(created_link.clone()));

        let service = LinkService::new(Arc::new(mock_link_repo), Arc::new(mock_domain_repo));

        let result = service
            .create_short_link("https://EXAMPLE.COM:443/path".to_string(), None)
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_create_short_link_deduplication() {
        let mut mock_link_repo = MockLinkRepository::new();
        let mut mock_domain_repo = MockDomainRepository::new();

        let domain = create_test_domain(1);
        mock_domain_repo
            .expect_get_default()
            .times(1)
            .returning(move || Ok(domain.clone()));

        let existing_link = create_test_link(5, "existing", "https://example.com", 1);
        mock_link_repo
            .expect_find_by_long_url()
            .times(1)
            .returning(move |_, _| Ok(Some(existing_link.clone())));

        mock_link_repo.expect_create().times(0);

        let service = LinkService::new(Arc::new(mock_link_repo), Arc::new(mock_domain_repo));

        let result = service
            .create_short_link("https://example.com".to_string(), None)
            .await;

        assert!(result.is_ok());
        let link = result.unwrap();
        assert_eq!(link.id, 5);
        assert_eq!(link.code, "existing");
    }

    #[tokio::test]
    async fn test_create_short_link_invalid_url() {
        let mock_link_repo = MockLinkRepository::new();
        let mock_domain_repo = MockDomainRepository::new();

        let service = LinkService::new(Arc::new(mock_link_repo), Arc::new(mock_domain_repo));

        let result = service
            .create_short_link_for_domain("not-a-url".to_string(), 1, None)
            .await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AppError::Validation { .. }));
    }

    #[tokio::test]
    async fn test_create_short_link_with_custom_code() {
        let mut mock_link_repo = MockLinkRepository::new();
        let mut mock_domain_repo = MockDomainRepository::new();

        let domain = create_test_domain(1);
        mock_domain_repo
            .expect_get_default()
            .times(1)
            .returning(move || Ok(domain.clone()));

        mock_link_repo
            .expect_find_by_long_url()
            .times(1)
            .returning(|_, _| Ok(None));

        mock_link_repo
            .expect_find_by_code()
            .withf(|code, _| code == "mycode12")
            .times(1)
            .returning(|_, _| Ok(None));

        let created_link = create_test_link(10, "mycode12", "https://example.com", 1);
        mock_link_repo
            .expect_create()
            .withf(|new_link| new_link.code == "mycode12")
            .times(1)
            .returning(move |_| Ok(created_link.clone()));

        let service = LinkService::new(Arc::new(mock_link_repo), Arc::new(mock_domain_repo));

        let result = service
            .create_short_link(
                "https://example.com".to_string(),
                Some("mycode12".to_string()),
            )
            .await;

        assert!(result.is_ok());
        let link = result.unwrap();
        assert_eq!(link.code, "mycode12");
    }

    #[tokio::test]
    async fn test_create_short_link_custom_code_conflict() {
        let mut mock_link_repo = MockLinkRepository::new();
        let mut mock_domain_repo = MockDomainRepository::new();

        let domain = create_test_domain(1);
        mock_domain_repo
            .expect_get_default()
            .times(1)
            .returning(move || Ok(domain.clone()));

        mock_link_repo
            .expect_find_by_long_url()
            .times(1)
            .returning(|_, _| Ok(None));

        let existing_link = create_test_link(5, "taken123", "https://other.com", 1);
        mock_link_repo
            .expect_find_by_code()
            .withf(|code, _| code == "taken123")
            .times(1)
            .returning(move |_, _| Ok(Some(existing_link.clone())));

        let service = LinkService::new(Arc::new(mock_link_repo), Arc::new(mock_domain_repo));

        let result = service
            .create_short_link(
                "https://example.com".to_string(),
                Some("taken123".to_string()),
            )
            .await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AppError::Conflict { .. }));
    }
}
