//! Rate limiting middleware using token bucket algorithm.
//!
//! Applies per-IP rate limits via [`tower_governor`]. Client IP is extracted by
//! [`SmartIpExtractor`] which supports deployments behind a reverse proxy.

use axum::extract::ConnectInfo;
use axum::http;
use governor::clock::QuantaInstant;
use governor::middleware::NoOpMiddleware;
use std::net::SocketAddr;
use std::sync::Arc;
use tower_governor::errors::GovernorError;
use tower_governor::key_extractor::KeyExtractor;
use tower_governor::{GovernorLayer, governor::GovernorConfigBuilder};

/// Extracts the client IP address for rate limiting.
///
/// When `behind_proxy` is `true`, checks `X-Forwarded-For` (leftmost entry) and
/// `X-Real-IP` headers before falling back to the peer socket address. This ensures
/// correct per-client limiting when the service runs behind nginx, Cloudflare, etc.
///
/// When `behind_proxy` is `false`, uses the peer socket address directly.
///
/// # Security
///
/// Only enable proxy-header mode (`behind_proxy = true`) when a trusted proxy is
/// guaranteed to set these headers, as they can otherwise be forged by clients.
#[derive(Clone)]
pub struct SmartIpExtractor {
    pub behind_proxy: bool,
}

impl KeyExtractor for SmartIpExtractor {
    type Key = String;

    fn extract<B>(&self, req: &http::Request<B>) -> Result<String, GovernorError> {
        if self.behind_proxy {
            // X-Forwarded-For: client, proxy1, proxy2 — take the leftmost (original client)
            if let Some(xff) = req.headers().get("x-forwarded-for")
                && let Ok(s) = xff.to_str()
                && let Some(ip) = s.split(',').next()
            {
                return Ok(ip.trim().to_string());
            }
            // X-Real-IP set by nginx
            if let Some(xri) = req.headers().get("x-real-ip")
                && let Ok(ip) = xri.to_str()
            {
                return Ok(ip.trim().to_string());
            }
        }

        req.extensions()
            .get::<ConnectInfo<SocketAddr>>()
            .map(|ci| ci.0.ip().to_string())
            .ok_or(GovernorError::UnableToExtractKey)
    }
}

fn build_layer(
    per_second: u64,
    burst_size: u32,
    behind_proxy: bool,
) -> GovernorLayer<SmartIpExtractor, NoOpMiddleware<QuantaInstant>, axum::body::Body> {
    let extractor = SmartIpExtractor { behind_proxy };
    let governor_conf = Arc::new(
        GovernorConfigBuilder::default()
            .per_second(per_second)
            .burst_size(burst_size)
            .key_extractor(extractor)
            .finish()
            .unwrap(),
    );
    GovernorLayer::new(governor_conf)
}

/// Rate limiter for public and redirect endpoints.
///
/// Limits: **2 req/s**, burst **100**. Exceeding the limit returns `429 Too Many Requests`.
///
/// See [`SmartIpExtractor`] for IP extraction behaviour.
pub fn layer(
    behind_proxy: bool,
) -> GovernorLayer<SmartIpExtractor, NoOpMiddleware<QuantaInstant>, axum::body::Body> {
    build_layer(2, 100, behind_proxy)
}

/// Stricter rate limiter for authenticated API endpoints.
///
/// Limits: **1 req/s**, burst **10**. Exceeding the limit returns `429 Too Many Requests`.
///
/// See [`SmartIpExtractor`] for IP extraction behaviour.
pub fn secure_layer(
    behind_proxy: bool,
) -> GovernorLayer<SmartIpExtractor, NoOpMiddleware<QuantaInstant>, axum::body::Body> {
    build_layer(1, 10, behind_proxy)
}
