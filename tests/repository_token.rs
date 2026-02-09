use sqlx::PgPool;
use std::sync::Arc;
use url_shortener::domain::repositories::TokenRepository;
use url_shortener::infrastructure::persistence::PgTokenRepository;

#[sqlx::test]
async fn test_create_token(pool: PgPool) {
    let repo = PgTokenRepository::new(Arc::new(pool));

    let result = repo.create_token("test-token", "hash123").await;

    assert!(result.is_ok());
    let token = result.unwrap();
    assert_eq!(token.name, "test-token");
    assert_eq!(token.token_hash, "hash123");
    assert!(token.revoked_at.is_none());
}

#[sqlx::test]
async fn test_validate_token_valid(pool: PgPool) {
    let repo = PgTokenRepository::new(Arc::new(pool));

    repo.create_token("valid-token", "validhash").await.unwrap();

    let result = repo.validate_token("validhash").await;

    assert!(result.is_ok());
    assert!(result.unwrap());
}

#[sqlx::test]
async fn test_validate_token_invalid(pool: PgPool) {
    let repo = PgTokenRepository::new(Arc::new(pool));

    let result = repo.validate_token("nonexistent").await;

    assert!(result.is_ok());
    assert!(!result.unwrap());
}

#[sqlx::test]
async fn test_validate_token_revoked(pool: PgPool) {
    let repo = PgTokenRepository::new(Arc::new(pool));

    let token = repo
        .create_token("revoked-token", "revokedhash")
        .await
        .unwrap();
    repo.revoke_token(token.id).await.unwrap();

    let result = repo.validate_token("revokedhash").await;

    assert!(result.is_ok());
    assert!(!result.unwrap());
}

#[sqlx::test]
async fn test_update_last_used(pool: PgPool) {
    let repo = PgTokenRepository::new(Arc::new(pool.clone()));

    let token = repo
        .create_token("update-token", "updatehash")
        .await
        .unwrap();

    let result = repo.update_last_used("updatehash").await;
    assert!(result.is_ok());

    let last_used = sqlx::query_scalar!(
        "SELECT last_used_at FROM api_tokens WHERE id = $1",
        token.id
    )
    .fetch_one(&pool)
    .await
    .unwrap();

    assert!(last_used.is_some());
}

#[sqlx::test]
async fn test_list_tokens(pool: PgPool) {
    let repo = PgTokenRepository::new(Arc::new(pool));

    repo.create_token("token1", "hash1").await.unwrap();
    repo.create_token("token2", "hash2").await.unwrap();
    repo.create_token("token3", "hash3").await.unwrap();

    let result = repo.list_tokens().await;

    assert!(result.is_ok());
    let tokens = result.unwrap();
    assert!(tokens.len() >= 3);
}

#[sqlx::test]
async fn test_find_by_id(pool: PgPool) {
    let repo = PgTokenRepository::new(Arc::new(pool));

    let created = repo.create_token("find-by-id", "findhash").await.unwrap();

    let result = repo.find_by_id(created.id).await;

    assert!(result.is_ok());
    let token = result.unwrap();
    assert!(token.is_some());
    assert_eq!(token.unwrap().name, "find-by-id");
}

#[sqlx::test]
async fn test_find_by_name(pool: PgPool) {
    let repo = PgTokenRepository::new(Arc::new(pool));

    repo.create_token("unique-name", "namehash").await.unwrap();

    let result = repo.find_by_name("unique-name").await;

    assert!(result.is_ok());
    let token = result.unwrap();
    assert!(token.is_some());
    assert_eq!(token.unwrap().name, "unique-name");
}

#[sqlx::test]
async fn test_revoke_token(pool: PgPool) {
    let repo = PgTokenRepository::new(Arc::new(pool.clone()));

    let token = repo
        .create_token("revoke-test", "revokehash")
        .await
        .unwrap();

    let result = repo.revoke_token(token.id).await;
    assert!(result.is_ok());

    let revoked_at =
        sqlx::query_scalar!("SELECT revoked_at FROM api_tokens WHERE id = $1", token.id)
            .fetch_one(&pool)
            .await
            .unwrap();

    assert!(revoked_at.is_some());
}

#[sqlx::test]
async fn test_revoke_already_revoked(pool: PgPool) {
    let repo = PgTokenRepository::new(Arc::new(pool));

    let token = repo
        .create_token("double-revoke", "doublehash")
        .await
        .unwrap();

    repo.revoke_token(token.id).await.unwrap();
    let result = repo.revoke_token(token.id).await;

    assert!(result.is_ok());
}
