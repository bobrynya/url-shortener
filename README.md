# URL Shortener

Production-ready URL shortener built with Rust using Clean Architecture principles, powered by Axum + SQLx + PostgreSQL.

[![Rust](https://img.shields.io/badge/rust-1.93%2B-orange.svg)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

## Features

### Core Functionality
- **Link Shortening**: `POST /api/shorten` accepts batch URL creation with optional custom codes and expiry
- **Smart Normalization**: automatic URL canonicalization (lowercase host, fragment removal, default port cleanup)
- **Deduplication**: identical normalized URLs receive the same short code per domain
- **Redirect**: `GET /{code}` performs 301 (permanent) or 307 (temporary) redirect based on link settings
- **Link Management**: update destination URL, expiry, redirect type; soft-delete and restore via `PATCH /api/links/{code}`
- **Async Analytics**: clicks recorded via in-memory channel with background worker and exponential backoff retry

### Statistics & Analytics
- **Link List**: `GET /api/stats` — all links with click counts
- **Detailed Stats**: `GET /api/stats/{code}` — individual link click history with pagination
- **Date Filtering**: `from` and `to` parameters in RFC3339 format
- **Domain Filtering**: `domain` query parameter
- **Click Metadata**: IP address, User-Agent, Referer, timestamp

### Domain Management
- **List Domains**: `GET /api/domains`
- **Create Domain**: `POST /api/domains`
- **Update Domain**: `PATCH /api/domains/{id}` — rename, toggle active/default, update description
- **Soft-Delete Domain**: `DELETE /api/domains/{id}` — deleted domains return 410 Gone on redirect

### Administration
- **Web Dashboard**: `GET /dashboard`, `/dashboard/links`, `/dashboard/stats/{code}`
- **Service Health**: `GET /health` — database, cache, and click queue checks
- **Admin CLI**: token management and domain setup via `cargo run --bin admin`

### Security & Operations
- **Bearer Token Auth**: all API write and read endpoints require authentication
- **Rate Limiting**: IP-based via tower_governor; proxy-aware via `X-Forwarded-For`/`X-Real-IP`
- **Structured Errors**: unified JSON error responses with machine-readable codes
- **Graceful Shutdown**: SIGTERM + Ctrl-C handled; in-flight requests and click worker drain cleanly
- **Metrics**: Prometheus-compatible counters for click worker events and database errors

## Architecture

Built with **Clean Architecture** principles for maximum maintainability and testability:

```
src/
├── lib.rs                     # Dependency composition
├── main.rs                    # Entry point
├── server.rs                  # Server bootstrap (pool, migrations, cache, worker, axum serve)
├── error.rs                   # AppError with IntoResponse
├── config.rs                  # Config from env vars with validate()
├── routes.rs                  # Top-level router (API + web + static)
├── state.rs                   # AppState (Arc-wrapped services, mpsc sender, cache)
├── api/                       # Presentation Layer
│   ├── routes.rs              # Protected API routes
│   ├── dto/                   # Request/response models
│   ├── handlers/              # HTTP handlers
│   │   ├── domains.rs         # list, create, update, delete domain
│   │   ├── links.rs           # shorten, update, delete link
│   │   ├── stats.rs           # stats list + detailed stats
│   │   ├── redirect.rs        # short code redirect with caching
│   │   └── health.rs          # health check
│   └── middleware/            # auth, rate_limit, tracing
├── application/
│   └── services/              # Business logic (LinkService, DomainService, StatsService, AuthService)
├── bin/
│   └── admin.rs               # CLI tool (token CRUD, domain setup)
├── domain/
│   ├── click_event.rs
│   ├── click_worker.rs        # Background click processor with JoinSet concurrency
│   ├── entities/              # Link, Click, Domain
│   └── repositories/          # Repository trait interfaces (mockall-derived mocks)
├── infrastructure/
│   ├── cache/                 # RedisCache / NullCache
│   └── persistence/           # PgLinkRepository, PgDomainRepository, PgStatsRepository, PgTokenRepository
├── utils/                     # code_generator, url_normalizer, extract_domain
└── web/                       # Askama HTML dashboard
    ├── handlers/
    ├── middleware/
    └── templates/
```

### Architecture Benefits

- **Separation of Concerns**: each layer has clear responsibilities
- **Testability**: business logic isolated from HTTP and database via repository traits + mockall
- **Framework Independence**: domain layer has no dependency on Axum or SQLx
- **Easy Infrastructure Replacement**: swap PostgreSQL or Redis without touching business logic

## Requirements

- **Rust**: stable 1.93+
- **PostgreSQL**: 14+
- **Redis**: 7+ (optional — falls back to NullCache)
- **sqlx-cli**: for running migrations

## Configuration

All configuration is loaded from environment variables or a `.env` file. See `.env.example` for a full annotated template.

### Core Variables

| Variable              | Required | Default | Description |
|:----------------------|:--------:|:-------:|:------------|
| `DATABASE_URL`        | ✓*       | —       | Full PostgreSQL connection string |
| `DB_HOST`             | ✓*       | —       | Database host (alternative to `DATABASE_URL`) |
| `DB_PORT`             | —        | `5432`  | Database port |
| `DB_USER`             | —        | —       | Database user |
| `DB_PASSWORD`         | —        | —       | Database password |
| `DB_NAME`             | —        | —       | Database name |
| `LISTEN`              | —        | `0.0.0.0:3000` | HTTP bind address |
| `TOKEN_SIGNING_SECRET`| ✓        | —       | HMAC key for token hashing |
| `RUST_LOG`            | —        | `info`  | Log level (`info`, `debug`, `trace`) |
| `LOG_FORMAT`          | —        | `text`  | Log format (`text` or `json`) |

*Either `DATABASE_URL` or individual `DB_*` components are required.

### Optional Variables

| Variable                  | Default  | Description |
|:--------------------------|:--------:|:------------|
| `REDIS_URL`               | —        | Redis connection string; disables caching if absent |
| `REDIS_HOST`              | —        | Redis host (alternative to `REDIS_URL`) |
| `CACHE_TTL_SECONDS`       | `3600`   | Redis cache TTL for URL mappings |
| `CLICK_QUEUE_CAPACITY`    | `10000`  | In-memory click event buffer size |
| `CLICK_WORKER_CONCURRENCY`| `4`      | Max concurrent click DB writes (1–256) |
| `BEHIND_PROXY`            | `false`  | Use `X-Forwarded-For`/`X-Real-IP` for rate limiting |
| `DB_MAX_CONNECTIONS`      | `10`     | PostgreSQL connection pool size |

## Quick Start

### 1. Install sqlx-cli

```bash
cargo install sqlx-cli --no-default-features --features postgres
```

### 2. Configure

```bash
cp .env.example .env
# Edit .env — set DATABASE_URL or DB_* vars and TOKEN_SIGNING_SECRET
```

### 3. Create Database and Run Migrations

```bash
sqlx database create
sqlx migrate run
```

### 4. Create a Default Domain and API Token

```bash
cargo run --bin admin -- add-domain "s.example.com" --default
cargo run --bin admin -- create-token "My App"
```

### 5. Start Service

```bash
cargo run
```

### Using Docker

```bash
docker-compose up -d
docker-compose exec app sqlx migrate run
```

## API Reference

All API endpoints require `Authorization: Bearer <token>` unless noted.

---

### Redirect (Public)

**`GET /{code}`**

No authentication required.

Returns `301 Permanent Redirect` or `307 Temporary Redirect` depending on the link's `permanent` flag.

- `404 Not Found` — code does not exist
- `410 Gone` — link is deleted, expired, or its domain has been soft-deleted

```bash
curl -i http://127.0.0.1:3000/promo2024
```

---

### Create Short Links

**`POST /api/shorten`**

Batch endpoint — processes each URL independently; individual failures don't stop the batch.

```json
{
  "urls": [
    { "url": "https://example.com/very/long/path", "custom_code": "promo2024" },
    { "url": "https://github.com/rust-lang/rust", "domain": "s.example.com" },
    { "url": "https://docs.rs/axum", "expires_at": "2026-12-31T23:59:59Z", "permanent": true }
  ]
}
```

Fields per item: `url` (required), `domain`, `custom_code`, `expires_at`, `permanent`.

Response `200 OK`:

```json
{
  "summary": { "total": 3, "successful": 3, "failed": 0 },
  "items": [
    { "long_url": "https://example.com/very/long/path", "code": "promo2024", "short_url": "https://s.example.com/promo2024" }
  ]
}
```

---

### Update a Link

**`PATCH /api/links/{code}`**

Host header determines which domain the code belongs to.

All fields optional — only provided fields are changed.
`expires_at: null` clears the expiry. `restore: true` un-deletes a soft-deleted link.

```json
{
  "url": "https://new-destination.com",
  "expires_at": "2027-01-01T00:00:00Z",
  "permanent": true,
  "restore": true
}
```

Response `200 OK`: updated link object with `code`, `long_url`, `short_url`, `permanent`, `expires_at`, `deleted_at`, `created_at`.

---

### Delete a Link

**`DELETE /api/links/{code}`**

Soft-delete — sets `deleted_at`. Subsequent redirects return `410 Gone`.
Can be restored via `PATCH` with `restore: true`.

Host header determines which domain the code belongs to.

Response `204 No Content`.

---

### List All Links with Statistics

**`GET /api/stats`**

| Parameter   | Default | Description |
|:------------|:-------:|:------------|
| `page`      | `1`     | Page number (1-indexed) |
| `page_size` | `25`    | Items per page (max 1000) |
| `from`      | —       | Click date range start (RFC3339) |
| `to`        | —       | Click date range end (RFC3339) |
| `domain`    | —       | Filter by domain name |

Response `200 OK`:

```json
{
  "pagination": { "page": 1, "page_size": 25, "total_items": 157, "total_pages": 7 },
  "items": [
    { "code": "promo2024", "domain": "s.example.com", "long_url": "https://example.com/...", "total": 42, "created_at": "2026-01-16T10:30:00Z" }
  ]
}
```

---

### Detailed Statistics by Code

**`GET /api/stats/{code}`**

Same query parameters as `GET /api/stats`.

Response `200 OK`:

```json
{
  "pagination": { "page": 1, "page_size": 25, "total_items": 42, "total_pages": 2 },
  "code": "promo2024",
  "domain": "s.example.com",
  "long_url": "https://example.com/...",
  "created_at": "2026-01-16T10:30:00Z",
  "total": 42,
  "items": [
    { "clicked_at": "2026-01-16T18:45:23Z", "user_agent": "Mozilla/5.0...", "referer": "https://news.ycombinator.com/", "ip": "203.0.113.42" }
  ]
}
```

---

### List Domains

**`GET /api/domains`**

```json
{
  "items": [
    {
      "id": 1,
      "domain": "s.example.com",
      "is_default": true,
      "is_active": true,
      "description": "Default domain",
      "deleted_at": null,
      "created_at": "2026-01-17T08:22:13Z",
      "updated_at": "2026-01-17T08:22:13Z"
    }
  ]
}
```

---

### Create Domain

**`POST /api/domains`** → `201 Created`

```json
{ "domain": "links.example.com", "is_default": false, "description": "Secondary domain" }
```

---

### Update Domain

**`PATCH /api/domains/{id}`** → `200 OK`

All fields optional.

- `is_default: true` — atomically transfers the default flag from the current default
- `is_default: false` — rejected (400); set another domain as default instead
- `description: null` — clears the description

```json
{ "domain": "new-name.example.com", "is_active": false, "is_default": true, "description": null }
```

---

### Delete Domain

**`DELETE /api/domains/{id}`** → `204 No Content`

Soft-delete. After deletion:
- The domain disappears from `GET /api/domains`
- Redirects via this domain return `410 Gone`
- New links cannot be created for it

Rejected (400) if the domain is the current default or has existing links.

---

### Service Health

**`GET /health`**

Response `200 OK` (healthy) or `503 Service Unavailable` (degraded):

```json
{
  "status": "healthy",
  "version": "0.1.0",
  "checks": {
    "database": { "status": "ok", "message": "Connected, default domain: s.example.com" },
    "click_queue": { "status": "ok", "message": "Capacity: 10000" },
    "cache": { "status": "ok", "message": "Redis connected" }
  }
}
```

---

## Authentication

All API endpoints except `GET /{code}` require a Bearer token:

```http
Authorization: Bearer <your-token>
```

### Creating Tokens

**CLI (recommended):**

```bash
cargo run --bin admin -- create-token "My App"
# Output: Token created: <random-secure-token>
```

**SQL (manual setup):**

```sql
-- TOKEN_SIGNING_SECRET must match the value in your .env
INSERT INTO api_tokens (name, token_hash)
VALUES ('My App', encode(hmac('your-secret-token', 'YOUR_SIGNING_SECRET', 'sha256'), 'hex'));
```

---

## Error Handling

All errors return a unified JSON structure:

```json
{
  "error": {
    "code": "not_found",
    "message": "Short link not found",
    "details": { "code": "unknown123" }
  }
}
```

| HTTP Status | Error Code         | When |
|:------------|:-------------------|:-----|
| 400         | `validation_error` | Invalid input data |
| 400         | `bad_request`      | Business rule violation |
| 401         | `unauthorized`     | Missing or invalid token |
| 404         | `not_found`        | Resource not found |
| 409         | `conflict`         | Duplicate resource (e.g., custom code already taken) |
| 410         | `gone`             | Link deleted/expired, or domain soft-deleted |
| 500         | `internal_error`   | Server error |

---

## Rate Limiting

IP-based, powered by tower_governor. When running behind a reverse proxy set `BEHIND_PROXY=true` to read the client IP from `X-Forwarded-For` / `X-Real-IP`.

| Endpoints | Limit | Burst |
|:----------|:-----:|:-----:|
| `GET /{code}` (redirect, public) | 2 req/s | 100 |
| All `/api/*` endpoints (protected) | 1 req/s | 10 |

Exceeding the limit returns `429 Too Many Requests`.

---

## Monitoring & Logging

### Logging

```bash
RUST_LOG=info cargo run          # important events only
RUST_LOG=debug cargo run         # include cache hits/misses
LOG_FORMAT=json cargo run        # structured JSON for log aggregators
```

### Metrics

Built-in Prometheus-compatible counters (exposed at `GET /metrics`):

| Metric | Description |
|:-------|:------------|
| `click_worker_received_total` | Click events received by the worker |
| `click_worker_processed_total` | Events successfully written to DB |
| `click_worker_failed_total` | Events that exhausted all retries |
| `click_worker_retried_total` | Total retry attempts |
| `database_errors_total{type}` | Database errors by type |

---

## Testing

```bash
cargo test             # all tests
cargo test --lib       # unit tests only (no database)
cargo test --tests     # integration tests only (requires PostgreSQL)
```

See [TESTING.md](TESTING.md) for details.

---

## CLI Tools

```bash
# Token management
cargo run --bin admin -- create-token "My App"
cargo run --bin admin -- list-tokens
cargo run --bin admin -- revoke-token <token_id>

# Domain management
cargo run --bin admin -- add-domain "short.link" --default
cargo run --bin admin -- list-domains
```

---

## Database Schema

**`domains`**

| Column | Type | Notes |
|:-------|:-----|:------|
| `id` | `BIGSERIAL` | PK |
| `domain` | `TEXT` | Unique |
| `is_default` | `BOOLEAN` | Only one can be true |
| `is_active` | `BOOLEAN` | |
| `description` | `TEXT` | Nullable |
| `deleted_at` | `TIMESTAMPTZ` | Nullable; soft-delete marker |
| `created_at` | `TIMESTAMPTZ` | |
| `updated_at` | `TIMESTAMPTZ` | |

**`links`**

| Column | Type | Notes |
|:-------|:-----|:------|
| `id` | `BIGSERIAL` | PK |
| `code` | `TEXT` | Unique per domain |
| `long_url` | `TEXT` | |
| `normalized_url` | `TEXT` | For deduplication |
| `domain_id` | `BIGINT` | FK → domains |
| `permanent` | `BOOLEAN` | 301 vs 307 redirect |
| `expires_at` | `TIMESTAMPTZ` | Nullable |
| `deleted_at` | `TIMESTAMPTZ` | Nullable; soft-delete marker |
| `created_at` | `TIMESTAMPTZ` | |

Unique constraints: `(code, domain_id)` and `(normalized_url, domain_id)`.

**`link_clicks`**

| Column | Type | Notes |
|:-------|:-----|:------|
| `id` | `BIGSERIAL` | PK |
| `link_id` | `BIGINT` | FK → links CASCADE |
| `clicked_at` | `TIMESTAMPTZ` | |
| `ip` | `INET` | Nullable |
| `user_agent` | `TEXT` | Nullable |
| `referer` | `TEXT` | Nullable |

**`api_tokens`**

| Column | Type | Notes |
|:-------|:-----|:------|
| `id` | `BIGSERIAL` | PK |
| `name` | `TEXT` | Human-readable label |
| `token_hash` | `TEXT` | HMAC-SHA256 of the raw token |
| `created_at` | `TIMESTAMPTZ` | |
| `last_used_at` | `TIMESTAMPTZ` | Updated on each authenticated request |
| `revoked_at` | `TIMESTAMPTZ` | Nullable; revoked tokens are rejected |

---

## Development

### Database Migrations

```bash
sqlx migrate add create_new_table   # create new migration file
sqlx migrate run                    # apply pending migrations
sqlx migrate revert                 # revert last migration

# Regenerate .sqlx/ for offline compile-time SQL checking
cargo sqlx prepare -- --bin url-shortener
```

### Code Quality

```bash
cargo fmt
cargo clippy -- -D warnings
cargo doc --open --no-deps
```

---

## License

MIT License — see [LICENSE](LICENSE) for details.

---

**Made with Rust 🦀**
