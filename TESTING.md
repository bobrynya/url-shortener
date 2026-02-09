# **TESTING.md**

```markdown
# Testing Guide

Comprehensive testing documentation for the URL Shortener project.

## 📊 Test Statistics

- **Total tests:** 163
- **Code coverage:** ~48%
- **Unit tests:** 108
- **Integration tests:** 55

## 🚀 Running Tests

### All Tests

Run the complete test suite:

```bash
cargo test
```


### Unit Tests (Fast, No Database)

Run only unit tests without database requirements:

```bash
cargo test --lib
```


### Integration Tests (With Database)

Run integration tests that require PostgreSQL:

```bash
cargo test --tests
```


### Specific Test

Run a single test by name:

```bash
cargo test test_create_link
```


### With Output

Show `println!` and logging output:

```bash
cargo test -- --nocapture
```


### Single-Threaded

Run tests sequentially (useful for debugging):

```bash
cargo test -- --test-threads=1
```


### Verbose Output

Show detailed test results:

```bash
cargo test -- --nocapture --test-threads=1
```


---

## 📋 Requirements

### For Unit Tests

- No external dependencies required
- Pure Rust logic testing


### For Integration Tests

- **PostgreSQL** 14+ (local or Docker)
- **DATABASE_URL** environment variable
- `sqlx::test` automatically creates temporary databases


### Setup Database for Tests

```bash
# Export database URL (or use .env)
export DATABASE_URL="postgres://postgres:postgres@localhost:5432/shorty_test"

# Create test database
sqlx database create

# Run migrations
sqlx migrate run
```


---

## 🏗️ Test Structure

```
tests/
├── common/
│   └── mod.rs                 # Test fixtures and helpers
├── handler_health_test.rs     # Health endpoint E2E tests
├── handler_shorten_test.rs    # Shorten endpoint E2E tests
├── handler_redirect_test.rs   # Redirect endpoint E2E tests
├── handler_stats_test.rs      # Statistics endpoint E2E tests
├── repository_link_test.rs    # Link repository integration tests
├── repository_click_test.rs   # Click repository integration tests
└── repository_domain_test.rs  # Domain repository integration tests
```


### Test Layers

#### 1. Unit Tests (`src/**/*.rs`)

Located alongside source code, testing individual functions and logic:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_url_normalization() {
        let url = "HTTPS://EXAMPLE.COM:443/path?query=1#fragment";
        let normalized = normalize_url(url).unwrap();
        assert_eq!(normalized, "https://example.com/path?query=1");
    }
}
```

**Test coverage:**

- URL normalization logic
- Error type conversions
- Utility functions
- Domain entities validation


#### 2. Integration Tests (`tests/*.rs`)

End-to-end testing with real database and HTTP server:

```rust
#[sqlx::test]
async fn test_create_link(pool: PgPool) {
    // Arrange
    let app = setup_test_app(pool).await;
    
    // Act
    let response = app
        .post("/api/shorten")
        .json(&json!({
            "urls": [{"url": "https://example.com"}]
        }))
        .await;
    
    // Assert
    assert_eq!(response.status(), StatusCode::OK);
}
```

**Test coverage:**

- API endpoints (handlers)
- Database repositories
- Authentication middleware
- Rate limiting
- Error responses

---

## 🧪 Test Categories

### 1. Handler Tests

Testing HTTP API endpoints with real requests.

**File:** `tests/handler_shorten_test.rs`

```bash
# Run all handler tests
cargo test handler_

# Run specific handler test suite
cargo test handler_shorten
```

**Example tests:**

- `test_shorten_success` — successful link creation
- `test_shorten_invalid_url` — validation error handling
- `test_shorten_duplicate_code` — conflict detection
- `test_shorten_unauthorized` — auth middleware


### 2. Repository Tests

Testing database operations directly.

**File:** `tests/repository_link_test.rs`

```bash
# Run all repository tests
cargo test repository_

# Run specific repository test suite
cargo test repository_link
```

**Example tests:**

- `test_create_and_find_link` — CRUD operations
- `test_find_by_normalized_url` — deduplication logic
- `test_pagination` — page/limit queries
- `test_concurrent_inserts` — race condition handling


### 3. Service Tests

Testing business logic layer.

**Located in:** `src/application/services/*.rs`

```bash
# Run service layer tests
cargo test application::services
```

**Example tests:**

- `test_link_service_create` — link creation logic
- `test_stats_service_filtering` — date range filtering
- `test_click_worker_retry` — retry mechanism

