mod common;

use sqlx::PgPool;
use std::sync::Arc;
use url_shortener::domain::entities::{NewDomain, UpdateDomain};
use url_shortener::domain::repositories::DomainRepository;
use url_shortener::infrastructure::persistence::PgDomainRepository;

#[sqlx::test]
async fn test_create_domain(pool: PgPool) {
    let repo = PgDomainRepository::new(Arc::new(pool));

    let new_domain = NewDomain {
        domain: "new-test.com".to_string(),
        is_default: false,
        description: Some("Test domain".to_string()),
    };

    let result = repo.create(new_domain).await;

    assert!(result.is_ok());
    let domain = result.unwrap();
    assert_eq!(domain.domain, "new-test.com");
    assert!(!domain.is_default);
    assert_eq!(domain.description, Some("Test domain".to_string()));
}

#[sqlx::test]
async fn test_find_by_name(pool: PgPool) {
    let repo = PgDomainRepository::new(Arc::new(pool));

    let new_domain = NewDomain {
        domain: "find-me.com".to_string(),
        is_default: false,
        description: None,
    };
    repo.create(new_domain).await.unwrap();

    let result = repo.find_by_name("find-me.com").await;

    assert!(result.is_ok());
    let domain = result.unwrap();
    assert!(domain.is_some());
    assert_eq!(domain.unwrap().domain, "find-me.com");
}

#[sqlx::test]
async fn test_get_default_domain(pool: PgPool) {
    let repo = PgDomainRepository::new(Arc::new(pool));

    let result = repo.get_default().await;

    assert!(result.is_ok());
    let domain = result.unwrap();
    assert!(domain.is_default);
}

#[sqlx::test]
async fn test_list_domains(pool: PgPool) {
    let repo = PgDomainRepository::new(Arc::new(pool));

    for i in 1..=3 {
        let new_domain = NewDomain {
            domain: format!("list-test-{}.com", i),
            is_default: false,
            description: None,
        };
        repo.create(new_domain).await.unwrap();
    }

    let result = repo.list(false).await;

    assert!(result.is_ok());
    let domains = result.unwrap();
    assert!(domains.len() >= 3);
}

#[sqlx::test]
async fn test_update_domain(pool: PgPool) {
    let repo = PgDomainRepository::new(Arc::new(pool));

    let new_domain = NewDomain {
        domain: "update-test.com".to_string(),
        is_default: false,
        description: Some("Old description".to_string()),
    };
    let created = repo.create(new_domain).await.unwrap();

    let update = UpdateDomain {
        is_active: Some(false),
        description: Some(Some("New description".to_string())),
        ..Default::default()
    };
    let result = repo.update(created.id, update).await;

    assert!(result.is_ok());
    let updated = result.unwrap();
    assert!(!updated.is_active);
    assert_eq!(updated.description, Some("New description".to_string()));
}

#[sqlx::test]
async fn test_count_links(pool: PgPool) {
    let repo = PgDomainRepository::new(Arc::new(pool.clone()));

    let new_domain = NewDomain {
        domain: "count-links.com".to_string(),
        is_default: false,
        description: None,
    };
    let domain = repo.create(new_domain).await.unwrap();

    for i in 1..=3 {
        common::create_test_link(
            &pool,
            &format!("code{}", i),
            &format!("https://example.com/{}", i),
            domain.id,
        )
        .await;
    }

    let result = repo.count_links(domain.id).await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 3);
}
