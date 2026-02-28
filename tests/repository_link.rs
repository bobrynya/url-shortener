mod common;

use sqlx::PgPool;
use std::sync::Arc;
use url_shortener::domain::entities::NewLink;
use url_shortener::domain::repositories::LinkRepository;
use url_shortener::infrastructure::persistence::PgLinkRepository;

#[sqlx::test]
async fn test_create_link(pool: PgPool) {
    let domain_id = common::create_test_domain(&pool, "test1.com").await;
    let repo = PgLinkRepository::new(Arc::new(pool));

    let new_link = NewLink {
        code: "test123".to_string(),
        long_url: "https://example.com".to_string(),
        domain_id,
        expires_at: None,
        permanent: false,
    };

    let result = repo.create(new_link).await;

    assert!(result.is_ok());
    let link = result.unwrap();
    assert_eq!(link.code, "test123");
    assert_eq!(link.long_url, "https://example.com");
}

#[sqlx::test]
async fn test_find_by_code(pool: PgPool) {
    let domain_id = common::create_test_domain(&pool, "test2.com").await;

    sqlx::query!(
        "INSERT INTO links (code, long_url, domain_id) VALUES ($1, $2, $3)",
        "abc123",
        "https://example.com",
        domain_id
    )
    .execute(&pool)
    .await
    .unwrap();

    let repo = PgLinkRepository::new(Arc::new(pool));
    let result = repo.find_by_code("abc123", domain_id).await;

    assert!(result.is_ok());
    let link = result.unwrap();
    assert!(link.is_some());
    assert_eq!(link.unwrap().code, "abc123");
}

#[sqlx::test]
async fn test_find_by_code_not_found(pool: PgPool) {
    let domain_id = common::create_test_domain(&pool, "test3.com").await;
    let repo = PgLinkRepository::new(Arc::new(pool));

    let result = repo.find_by_code("notfound", domain_id).await;

    assert!(result.is_ok());
    assert!(result.unwrap().is_none());
}

#[sqlx::test]
async fn test_find_by_long_url(pool: PgPool) {
    let domain_id = common::create_test_domain(&pool, "test4.com").await;

    sqlx::query!(
        "INSERT INTO links (code, long_url, domain_id) VALUES ($1, $2, $3)",
        "xyz789",
        "https://unique-url.com",
        domain_id
    )
    .execute(&pool)
    .await
    .unwrap();

    let repo = PgLinkRepository::new(Arc::new(pool));
    let result = repo
        .find_by_long_url("https://unique-url.com", domain_id)
        .await;

    assert!(result.is_ok());
    let link = result.unwrap();
    assert!(link.is_some());
    assert_eq!(link.unwrap().code, "xyz789");
}
