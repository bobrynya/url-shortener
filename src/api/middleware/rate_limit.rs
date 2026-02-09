//! Rate limiting middleware using token bucket algorithm.

use governor::clock::QuantaInstant;
use governor::middleware::NoOpMiddleware;
use std::sync::Arc;
use tower_governor::{
    GovernorLayer, governor::GovernorConfigBuilder, key_extractor::PeerIpKeyExtractor,
};

/// Creates a rate limiter for public endpoints.
///
/// # Limits
///
/// - **Rate**: 2 requests per second
/// - **Burst**: 100 requests
///
/// Requests exceeding the limit receive `429 Too Many Requests`.
///
/// # Key Extraction
///
/// Rate limits are applied per client IP address extracted from the
/// socket peer address.
///
/// # Example
///
/// ```rust,ignore
/// let app = Router::new()
///     .route("/shorten", post(shorten_handler))
///     .layer(rate_limit::layer());
/// ```
pub fn layer() -> GovernorLayer<PeerIpKeyExtractor, NoOpMiddleware<QuantaInstant>, axum::body::Body>
{
    let governor_conf = Arc::new(
        GovernorConfigBuilder::default()
            .per_second(2)
            .burst_size(100)
            .finish()
            .unwrap(),
    );

    GovernorLayer::new(governor_conf)
}

/// Creates a stricter rate limiter for authenticated endpoints.
///
/// # Limits
///
/// - **Rate**: 1 request per second
/// - **Burst**: 10 requests
///
/// Used for sensitive operations like API token management or admin endpoints.
///
/// # Example
///
/// ```rust,ignore
/// let secure_routes = Router::new()
///     .route("/admin/tokens", post(create_token_handler))
///     .layer(rate_limit::secure_layer());
/// ```
pub fn secure_layer()
-> GovernorLayer<PeerIpKeyExtractor, NoOpMiddleware<QuantaInstant>, axum::body::Body> {
    let governor_conf = Arc::new(
        GovernorConfigBuilder::default()
            .per_second(1)
            .burst_size(10)
            .finish()
            .unwrap(),
    );

    GovernorLayer::new(governor_conf)
}
