# URL Shortener

Production-ready URL shortener built with Rust using Clean Architecture principles, powered by Axum + SQLx + PostgreSQL.

[![Rust](https://img.shields.io/badge/rust-1.83%2B-orange.svg)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

## 🚀 Features

### Core Functionality
- **Link Shortening**: `POST /api/shorten` accepts batch URL creation
- **Smart Normalization**: automatic URL canonicalization (lowercase host, fragment removal, default port cleanup)
- **Deduplication**: identical normalized URLs receive the same short code
- **Redirect**: `GET /{code}` performs 307 redirect to original URL
- **Async Analytics**: clicks recorded via in-memory queue with background worker and retry logic

### Statistics & Analytics
- **Link List**: `GET /api/stats` — all links with click counts
- **Detailed Stats**: `GET /api/stats/{code}` — individual link click history
- **Pagination**: `page` and `page_size` parameters (10-50, default 25)
- **Date Filtering**: `from` and `to` parameters in RFC3339 format
- **Domain Filtering**: `domain` parameter (string)
- **Click Metadata**: IP address, User-Agent, Referer, timestamp

### Administration
- **Dashboard**: `GET /dashboard`
- **All Links**: `GET /dashboard/links`
- **Link Statistics**: `GET /dashboard/stats/{code}`
- **Token Login**: `GET /dashboard/login`
- **Domain List**: `GET /api/domains`
- **Service Health**: `GET /api/health`

### Security & Monitoring
- **Authentication**: Bearer token protection for statistics endpoints
- **Detailed Errors**: structured JSON responses with error codes and details
- **Access Logging**: nginx-style logging (IP, method, path, status, latency)
- **Metrics**: database error counters by type for monitoring
- **Graceful Error Handling**: different database error types handled appropriately

## 🏗️ Architecture

Built with **Clean Architecture** principles for maximum maintainability and testability:

```

src/
├── lib.rs                     \# Dependency composition
├── main.rs                    \# Entry point
├── server.rs                  \# HTTP server
├── error.rs                   \# Error handling + sqlx::Error mapping
├── config.rs                  \# Configuration
├── routes.rs                  \# Common routes
├── api/                       \# Presentation Layer
│   ├── routes.rs
│   ├── dto/                   \# Request/Response models
│   ├── handlers/              \# HTTP handlers
│   └── middleware/            \# HTTP middleware
├── application/               \# Application Layer
│   └── services/              \# Business logic
├── bin/
│   └── admin.rs               \# CLI tools
├── domain/                    \# Domain Layer
│   ├── click_event.rs         \# Click event domain object
│   ├── click_worker.rs        \# Click processing worker
│   ├── entities/              \# Domain entities
│   └── repositories/          \# Repository trait interfaces
├── infrastructure/            \# Infrastructure Layer
│   ├── cache/                 \# Redis implementations
│   └── persistence/           \# PostgreSQL implementations
├── utils/                     \# Utilities
└── web/                       \# Frontend
    ├── handlers/
    ├── middleware/
    └── templates/

```

### Architecture Benefits

✅ **Separation of Concerns**: each layer has clear responsibilities  
✅ **Testability**: business logic isolated from HTTP and database  
✅ **Framework Independence**: domain layer doesn't depend on Axum or SQLx  
✅ **Easy Infrastructure Replacement**: swap PostgreSQL for MySQL without touching business logic  
✅ **Scalability**: easy to add new features and services

## 📋 Requirements

- **Rust**: stable toolchain + cargo
- **PostgreSQL**: 14+ (local or via Docker)
- **Redis**: 7+ (local or via Docker)
- **sqlx-cli**: for migrations (optional)

## ⚙️ Configuration

Environment variables (or `.env` file):

```env
# Required
DATABASE_URL=postgres://postgres:postgres@127.0.0.1:5432/shorty
LISTEN=0.0.0.0:3000

# Optional
REDIS_URL=redis://localhost:6379
AUTH_TOKEN=your-secret-token-here

# Logging
RUST_LOG=info,url_shortener=debug
LOG_FORMAT=json
```

| Variable       | Description                  | Example                       |
|:---------------|:-----------------------------|:------------------------------|
| `DATABASE_URL` | PostgreSQL connection string | `postgres://user:pass@host/db` |
| `REDIS_URL`    | Redis connection string      | `redis://localhost:6379`      |
| `LISTEN`       | HTTP server bind address     | `0.0.0.0:3000`                |
| `AUTH_TOKEN`   | API authentication token     | `your-secret-token`           |
| `RUST_LOG`     | Logging level                | `info` / `debug` / `trace`    |
| `LOG_FORMAT`   | Logging format               | `text` / `json`     |

## 🚀 Quick Start

### 1. Install sqlx-cli

```bash
cargo install sqlx-cli --no-default-features --features postgres
```


### 2. Create Database and Run Migrations

```bash
# Create database
sqlx database create

# Run migrations
sqlx migrate run
```


### 3. Prepare Offline Mode (Optional)

```bash
# Generates .sqlx/ for compile-time SQL checking without database
cargo sqlx prepare -- --bin url-shortener
```


### 4. Start Service

```bash
# Development mode
cargo run

# Production build
cargo build --release
./target/release/url-shortener
```


### 5. Using Docker

```bash
# Build image
docker build -t url-shortener .

# Run with docker-compose
docker-compose up -d
```


## 🐳 Docker Deployment

### Docker Compose (Recommended)

**Start services:**

```bash
docker-compose up -d
```

**Run migrations:**

```bash
docker-compose exec app sqlx migrate run
```


## 📡 API Reference

### Create Short Links

**Endpoint:** `POST /api/shorten`

**Content-Type:** `application/json`

**Request Body:**

- `domain` — optional, link will use default domain if not specified
- `custom_code` — optional, desired custom code; auto-generated if not provided

```json
{
  "urls": [
    {
      "url": "https://example.com/very/long/path",
      "custom_code": "promo2024"
    },
    {
      "url": "https://github.com/rust-lang/rust",
      "domain": "s.example.com",
      "custom_code": "rust-repo"
    },
    {
      "url": "https://docs.rs/axum"
    }
  ]
}
```

**Response:** `200 OK`

```json
{
  "summary": {
    "total": 3,
    "successful": 2,
    "failed": 1
  },
  "items": [
    {
      "long_url": "https://example.com/very/long/path",
      "code": "promo2024",
      "short_url": "https://s.example.com/promo2024"
    },
    {
      "long_url": "https://github.com/rust-lang/rust",
      "error": {
        "code": "conflict",
        "message": "Custom code already exists for this domain",
        "details": {
          "code": "rust-repo",
          "domain_id": 1
        }
      }
    },
    {
      "long_url": "https://docs.rs/axum",
      "code": "qh3h-ccXXRgY",
      "short_url": "https://s.example.com/qh3h-ccXXRgY"
    }
  ]
}
```

**Example with curl:**

```bash
curl -X POST http://127.0.0.1:3000/api/shorten \
  -H 'Content-Type: application/json' \
  -d '{
    "urls": [
      {
        "url": "https://example.com/very/long/path",
        "custom_code": "promo2024"
      }
    ]
  }' | jq
```


---

### Redirect by Short Code

**Endpoint:** `GET /{code}`

**Response:** `307 Temporary Redirect`

```http
Location: https://example.com/very/long/path
```

**Example:**

```bash
curl -i http://127.0.0.1:3000/promo2024
```

**Behavior:**

- Redirects to original URL
- Asynchronously records click event (IP, User-Agent, Referer)

---

### List All Links with Statistics

**Endpoint:** `GET /api/stats`

**Authorization:** `Bearer <token>` (required)

**Query Parameters:**


| Parameter | Type | Default | Description |
| :-- | :-- | :-- | :-- |
| `page` | integer | 1 | Page number (1-indexed) |
| `page_size` | integer | 25 | Page size (10-50) |
| `from` | RFC3339 | — | Filter: clicks from date |
| `to` | RFC3339 | — | Filter: clicks until date |
| `domain` | string | — | Filter: by domain |

**Response:** `200 OK`

```json
{
  "pagination": {
    "page": 1,
    "page_size": 25,
    "total_items": 157,
    "total_pages": 7
  },
  "items": [
    {
      "code": "promo2024",
      "domain": "s.example.com",
      "long_url": "https://example.com/very/long/path",
      "total": 42,
      "created_at": "2026-01-16T10:30:00Z"
    }
  ]
}
```

**Example:**

```bash
curl "http://127.0.0.1:3000/api/stats?page=1&page_size=10" \
  -H "Authorization: Bearer YOUR_TOKEN" | jq
```


---

### Detailed Statistics by Code

**Endpoint:** `GET /api/stats/{code}`

**Authorization:** `Bearer <token>` (required)

**Query Parameters:** same as `/api/stats`

**Note:** if `domain` filter is not provided, returns first matching link

**Response:** `200 OK`

```json
{
  "pagination": {
    "page": 1,
    "page_size": 25,
    "total_items": 42,
    "total_pages": 2
  },
  "code": "promo2024",
  "domain": "s.example.com",
  "long_url": "https://example.com/very/long/path",
  "created_at": "2026-01-16T10:30:00Z",
  "total": 42,
  "items": [
    {
      "clicked_at": "2026-01-16T18:45:23Z",
      "user_agent": "Mozilla/5.0...",
      "referer": "https://news.ycombinator.com/",
      "ip": "203.0.113.42"
    }
  ]
}
```

**Example with filtering:**

```bash
curl "http://127.0.0.1:3000/api/stats/promo2024?from=2026-01-01T00:00:00Z&to=2026-01-16T23:59:59Z" \
  -H "Authorization: Bearer YOUR_TOKEN" | jq
```


---

### Service Health

**Endpoint:** `GET /api/health`

**Authorization:** `Bearer <token>` (required)

**Response:** `200 OK`

```json
{
  "status": "healthy",
  "version": "0.1.0",
  "checks": {
    "database": {
      "status": "ok",
      "message": "Connected, default domain: s.example.com"
    },
    "click_queue": {
      "status": "ok",
      "message": "Capacity: 10000"
    }
  }
}
```

**Example:**

```bash
curl "http://127.0.0.1:3000/api/health" \
  -H "Authorization: Bearer YOUR_TOKEN" | jq
```


---

### List Domains

**Endpoint:** `GET /api/domains`

**Authorization:** `Bearer <token>` (required)

**Response:** `200 OK`

```json
{
  "items": [
    {
      "domain": "s.example.com",
      "is_default": true,
      "is_active": true,
      "description": "Default domain",
      "created_at": "2026-01-17T08:22:13.685467Z",
      "updated_at": "2026-01-17T08:22:13.685467Z"
    }
  ]
}
```

**Example:**

```bash
curl "http://127.0.0.1:3000/api/domains" \
  -H "Authorization: Bearer YOUR_TOKEN" | jq
```


---

## 🔐 Authentication

Protected endpoints:

- `GET /api/stats`
- `GET /api/stats/{code}`
- `GET /api/health`
- `GET /api/domains`

**Header format:**

```http
Authorization: Bearer <your-token>
```


### Creating Tokens

**Option 1: SQL (for initial setup)**

```sql
-- Insert token (stored as SHA256 hash)
INSERT INTO api_tokens (name, token_hash)
VALUES ('My App', encode(sha256('your-secret-token'::bytea), 'hex'));
```

**Option 2: CLI Tool**

```bash
cargo run --bin admin -- create-token "My App"
# Output: Token created: randomly-generated-secure-token
```

**Usage:**

```bash
export TOKEN="your-secret-token"
curl -H "Authorization: Bearer $TOKEN" http://127.0.0.1:3000/api/stats
```


---

## ❌ Error Handling

All errors return unified JSON format:

```json
{
  "error": {
    "code": "validation_error",
    "message": "Invalid URL format",
    "details": {
      "reason": "Only HTTP and HTTPS protocols are allowed"
    }
  }
}
```

**Error Types:**


| HTTP Status | Error Code | Description |
| :-- | :-- | :-- |
| 400 | `validation_error` | Invalid input data |
| 401 | `unauthorized` | Missing or invalid token |
| 404 | `not_found` | Resource not found |
| 409 | `conflict` | Conflict (e.g., duplicate code) |
| 500 | `internal_error` | Internal server error |

**Examples:**

Invalid URL:

```json
{
  "error": {
    "code": "validation_error",
    "message": "Invalid URL format",
    "details": { "reason": "Invalid URL: relative URL without a base" }
  }
}
```

Code not found:

```json
{
  "error": {
    "code": "not_found",
    "message": "Short link not found",
    "details": { "code": "unknown123" }
  }
}
```

Unauthorized:

```json
{
  "error": {
    "code": "unauthorized",
    "message": "Unauthorized",
    "details": { "reason": "Invalid or revoked token" }
  }
}
```


---

## 🚦 Rate Limiting

IP-based rate limiting protects against abuse.

### Public Endpoints

Applied to `/api/shorten` and `/{code}`:

- **Limit**: 2 requests per second (120 req/min)
- **Burst**: up to 100 concurrent requests
- **Key**: Client IP address


### Protected Endpoints (require authentication)

Applied to `/api/health`, `/api/domains`, `/api/stats`, `/api/stats/{code}`:

- **Limit**: 1 request per second (60 req/min)
- **Burst**: up to 10 concurrent requests
- **Key**: Client IP address


### Behavior on Limit Exceeded

Client receives HTTP `429 Too Many Requests`. Counters reset automatically based on `per_second` settings.

---

## 📊 Monitoring \& Logging

### Logging

Using `tracing` for structured logs:

```bash
# Important events only
RUST_LOG=info cargo run

# Detailed debugging
RUST_LOG=debug,url_shortener=trace cargo run
```


### Metrics

Built-in counters (via `metrics` crate):

- `click_worker_received_total` — click events received
- `click_worker_processed_total` — events successfully processed
- `click_worker_failed_total` — processing errors
- `click_worker_retried_total` — retry count
- `database_errors_total{type="..."}` — database errors by type


### Metrics Endpoint

**Endpoint:** `GET /metrics`

**Response:** Prometheus format

```
# HELP click_worker_received_total Total click events received
# TYPE click_worker_received_total counter
click_worker_received_total 1234

# HELP click_worker_processed_total Total click events processed
# TYPE click_worker_processed_total counter
click_worker_processed_total 1200

# HELP database_errors_total Database errors by type
# TYPE database_errors_total counter
database_errors_total{type="connection"} 5
database_errors_total{type="query"} 12
```


---

## 🧪 Testing

Run all tests:

```bash
cargo test
```

Run specific test suite:

```bash
# Unit tests only
cargo test --lib

# Integration tests only
cargo test --test '*'

# With logging output
cargo test -- --nocapture
```

See [TESTING.md](TESTING.md) for detailed testing documentation.

---

## 🛠️ CLI Tools

### Admin CLI

```bash
# Create API token
cargo run --bin admin -- create-token "My Application"

# List all tokens
cargo run --bin admin -- list-tokens

# Revoke token
cargo run --bin admin -- revoke-token <token_id>

# Add domain
cargo run --bin admin -- add-domain "short.link" --default

# List domains
cargo run --bin admin -- list-domains
```


---

## 📦 Database Schema

### Tables

**domains**

- `id` (PK): Domain identifier
- `domain`: Domain name (unique)
- `is_default`: Default domain flag
- `is_active`: Active status
- `description`: Optional description
- `created_at`, `updated_at`: Timestamps

**links**

- `id` (PK): Link identifier
- `code`: Short code (unique per domain)
- `long_url`: Original URL
- `normalized_url`: Canonicalized URL (for deduplication)
- `domain_id` (FK): Domain reference
- `created_at`: Creation timestamp
- Unique constraint: `(code, domain_id)`
- Unique constraint: `(normalized_url, domain_id)`

**clicks**

- `id` (PK): Click identifier
- `link_id` (FK): Link reference
- `clicked_at`: Click timestamp
- `ip`: Client IP address
- `user_agent`: Client User-Agent
- `referer`: HTTP Referer header

**api_tokens**

- `id` (PK): Token identifier
- `name`: Token description
- `token_hash`: SHA256 hash of token
- `is_active`: Active status
- `created_at`, `last_used_at`: Timestamps

---

## 🔧 Development

### Database Migrations

Create new migration:

```bash
sqlx migrate add create_new_table
```

Run migrations:

```bash
sqlx migrate run
```

Revert last migration:

```bash
sqlx migrate revert
```


### Code Quality

```bash
# Format code
cargo fmt

# Linting
cargo clippy -- -D warnings

# Generate documentation
cargo doc --open --no-deps
```


---

## 📈 Performance

### Benchmarks

- **Link creation**: ~500 req/s (single core)
- **Redirect**: ~10,000 req/s (single core, cached)
- **Statistics**: ~1,000 req/s (with pagination)


### Optimization Tips

1. **Redis caching**: Enable Redis for frequently accessed links
2. **Connection pooling**: Adjust `DATABASE_POOL_SIZE` based on load
3. **Click batching**: Increase `CLICK_BATCH_SIZE` for high traffic
4. **Horizontal scaling**: Run multiple instances behind load balancer

---

## 📝 License

MIT License - see [LICENSE](LICENSE) file for details

---

## 🤝 Contributing

Pull requests are welcome! For major changes:

1. Open an issue to discuss proposed changes
2. Fork the repository
3. Create a feature branch
4. Make your changes with tests
5. Submit a pull request

### Development Setup

```bash
# Clone repository
git clone https://github.com/bobrynya/url-shortener.git
cd url-shortener

# Install dependencies
cargo build

# Run tests
cargo test

# Start development server
cargo run
```


---

## 🔗 Additional Resources

- [Axum Documentation](https://docs.rs/axum)
- [SQLx Documentation](https://docs.rs/sqlx)
- [Clean Architecture by Robert Martin](https://blog.cleancoder.com/uncle-bob/2012/08/13/the-clean-architecture.html)
- [Rust Async Book](https://rust-lang.github.io/async-book/)

---

## 📝 Roadmap

- [ ] Dashboard improvements
  - [ ] Link deletion
  - [ ] Domain management CRUD
  - [ ] Health status display
- [ ] QR code generation for short links
- [ ] Custom URL slugs validation rules
- [ ] Link expiration/TTL
- [ ] Webhook notifications for clicks
- [ ] Export statistics to CSV/JSON
- [ ] Multi-language support
- [ ] Link preview metadata scraping

---

## 📞 Support

- **Issues**: [GitHub Issues](https://github.com/bobrynya/url-shortener/issues)
- **Discussions**: [GitHub Discussions](https://github.com/bobrynya/url-shortener/discussions)
- **Email**: chernyakov@decanet.ru

---

**Made with ❤️ and 🦀 Rust**

```

