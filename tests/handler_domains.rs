mod common;

use axum::{Router, routing::get};
use axum_test::TestServer;
use sqlx::PgPool;
use url_shortener::api::handlers::domain_list_handler;

#[sqlx::test]
async fn test_domains_list_success(pool: PgPool) {
    let (state, _rx) = common::create_test_state(pool.clone());
    let app = Router::new()
        .route("/api/domains", get(domain_list_handler))
        .with_state(state);

    let server = TestServer::new(app).unwrap();

    common::create_test_domain(&pool, "test1.com").await;
    common::create_test_domain(&pool, "test2.com").await;

    let response = server.get("/api/domains").await;

    response.assert_status_ok();

    let json = response.json::<serde_json::Value>();
    let items = json["items"].as_array().unwrap();

    assert!(items.len() >= 3);

    assert!(items[0].get("domain").is_some());
    assert!(items[0].get("is_default").is_some());
    assert!(items[0].get("is_active").is_some());
}

#[sqlx::test]
async fn test_domains_list_has_default(pool: PgPool) {
    let (state, _rx) = common::create_test_state(pool);
    let app = Router::new()
        .route("/api/domains", get(domain_list_handler))
        .with_state(state);

    let server = TestServer::new(app).unwrap();

    let response = server.get("/api/domains").await;

    response.assert_status_ok();

    let json = response.json::<serde_json::Value>();
    let items = json["items"].as_array().unwrap();

    let has_default = items.iter().any(|item| item["is_default"] == true);
    assert!(has_default);
}

#[sqlx::test]
async fn test_domains_list_structure(pool: PgPool) {
    let (state, _rx) = common::create_test_state(pool);
    let app = Router::new()
        .route("/api/domains", get(domain_list_handler))
        .with_state(state);

    let server = TestServer::new(app).unwrap();

    let response = server.get("/api/domains").await;

    response.assert_status_ok();

    let json = response.json::<serde_json::Value>();

    assert!(json.get("items").is_some());
    assert!(json["items"].is_array());

    let items = json["items"].as_array().unwrap();
    if !items.is_empty() {
        let first = &items[0];
        assert!(first.get("domain").is_some());
        assert!(first.get("is_default").is_some());
        assert!(first.get("is_active").is_some());
        assert!(first.get("created_at").is_some());
        assert!(first.get("updated_at").is_some());
    }
}
