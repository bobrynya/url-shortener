//! Click entity representing a single redirect event.

use chrono::{DateTime, Utc};

/// A click event recorded when a shortened link is accessed.
///
/// Captures metadata about each redirect for analytics purposes, including
/// client information (user agent, referrer) and network details (IP address).
#[derive(Debug, Clone)]
pub struct Click {
    #[allow(dead_code)]
    pub id: i64,
    #[allow(dead_code)]
    pub link_id: i64,
    pub clicked_at: DateTime<Utc>,
    pub user_agent: Option<String>,
    pub referer: Option<String>,
    pub ip: Option<String>,
}

impl Click {
    /// Creates a new Click instance.
    ///
    /// All metadata fields are optional to handle cases where client information
    /// is unavailable or privacy settings restrict data collection.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let click = Click::new(
    ///     1,
    ///     42,
    ///     Utc::now(),
    ///     Some("Mozilla/5.0".to_string()),
    ///     Some("https://google.com".to_string()),
    ///     Some("192.168.1.1".to_string()),
    /// );
    /// ```
    pub fn new(
        id: i64,
        link_id: i64,
        clicked_at: DateTime<Utc>,
        user_agent: Option<String>,
        referer: Option<String>,
        ip: Option<String>,
    ) -> Self {
        Self {
            id,
            link_id,
            clicked_at,
            user_agent,
            referer,
            ip,
        }
    }
}

/// Input data for recording a new click event.
///
/// Used when logging a redirect. The `link_id` must reference an existing link,
/// and the timestamp is automatically set by the database.
#[derive(Debug, Clone)]
pub struct NewClick {
    pub link_id: i64,
    pub user_agent: Option<String>,
    pub referer: Option<String>,
    pub ip: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_click_creation_with_all_fields() {
        let now = Utc::now();
        let click = Click::new(
            1,
            42,
            now,
            Some("Mozilla/5.0".to_string()),
            Some("https://google.com".to_string()),
            Some("192.168.1.1".to_string()),
        );

        assert_eq!(click.id, 1);
        assert_eq!(click.link_id, 42);
        assert_eq!(click.clicked_at, now);
        assert_eq!(click.user_agent, Some("Mozilla/5.0".to_string()));
        assert_eq!(click.referer, Some("https://google.com".to_string()));
        assert_eq!(click.ip, Some("192.168.1.1".to_string()));
    }

    #[test]
    fn test_click_creation_minimal() {
        let now = Utc::now();
        let click = Click::new(1, 10, now, None, None, None);

        assert_eq!(click.link_id, 10);
        assert!(click.user_agent.is_none());
        assert!(click.referer.is_none());
        assert!(click.ip.is_none());
    }

    #[test]
    fn test_new_click_creation() {
        let new_click = NewClick {
            link_id: 99,
            user_agent: Some("Chrome/120".to_string()),
            referer: None,
            ip: Some("10.0.0.1".to_string()),
        };

        assert_eq!(new_click.link_id, 99);
        assert!(new_click.user_agent.is_some());
        assert!(new_click.referer.is_none());
        assert!(new_click.ip.is_some());
    }
}
