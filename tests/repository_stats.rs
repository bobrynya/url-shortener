mod common;

use sqlx::PgPool;
use std::sync::Arc;
use url_shortener::domain::entities::NewClick;
use url_shortener::domain::repositories::{StatsFilter, StatsRepository};
use url_shortener::infrastructure::persistence::PgStatsRepository;

#[sqlx::test]
async fn test_record_click(pool: PgPool) {
    let repo = PgStatsRepository::new(Arc::new(pool.clone()));

    let domain_id = common::create_test_domain(&pool, "stats-test.com").await;
    common::create_test_link(&pool, "click123", "https://example.com", domain_id).await;

    let link_id: i64 = sqlx::query_scalar!("SELECT id FROM links WHERE code = $1", "click123")
        .fetch_one(&pool)
        .await
        .unwrap();

    let new_click = NewClick {
        link_id,
        user_agent: Some("Mozilla/5.0".to_string()),
        referer: None,
        ip: Some("192.168.1.1".to_string()),
    };

    let result = repo.record_click(new_click).await;

    assert!(result.is_ok());
    let click = result.unwrap();
    assert_eq!(click.link_id, link_id);
    assert_eq!(click.user_agent, Some("Mozilla/5.0".to_string()));
}

#[sqlx::test]
async fn test_get_stats_by_code(pool: PgPool) {
    let repo = PgStatsRepository::new(Arc::new(pool.clone()));

    let domain_id = common::create_test_domain(&pool, "stats-test2.com").await;
    common::create_test_link(&pool, "stats456", "https://example.com", domain_id).await;

    let link_id: i64 = sqlx::query_scalar!("SELECT id FROM links WHERE code = $1", "stats456")
        .fetch_one(&pool)
        .await
        .unwrap();

    for i in 1..=5 {
        common::create_test_click(&pool, link_id, &format!("192.168.1.{}", i)).await;
    }

    let filter = StatsFilter::new(0, 10).with_domain(Some(domain_id));
    let result = repo.get_stats_by_code("stats456", filter).await;

    assert!(result.is_ok());
    let stats = result.unwrap();
    assert!(stats.is_some());
    let stats = stats.unwrap();
    assert_eq!(stats.total, 5);
    assert_eq!(stats.link.code, "stats456");
}

#[sqlx::test]
async fn test_get_all_stats(pool: PgPool) {
    let repo = PgStatsRepository::new(Arc::new(pool.clone()));

    let domain_id = common::create_test_domain(&pool, "stats-test3.com").await;

    common::create_test_link(&pool, "link1", "https://example.com/1", domain_id).await;
    common::create_test_link(&pool, "link2", "https://example.com/2", domain_id).await;

    let link1_id: i64 = sqlx::query_scalar!("SELECT id FROM links WHERE code = $1", "link1")
        .fetch_one(&pool)
        .await
        .unwrap();

    for i in 1..=3 {
        common::create_test_click(&pool, link1_id, &format!("192.168.1.{}", i)).await;
    }

    let filter = StatsFilter::new(0, 10).with_domain(Some(domain_id));
    let result = repo.get_all_stats(filter).await;

    assert!(result.is_ok());
    let stats = result.unwrap();
    assert!(stats.len() >= 2);

    let link1_stats = stats.iter().find(|s| s.code == "link1");
    assert!(link1_stats.is_some());
    assert_eq!(link1_stats.unwrap().total, 3);
}

#[sqlx::test]
async fn test_count_all_links(pool: PgPool) {
    let repo = PgStatsRepository::new(Arc::new(pool.clone()));

    let domain_id = common::create_test_domain(&pool, "count-test.com").await;

    for i in 1..=4 {
        common::create_test_link(
            &pool,
            &format!("count{}", i),
            &format!("https://example.com/{}", i),
            domain_id,
        )
        .await;
    }

    let result = repo.count_all_links().await;

    assert!(result.is_ok());
    assert!(result.unwrap() >= 4);
}

#[sqlx::test]
async fn test_count_clicks_by_link_id(pool: PgPool) {
    let repo = PgStatsRepository::new(Arc::new(pool.clone()));

    let domain_id = common::create_test_domain(&pool, "clicks-count.com").await;
    common::create_test_link(&pool, "countme", "https://example.com", domain_id).await;

    let link_id: i64 = sqlx::query_scalar!("SELECT id FROM links WHERE code = $1", "countme")
        .fetch_one(&pool)
        .await
        .unwrap();

    for i in 1..=7 {
        common::create_test_click(&pool, link_id, &format!("10.0.0.{}", i)).await;
    }

    let result = repo.count_clicks_by_link_id(link_id, None, None).await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 7);
}
