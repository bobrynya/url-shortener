//! HTTP request/response tracing middleware.

use tower_http::LatencyUnit;
use tower_http::trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer};
use tracing::Level;

/// Creates a tracing middleware for HTTP requests.
///
/// # Logging Behavior
///
/// **On Request:**
/// - Creates a span at `INFO` level with:
///   - HTTP method
///   - URI path
///   - HTTP version
///
/// **On Response:**
/// - Logs at `INFO` level with:
///   - Status code
///   - Latency in milliseconds
///
/// # Example Logs
///
/// ```text
/// INFO request{method=POST uri=/api/shorten version=HTTP/1.1}: Processing request
/// INFO request{method=POST uri=/api/shorten version=HTTP/1.1}: Response 200 OK in 12ms
/// ```
///
/// # Integration
///
/// ```rust,ignore
/// let app = Router::new()
///     .nest("/api", api_routes())
///     .layer(tracing::layer());
/// ```
pub fn layer()
-> TraceLayer<tower_http::classify::SharedClassifier<tower_http::classify::ServerErrorsAsFailures>>
{
    TraceLayer::new_for_http()
        .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
        .on_response(
            DefaultOnResponse::new()
                .level(Level::INFO)
                .latency_unit(LatencyUnit::Millis),
        )
}
