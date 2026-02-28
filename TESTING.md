# Testing Guide

## Test Statistics

- **Unit tests:** 123 (run with `cargo test --lib`, no database required)
- **Integration tests:** `tests/*.rs` (require PostgreSQL)

## Running Tests

```bash
# All tests
cargo test

# Unit tests only (fast, no database)
cargo test --lib

# Integration tests only
cargo test --tests

# Single test by name
cargo test test_create_link

# With stdout/tracing output
cargo test -- --nocapture

# Sequential (useful for debugging)
cargo test -- --test-threads=1
```

---

## Requirements

### Unit Tests

No external dependencies. Pure Rust logic with mockall mocks.

### Integration Tests

- **PostgreSQL** 14+ with `DATABASE_URL` set
- `TOKEN_SIGNING_SECRET` set (any value works for tests)
- `sqlx::test` creates and tears down isolated databases automatically — no manual setup needed

```bash
export DATABASE_URL="postgres://postgres:postgres@localhost:5432/shorty"
export TOKEN_SIGNING_SECRET="test-secret"
cargo test --tests
```

---

## Test Structure

```
tests/
├── common/
│   └── mod.rs                # shared app setup, token helpers
├── handler_shorten.rs        # POST /api/shorten
├── handler_redirect.rs       # GET /{code}
├── handler_stats.rs          # GET /api/stats, GET /api/stats/{code}
├── handler_health.rs         # GET /health
├── handler_domains.rs        # GET/POST/PATCH/DELETE /api/domains
├── repository_link.rs        # PgLinkRepository
├── repository_domain.rs      # PgDomainRepository
├── repository_stats.rs       # PgStatsRepository
└── repository_token.rs       # PgTokenRepository
```

### Unit Tests (`src/**/*.rs`)

Co-located with source code, using mockall mocks for repository traits.

Covered modules:
- `domain/entities` — Link, Domain, Click construction and behaviour
- `domain/click_worker` — event processing, concurrency
- `application/services` — LinkService, DomainService, StatsService, AuthService
- `config` — env var loading, validation, URL assembly
- `utils` — URL normalizer, code generator, domain extractor

### Integration Tests (`tests/*.rs`)

End-to-end with a real PostgreSQL database and a full axum test server.

```rust
#[sqlx::test(migrations = "migrations")]
async fn test_shorten_success(pool: PgPool) {
    let app = common::setup_app(pool).await;

    let response = app
        .post("/api/shorten")
        .add_header("Authorization", "Bearer test-token")
        .json(&json!({ "urls": [{ "url": "https://example.com" }] }))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);
}
```

---

## Test Categories

### Handler Tests

Run all:
```bash
cargo test handler_
```

Key scenarios covered:
- Successful operations and response shape
- Authentication — missing/invalid/revoked token → 401
- Validation errors → 400
- Conflict on duplicate code → 409
- Not found → 404
- Soft-delete and 410 Gone on redirect/link lookup
- Domain CRUD including soft-delete

### Repository Tests

Run all:
```bash
cargo test repository_
```

Key scenarios covered:
- CRUD operations with real SQL
- Deduplication via normalized URL
- Soft-delete behaviour (deleted records excluded from list queries)
- Pagination (offset/limit)
- `count_links` safety check before domain deletion

### Service Tests

```bash
cargo test application::services
```

All service tests use mockall mocks — no database required.

---

## Code Coverage

### cargo-tarpaulin

```bash
cargo install cargo-tarpaulin
cargo tarpaulin --out Html
open tarpaulin-report.html
```

### cargo-llvm-cov

```bash
rustup component add llvm-tools-preview
cargo install cargo-llvm-cov
cargo llvm-cov --html --open
```

---

## Testing with Docker

```bash
docker run -d \
  --name postgres-test \
  -e POSTGRES_PASSWORD=postgres \
  -e POSTGRES_DB=shorty \
  -p 5432:5432 \
  postgres:16-alpine

export DATABASE_URL="postgres://postgres:postgres@localhost:5432/shorty"
export TOKEN_SIGNING_SECRET="test-secret"
cargo test
```

---

## Debugging

```bash
# Enable tracing output
RUST_LOG=debug cargo test -- --nocapture

# Run one test with full trace
RUST_LOG=trace cargo test test_specific_function -- --nocapture --exact
```

---

## Checklist Before a PR

- [ ] `cargo test` passes
- [ ] `cargo fmt` — no formatting changes
- [ ] `cargo clippy -- -D warnings` — no warnings
- [ ] New behaviour has tests (unit or integration)
- [ ] Error paths are covered
