use url::Url;

pub fn normalize_url(input: &str) -> Result<String, String> {
    let mut url = Url::parse(input).map_err(|e| format!("Invalid URL: {e}"))?;

    match url.scheme() {
        "http" | "https" => {}
        _ => return Err("Only http/https URLs are allowed".to_string()),
    }

    if let Some(host) = url.host_str() {
        let host_lc = host.to_ascii_lowercase();
        url.set_host(Some(&host_lc))
            .map_err(|_| "Failed to set host".to_string())?;
    }

    url.set_fragment(None);

    let is_default_port = matches!(
        (url.scheme(), url.port()),
        ("http", Some(80)) | ("https", Some(443))
    );
    if is_default_port {
        url.set_port(None)
            .map_err(|_| "Failed to drop port".to_string())?;
    }

    Ok(url.to_string())
}
