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
}
