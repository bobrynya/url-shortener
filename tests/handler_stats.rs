mod common;

use axum::{Router, routing::get};
use axum_test::TestServer;
use sqlx::PgPool;
use url_shortener::api::handlers::{stats_handler, stats_list_handler};

#[sqlx::test]
async fn test_stats_by_code_success(pool: PgPool) {
    let (state, _rx) = common::create_test_state(pool.clone());
    let app = Router::new()
        .route("/api/stats/{code}", get(stats_handler))
        .with_state(state);

    let server = TestServer::new(app).unwrap();

    let domain_id = common::create_test_domain(&pool, "stats-test.com").await;
    common::create_test_link(&pool, "testcode", "https://example.com", domain_id).await;

    let link_id: i64 = sqlx::query_scalar!("SELECT id FROM links WHERE code = 'testcode'")
        .fetch_one(&pool)
        .await
        .unwrap();

    for i in 1..=5 {
        common::create_test_click(&pool, link_id, &format!("192.168.1.{}", i)).await;
    }

    let response = server.get("/api/stats/testcode").await;

    response.assert_status_ok();

    let json = response.json::<serde_json::Value>();
    assert_eq!(json["code"], "testcode");
    assert_eq!(json["long_url"], "https://example.com");
    assert_eq!(json["total"], 5);
    assert!(json["items"].as_array().unwrap().len() <= 5);
}

#[sqlx::test]
async fn test_stats_by_code_not_found(pool: PgPool) {
    let (state, _rx) = common::create_test_state(pool);
    let app = Router::new()
        .route("/api/stats/{code}", get(stats_handler))
        .with_state(state);

    let server = TestServer::new(app).unwrap();
    let response = server.get("/api/stats/notfound").await;

    response.assert_status_not_found();

    let json = response.json::<serde_json::Value>();
    assert_eq!(json["error"]["code"], "not_found");
}

#[sqlx::test]
async fn test_stats_with_pagination(pool: PgPool) {
    let (state, _rx) = common::create_test_state(pool.clone());
    let app = Router::new()
        .route("/api/stats/{code}", get(stats_handler))
        .with_state(state);

    let server = TestServer::new(app).unwrap();

    let domain_id = common::create_test_domain(&pool, "paginate-test.com").await;
    common::create_test_link(&pool, "paginate", "https://example.com", domain_id).await;

    let link_id: i64 = sqlx::query_scalar!("SELECT id FROM links WHERE code = 'paginate'")
        .fetch_one(&pool)
        .await
        .unwrap();

    for i in 1..=15 {
        common::create_test_click(&pool, link_id, &format!("10.0.0.{}", i)).await;
    }
    let response = server
        .get("/api/stats/paginate")
        .add_query_param("page_size", "10")
        .await;

    response.assert_status_ok();

    let json = response.json::<serde_json::Value>();
    assert_eq!(json["total"], 15);
    assert_eq!(json["pagination"]["page_size"], 10);
    assert_eq!(json["items"].as_array().unwrap().len(), 10);
}

#[sqlx::test]
async fn test_stats_list_all(pool: PgPool) {
    let (state, _rx) = common::create_test_state(pool.clone());
    let app = Router::new()
        .route("/api/stats", get(stats_list_handler))
        .with_state(state);

    let server = TestServer::new(app).unwrap();

    let domain_id = common::create_test_domain(&pool, "list-test.com").await;

    for i in 1..=3 {
        common::create_test_link(
            &pool,
            &format!("link{}", i),
            &format!("https://example.com/{}", i),
            domain_id,
        )
        .await;
    }

    let response = server.get("/api/stats").await;

    response.assert_status_ok();

    let json = response.json::<serde_json::Value>();
    assert!(json["items"].as_array().unwrap().len() >= 3);
    assert!(json.get("pagination").is_some());
}

#[sqlx::test]
async fn test_stats_list_with_clicks(pool: PgPool) {
    let (state, _rx) = common::create_test_state(pool.clone());
    let app = Router::new()
        .route("/api/stats", get(stats_list_handler))
        .with_state(state);

    let server = TestServer::new(app).unwrap();

    let domain_id = common::create_test_domain(&pool, "clicks-list.com").await;
    common::create_test_link(&pool, "popular", "https://example.com", domain_id).await;

    let link_id: i64 = sqlx::query_scalar!("SELECT id FROM links WHERE code = 'popular'")
        .fetch_one(&pool)
        .await
        .unwrap();

    for i in 1..=10 {
        common::create_test_click(&pool, link_id, &format!("192.168.1.{}", i)).await;
    }

    let response = server.get("/api/stats").await;

    response.assert_status_ok();

    let json = response.json::<serde_json::Value>();
    let items = json["items"].as_array().unwrap();

    let popular = items.iter().find(|item| item["code"] == "popular");

    assert!(popular.is_some());
    assert_eq!(popular.unwrap()["total"], 10);
}

#[sqlx::test]
async fn test_stats_list_pagination(pool: PgPool) {
    let (state, _rx) = common::create_test_state(pool.clone());
    let app = Router::new()
        .route("/api/stats", get(stats_list_handler))
        .with_state(state);

    let server = TestServer::new(app).unwrap();

    let domain_id = common::create_test_domain(&pool, "many-links.com").await;

    for i in 1..=30 {
        common::create_test_link(
            &pool,
            &format!("many{}", i),
            &format!("https://example.com/{}", i),
            domain_id,
        )
        .await;
    }
    let response = server
        .get("/api/stats")
        .add_query_param("page", "2")
        .add_query_param("page_size", "10")
        .await;

    response.assert_status_ok();

    let json = response.json::<serde_json::Value>();
    assert_eq!(json["pagination"]["page"], 2);
    assert_eq!(json["pagination"]["page_size"], 10);
    assert!(json["pagination"]["total_items"].as_i64().unwrap() >= 30);
}
