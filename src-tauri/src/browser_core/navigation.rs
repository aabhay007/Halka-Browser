use url::Url;

pub struct NavigationManager;

impl NavigationManager {
    /// Parses input string into a target URL.
    /// If it looks like a valid URL or host (e.g. "google.com", "https://github.com", "localhost:3000"),
    /// it formats it as a URL. Otherwise, it performs a search using Google.
    pub fn parse_input_to_url(input: &str) -> String {
        let trimmed = input.trim();
        if trimmed.is_empty() {
            return "https://www.google.com".to_string();
        }

        // Direct valid URL check
        if let Ok(parsed) = Url::parse(trimmed) {
            if parsed.scheme() == "http" || parsed.scheme() == "https" || parsed.scheme() == "file" {
                return parsed.to_string();
            }
        }

        // Domain-like check (e.g., "github.com", "reddit.com/r/rust")
        if !trimmed.contains(' ') && (trimmed.contains('.') || trimmed.starts_with("localhost")) {
            let prefixed = format!("https://{}", trimmed);
            if let Ok(parsed) = Url::parse(&prefixed) {
                return parsed.to_string();
            }
        }

        // Search engine fallback (Google)
        let encoded_query = urlencoding::encode(trimmed);
        format!("https://www.google.com/search?q={}", encoded_query)
    }

    /// Extracts a friendly human-readable display title from a URL string.
    pub fn extract_display_title(url_str: &str) -> String {
        if let Ok(url) = Url::parse(url_str) {
            if let Some(host) = url.host_str() {
                let host_clean = host.strip_prefix("www.").unwrap_or(host);

                // Google Search query extraction
                if host_clean.contains("google.") && url.path().starts_with("/search") {
                    for (key, val) in url.query_pairs() {
                        if key == "q" && !val.trim().is_empty() {
                            return format!("{} - Google Search", val.trim());
                        }
                    }
                    return "Google Search".to_string();
                }

                // Well-known domain friendly titles
                if host_clean.contains("google.") {
                    return "Google".to_string();
                }
                if host_clean.contains("youtube.com") {
                    return "YouTube".to_string();
                }
                if host_clean.contains("github.com") {
                    let segments: Vec<&str> = url.path().trim_matches('/').split('/').filter(|s| !s.is_empty()).collect();
                    if segments.len() == 1 {
                        return format!("{} - GitHub", segments[0]);
                    } else if segments.len() >= 2 {
                        return format!("{}/{} - GitHub", segments[0], segments[1]);
                    }
                    return "GitHub".to_string();
                }
                if host_clean.contains("wikipedia.org") {
                    let segments: Vec<&str> = url.path().trim_matches('/').split('/').filter(|s| !s.is_empty()).collect();
                    if segments.len() >= 2 && segments[0] == "wiki" {
                        let article = segments[1].replace('_', " ");
                        return format!("{} - Wikipedia", article);
                    }
                    return "Wikipedia".to_string();
                }
                if host_clean.contains("reddit.com") {
                    return "Reddit".to_string();
                }
                if host_clean.contains("duckduckgo.com") {
                    for (key, val) in url.query_pairs() {
                        if key == "q" && !val.trim().is_empty() {
                            return format!("{} - DuckDuckGo", val.trim());
                        }
                    }
                    return "DuckDuckGo".to_string();
                }
                if host_clean.contains("bing.com") {
                    return "Bing".to_string();
                }
                if host_clean.contains("twitter.com") || host_clean == "x.com" {
                    return "X".to_string();
                }

                return host_clean.to_string();
            }
        }
        "New Tab".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_urls() {
        assert_eq!(
            NavigationManager::parse_input_to_url("https://github.com"),
            "https://github.com/"
        );
        assert_eq!(
            NavigationManager::parse_input_to_url("http://google.com"),
            "http://google.com/"
        );
    }

    #[test]
    fn test_domain_names() {
        assert_eq!(
            NavigationManager::parse_input_to_url("wikipedia.org"),
            "https://wikipedia.org/"
        );
    }

    #[test]
    fn test_search_queries() {
        assert_eq!(
            NavigationManager::parse_input_to_url("rust programming language"),
            "https://www.google.com/search?q=rust%20programming%20language"
        );
    }

    #[test]
    fn test_extract_display_title() {
        assert_eq!(
            NavigationManager::extract_display_title("https://www.google.com"),
            "Google"
        );
        assert_eq!(
            NavigationManager::extract_display_title("https://www.google.com/search?q=tauri+v2"),
            "tauri v2 - Google Search"
        );
        assert_eq!(
            NavigationManager::extract_display_title("https://github.com/rust-lang/rust"),
            "rust-lang/rust - GitHub"
        );
        assert_eq!(
            NavigationManager::extract_display_title("https://en.wikipedia.org/wiki/Rust_(programming_language)"),
            "Rust (programming language) - Wikipedia"
        );
    }
}
