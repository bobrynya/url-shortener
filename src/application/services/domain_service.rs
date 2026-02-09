//! Domain management service.

use crate::domain::entities::{Domain, NewDomain, UpdateDomain};
use crate::domain::repositories::DomainRepository;
use crate::error::AppError;
use serde_json::json;
use std::sync::Arc;

/// Service for managing domains that serve shortened URLs.
///
/// Handles domain CRUD operations with validation to ensure:
/// - Valid DNS-compatible domain names
/// - Proper default domain management
/// - Safe deletion (prevents cascading issues)
pub struct DomainService<R: DomainRepository> {
    repository: Arc<R>,
}

impl<R: DomainRepository> DomainService<R> {
    /// Creates a new domain service.
    pub fn new(repository: Arc<R>) -> Self {
        Self { repository }
    }

    /// Creates a new domain.
    ///
    /// # Arguments
    ///
    /// - `domain` - Domain name (e.g., "s.example.com")
    /// - `is_default` - Whether to set as system default
    /// - `description` - Optional human-readable description
    ///
    /// # Validation
    ///
    /// - Must contain at least one dot
    /// - Length: 1-255 characters
    /// - Allowed characters: alphanumeric, dots, hyphens
    ///
    /// # Errors
    ///
    /// Returns [`AppError::Validation`] if validation fails.
    /// Returns [`AppError::Conflict`] if domain already exists.
    /// Returns [`AppError::Internal`] on database errors.
    pub async fn create_domain(
        &self,
        domain: String,
        is_default: bool,
        description: Option<String>,
    ) -> Result<Domain, AppError> {
        self.validate_domain_name(&domain)?;

        if self.repository.find_by_name(&domain).await?.is_some() {
            return Err(AppError::conflict(
                "Domain already exists",
                json!({"domain": domain}),
            ));
        }

        let new_domain = NewDomain {
            domain,
            is_default,
            description,
        };

        let created = self.repository.create(new_domain).await?;

        if is_default {
            self.repository.set_default(created.id).await?;
        }

        Ok(created)
    }

    /// Lists all domains, optionally filtered by active status.
    ///
    /// # Errors
    ///
    /// Returns [`AppError::Internal`] on database errors.
    pub async fn list_domains(&self, only_active: bool) -> Result<Vec<Domain>, AppError> {
        self.repository.list(only_active).await
    }

    /// Retrieves a domain by name.
    ///
    /// # Errors
    ///
    /// Returns [`AppError::NotFound`] if the domain does not exist.
    /// Returns [`AppError::Internal`] on database errors.
    pub async fn get_domain(&self, domain: &str) -> Result<Domain, AppError> {
        self.repository
            .find_by_name(domain)
            .await?
            .ok_or_else(|| AppError::not_found("Domain not found", json!({"domain": domain})))
    }

    /// Retrieves the system default domain.
    ///
    /// # Errors
    ///
    /// Returns [`AppError::NotFound`] if no default domain is configured.
    /// Returns [`AppError::Internal`] on database errors.
    pub async fn get_default_domain(&self) -> Result<Domain, AppError> {
        self.repository.get_default().await
    }

    /// Sets a domain as the system default.
    ///
    /// Only one domain can be default at a time.
    ///
    /// # Errors
    ///
    /// Returns [`AppError::NotFound`] if the domain does not exist.
    /// Returns [`AppError::Internal`] on database errors.
    pub async fn set_default(&self, domain_id: i64) -> Result<(), AppError> {
        self.repository.set_default(domain_id).await
    }

    /// Updates an existing domain.
    ///
    /// # Errors
    ///
    /// Returns [`AppError::NotFound`] if the domain does not exist.
    /// Returns [`AppError::Internal`] on database errors.
    pub async fn update_domain(
        &self,
        domain_id: i64,
        update: UpdateDomain,
    ) -> Result<Domain, AppError> {
        self.repository.update(domain_id, update).await
    }

    /// Deletes a domain with safety checks.
    ///
    /// # Safety Checks
    ///
    /// - Cannot delete the default domain (set another as default first)
    /// - Cannot delete domains with existing links
    ///
    /// # Errors
    ///
    /// Returns [`AppError::NotFound`] if the domain does not exist.
    /// Returns [`AppError::Validation`] if safety checks fail.
    /// Returns [`AppError::Internal`] on database errors.
    pub async fn delete_domain(&self, domain_id: i64) -> Result<(), AppError> {
        let domain = self
            .repository
            .find_by_id(domain_id)
            .await?
            .ok_or_else(|| AppError::not_found("Domain not found", json!({"id": domain_id})))?;

        if domain.is_default {
            return Err(AppError::bad_request(
                "Cannot delete default domain",
                json!({"hint": "Set another domain as default first"}),
            ));
        }

        let links_count = self.repository.count_links(domain_id).await?;
        if links_count > 0 {
            return Err(AppError::bad_request(
                "Cannot delete domain with existing links",
                json!({"links_count": links_count}),
            ));
        }

        self.repository.delete(domain_id).await
    }