---

## 📈 Code Coverage

### Using Tarpaulin

Install:

```bash
cargo install cargo-tarpaulin
```

Generate HTML coverage report:

```bash
cargo tarpaulin --out Html
```

Open report:

```bash
open tarpaulin-report.html
```


### Using Llvm-cov

Install:

```bash
rustup component add llvm-tools-preview
cargo install cargo-llvm-cov
```

Generate coverage:

```bash
# HTML report
cargo llvm-cov --html

# Open in browser
cargo llvm-cov --open

# JSON output for CI
cargo llvm-cov --json --output-path coverage.json
```


### Coverage Targets

Current coverage by module:


| Module | Coverage | Target |
| :-- | :-- | :-- |
| `domain/entities` | 85% | 90% |
| `domain/repositories` | 100% | 100% |
| `application/services` | 65% | 80% |
| `infrastructure` | 40% | 60% |
| `api/handlers` | 70% | 85% |
| `api/middleware` | 60% | 75% |
| **Overall** | **48%** | **70%** |


---

## 🔍 Testing Best Practices

### 1. Arrange-Act-Assert Pattern

```rust
#[test]
fn test_example() {
    // Arrange: setup test data
    let input = "test";
    
    // Act: execute function under test
    let result = process(input);
    
    // Assert: verify expectations
    assert_eq!(result, "expected");
}
```


### 2. Use Descriptive Test Names

```rust
// ❌ Bad
#[test]
fn test1() { }

// ✅ Good
#[test]
fn test_create_link_with_custom_code_succeeds() { }
```


### 3. Test Edge Cases

```rust
#[test]
fn test_empty_url() { }

#[test]
fn test_malformed_url() { }

#[test]
fn test_very_long_url() { }

#[test]
fn test_special_characters_in_code() { }
```


### 4. Use Test Fixtures

**File:** `tests/common/mod.rs`

```rust
pub async fn setup_test_app(pool: PgPool) -> TestApp {
    // Common setup logic
}

pub fn sample_link() -> Link {
    Link {
        code: "test123".to_string(),
        long_url: "https://example.com".to_string(),
        // ...
    }
}
```


### 5. Clean Up After Tests

```rust
#[sqlx::test]
async fn test_with_cleanup(pool: PgPool) {
    // Test logic...
    
    // Cleanup (if not using sqlx::test auto-rollback)
    sqlx::query!("DELETE FROM links WHERE code = $1", "test123")
        .execute(&pool)
        .await
        .unwrap();
}
```


---

## 🐳 Testing with Docker

### Start Test Database

```bash
docker run -d \
  --name postgres-test \
  -e POSTGRES_PASSWORD=postgres \
  -e POSTGRES_DB=shorty_test \
  -p 5433:5432 \
  postgres:16-alpine
```


### Run Tests Against Docker

```bash
DATABASE_URL="postgres://postgres:postgres@localhost:5433/shorty_test" \
  cargo test
```


### Docker Compose for Tests

**File:** `docker-compose.test.yml`

```yaml
version: '3.8'

services:
  postgres-test:
    image: postgres:16-alpine
    environment:
      POSTGRES_PASSWORD: postgres
      POSTGRES_DB: shorty_test
    ports:
      - "5433:5432"
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U postgres"]
      interval: 5s
      timeout: 5s
      retries: 5
```

**Usage:**

```bash
# Start test database
docker-compose -f docker-compose.test.yml up -d

# Wait for health check
docker-compose -f docker-compose.test.yml ps

# Run tests
DATABASE_URL="postgres://postgres:postgres@localhost:5433/shorty_test" \
  cargo test

# Stop test database
docker-compose -f docker-compose.test.yml down -v
```


---

## 🔧 Debugging Tests

### Enable Logging

```bash
RUST_LOG=debug cargo test -- --nocapture
```


### Run Single Test with Trace

```bash
RUST_LOG=trace cargo test test_specific_function -- --nocapture --exact
```


### Print SQL Queries

Add to test:

```rust
#[sqlx::test]
async fn test_with_sql_logging(pool: PgPool) {
    std::env::set_var("SQLX_LOGGING", "true");
    // Test logic...
}
```


### Use Debugger

With VS Code launch.json:

