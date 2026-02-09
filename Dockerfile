# ---------- chef stage ----------
FROM rust:1.93-bookworm AS chef
WORKDIR /app
RUN cargo install cargo-chef

# ---------- planner stage ----------
FROM chef AS planner
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo chef prepare --recipe-path recipe.json

# ---------- builder stage ----------
FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json

# Build only dependencies (cached)
RUN cargo chef cook --release --recipe-path recipe.json

# Copy actual code and resources
COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY migrations ./migrations
COPY .sqlx ./.sqlx

# Copy templates
RUN mkdir -p templates
COPY src/web/templates ./templates

# Build with offline mode
ENV SQLX_OFFLINE=true
RUN cargo build --release --bin url-shortener

# Strip binary to reduce size
RUN strip target/release/url-shortener

# ---------- runtime stage ----------
FROM debian:bookworm-slim AS runtime
WORKDIR /app

# Create non-root user
RUN groupadd -r appuser && useradd -r -g appuser appuser

# Install runtime dependencies
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    ca-certificates \
    curl \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Copy binary from builder
COPY --from=builder /app/target/release/url-shortener /app/url-shortener

# Copy runtime resources
COPY static /app/static

# Set ownership
RUN chown -R appuser:appuser /app

# Switch to non-root user
USER appuser

ENV LISTEN=0.0.0.0:8000
EXPOSE 8000

CMD ["/app/url-shortener"]