    /// Validates domain name format.
    ///
    /// # Errors
    ///
    /// Returns [`AppError::ValidationError`] if validation fails.
    fn validate_domain_name(&self, domain: &str) -> Result<(), AppError> {
        if domain.is_empty() || domain.len() > 255 {
            return Err(AppError::bad_request(
                "Invalid domain name length",
                json!({"min": 1, "max": 255}),
            ));
        }

        if !domain.contains('.') {
            return Err(AppError::bad_request(
                "Invalid domain format",
                json!({"hint": "Domain must contain at least one dot"}),
            ));
        }

        if !domain
            .chars()
            .all(|c| c.is_alphanumeric() || c == '.' || c == '-')
        {
            return Err(AppError::bad_request(
                "Invalid characters in domain name",
                json!({"allowed": "a-z, 0-9, dots, hyphens"}),
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::repositories::MockDomainRepository;
    use chrono::Utc;

    fn create_test_domain(id: i64, name: &str, is_default: bool) -> Domain {
        Domain::new(
            id,
            name.to_string(),
            is_default,
            true,
            None,
            Utc::now(),
            Utc::now(),
        )
    }

    #[tokio::test]
    async fn test_create_domain_success() {
        let mut mock_repo = MockDomainRepository::new();

        mock_repo
            .expect_find_by_name()
            .withf(|name| name == "new.example.com")
            .times(1)
            .returning(|_| Ok(None));

        let created_domain = create_test_domain(1, "new.example.com", false);
        mock_repo
            .expect_create()
            .times(1)
            .returning(move |_| Ok(created_domain.clone()));

        let service = DomainService::new(Arc::new(mock_repo));

        let result = service
            .create_domain("new.example.com".to_string(), false, None)
            .await;

        assert!(result.is_ok());
        let domain = result.unwrap();
        assert_eq!(domain.domain, "new.example.com");
    }

    #[tokio::test]
    async fn test_create_domain_already_exists() {
        let mut mock_repo = MockDomainRepository::new();

        let existing = create_test_domain(1, "existing.com", false);
        mock_repo
            .expect_find_by_name()
            .times(1)
            .returning(move |_| Ok(Some(existing.clone())));

        let service = DomainService::new(Arc::new(mock_repo));

        let result = service
            .create_domain("existing.com".to_string(), false, None)
            .await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AppError::Conflict { .. }));
    }

    #[tokio::test]
    async fn test_create_domain_invalid_empty() {
        let mock_repo = MockDomainRepository::new();
        let service = DomainService::new(Arc::new(mock_repo));

        let result = service.create_domain("".to_string(), false, None).await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AppError::Validation { .. }));
    }

    #[tokio::test]
    async fn test_create_domain_invalid_no_dot() {
        let mock_repo = MockDomainRepository::new();
        let service = DomainService::new(Arc::new(mock_repo));

        let result = service
            .create_domain("localhost".to_string(), false, None)
            .await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AppError::Validation { .. }));
    }

    #[tokio::test]
    async fn test_create_domain_invalid_characters() {
        let mock_repo = MockDomainRepository::new();
        let service = DomainService::new(Arc::new(mock_repo));

        let result = service
            .create_domain("bad_domain!.com".to_string(), false, None)
            .await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AppError::Validation { .. }));
    }

    #[tokio::test]
    async fn test_get_domain_success() {
        let mut mock_repo = MockDomainRepository::new();

        let domain = create_test_domain(1, "test.com", false);
        mock_repo
            .expect_find_by_name()
            .withf(|name| name == "test.com")
            .times(1)
            .returning(move |_| Ok(Some(domain.clone())));

        let service = DomainService::new(Arc::new(mock_repo));

        let result = service.get_domain("test.com").await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap().domain, "test.com");
    }

    #[tokio::test]
    async fn test_get_domain_not_found() {
        let mut mock_repo = MockDomainRepository::new();

        mock_repo
            .expect_find_by_name()
            .times(1)
            .returning(|_| Ok(None));

        let service = DomainService::new(Arc::new(mock_repo));

        let result = service.get_domain("notfound.com").await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AppError::NotFound { .. }));
    }

    #[tokio::test]
    async fn test_delete_domain_with_links() {
        let mut mock_repo = MockDomainRepository::new();

        let domain = create_test_domain(1, "test.com", false);
        mock_repo
            .expect_find_by_id()
            .times(1)
            .returning(move |_| Ok(Some(domain.clone())));

        mock_repo.expect_count_links().times(1).returning(|_| Ok(5));

        let service = DomainService::new(Arc::new(mock_repo));

        let result = service.delete_domain(1).await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AppError::Validation { .. }));
    }

    #[tokio::test]
    async fn test_delete_default_domain() {
        let mut mock_repo = MockDomainRepository::new();

        let domain = create_test_domain(1, "default.com", true);
        mock_repo
            .expect_find_by_id()
            .times(1)
            .returning(move |_| Ok(Some(domain.clone())));

        let service = DomainService::new(Arc::new(mock_repo));

        let result = service.delete_domain(1).await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AppError::Validation { .. }));
    }

    #[tokio::test]
    async fn test_list_domains() {
        let mut mock_repo = MockDomainRepository::new();

        let domains = vec![
            create_test_domain(1, "default.com", true),
            create_test_domain(2, "secondary.com", false),
        ];

        mock_repo
            .expect_list()
            .times(1)
            .returning(move |_| Ok(domains.clone()));

        let service = DomainService::new(Arc::new(mock_repo));

        let result = service.list_domains(true).await;

        assert!(result.is_ok());
        let list = result.unwrap();
        assert_eq!(list.len(), 2);
    }
}