```json
{
  "type": "lldb",
  "request": "launch",
  "name": "Debug unit tests",
  "cargo": {
    "args": ["test", "--no-run", "--lib"],
    "filter": {
      "name": "url_shortener",
      "kind": "lib"
    }
  },
  "args": ["test_function_name"],
  "cwd": "${workspaceFolder}"
}
```


---

## 🚦 Continuous Integration

### GitHub Actions Example

**File:** `.github/workflows/test.yml`

```yaml
name: Tests

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    
    services:
      postgres:
        image: postgres:16
        env:
          POSTGRES_PASSWORD: postgres
          POSTGRES_DB: shorty_test
        options: >-
          --health-cmd pg_isready
          --health-interval 10s
          --health-timeout 5s
          --health-retries 5
        ports:
          - 5432:5432
    
    steps:
      - uses: actions/checkout@v3
      
      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          override: true
      
      - name: Run migrations
        run: |
          cargo install sqlx-cli --no-default-features --features postgres
          sqlx migrate run
        env:
          DATABASE_URL: postgres://postgres:postgres@localhost:5432/shorty_test
      
      - name: Run tests
        run: cargo test --verbose
        env:
          DATABASE_URL: postgres://postgres:postgres@localhost:5432/shorty_test
      
      - name: Generate coverage
        run: |
          cargo install cargo-tarpaulin
          cargo tarpaulin --out Xml
      
      - name: Upload coverage
        uses: codecov/codecov-action@v3
        with:
          files: ./cobertura.xml
```


---

## 📝 Writing New Tests

### 1. Unit Test Template

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_function_name() {
        // Arrange
        let input = setup_test_data();
        
        // Act
        let result = function_under_test(input);
        
        // Assert
        assert_eq!(result, expected_value);
    }

    #[test]
    #[should_panic(expected = "error message")]
    fn test_function_panics() {
        panic_function();
    }
}
```


### 2. Integration Test Template

```rust
use sqlx::PgPool;

mod common;

#[sqlx::test]
async fn test_database_operation(pool: PgPool) -> Result<(), Box<dyn std::error::Error>> {
    // Arrange
    let repo = LinkRepository::new(pool);
    let link = common::sample_link();
    
    // Act
    let created = repo.create(link).await?;
    
    // Assert
    assert_eq!(created.code, "test123");
    
    Ok(())
}
```


### 3. Handler Test Template

```rust
use axum::http::StatusCode;
use serde_json::json;

mod common;

#[sqlx::test]
async fn test_api_endpoint(pool: PgPool) {
    // Arrange
    let app = common::setup_test_app(pool).await;
    
    // Act
    let response = app
        .post("/api/shorten")
        .json(&json!({
            "urls": [{"url": "https://example.com"}]
        }))
        .await;
    
    // Assert
    assert_eq!(response.status(), StatusCode::OK);
    
    let body: serde_json::Value = response.json().await;
    assert_eq!(body["summary"]["successful"], 1);
}
```


---

## 🎯 Test Checklist

Before submitting a PR, ensure:

- [ ] All tests pass: `cargo test`
- [ ] Code is formatted: `cargo fmt`
- [ ] No clippy warnings: `cargo clippy -- -D warnings`
- [ ] New features have tests
- [ ] Edge cases are covered
- [ ] Error cases are tested
- [ ] Documentation is updated
- [ ] Integration tests pass with real database
- [ ] Coverage hasn't significantly decreased

---

## 🔗 Resources

- [Rust Testing Documentation](https://doc.rust-lang.org/book/ch11-00-testing.html)
- [SQLx Testing Guide](https://github.com/launchbadge/sqlx/blob/main/sqlx-macros/README.md#testing)
- [Axum Testing Examples](https://github.com/tokio-rs/axum/tree/main/examples)
- [Tarpaulin Documentation](https://github.com/xd009642/tarpaulin)

---

## 💡 Tips \& Tricks

### Parallel Test Execution

```bash
# Default: parallel execution
cargo test

# Sequential execution (for debugging)
cargo test -- --test-threads=1
```


### Filter Tests by Name

```bash
# Run tests containing "link" in name
cargo test link

# Run tests in specific module
cargo test application::services::link_service
```


### Ignore Slow Tests

Mark slow tests:

```rust
#[test]
#[ignore]
fn slow_integration_test() {
    // Expensive test...
}
```

Run only ignored tests:

```bash
cargo test -- --ignored
```

Run all tests including ignored:

```bash
cargo test -- --include-ignored
```


### Test with Release Build

```bash
cargo test --release
```


---

**Happy Testing! 🧪🦀**
