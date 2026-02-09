mod common;

use axum::{Router, routing::get};
use axum_test::TestServer;
use sqlx::PgPool;
use url_shortener::api::handlers::health_handler;

#[sqlx::test]
async fn test_health_endpoint_success(pool: PgPool) {
    let (state, _rx) = common::create_test_state(pool);
    let app = Router::new()
        .route("/api/health", get(health_handler))
        .with_state(state);

    let server = TestServer::new(app).unwrap();

    let response = server.get("/api/health").await;

    response.assert_status_ok();

    let json = response.json::<serde_json::Value>();
    assert_eq!(json["status"], "healthy");
    assert_eq!(json["checks"]["database"]["status"], "ok");
    assert_eq!(json["checks"]["click_queue"]["status"], "ok");
}

#[sqlx::test]
async fn test_health_endpoint_structure(pool: PgPool) {
    let (state, _rx) = common::create_test_state(pool);
    let app = Router::new()
        .route("/api/health", get(health_handler))
        .with_state(state);

    let server = TestServer::new(app).unwrap();

    let response = server.get("/api/health").await;

    let json = response.json::<serde_json::Value>();

    assert!(json.get("status").is_some());
    assert!(json.get("version").is_some());
    assert!(json.get("checks").is_some());
    assert!(json["checks"].get("database").is_some());
    assert!(json["checks"].get("click_queue").is_some());
    assert!(json["checks"].get("cache").is_some());
}
