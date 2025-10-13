pub fn sanitize_html(input: &str) -> String {
    ammonia::clean(input)
}

pub fn sanitize_optional_html(input: Option<&str>) -> Option<String> {
    input.map(sanitize_html)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_html() {
        let input = "<script>alert('xss')</script>Hello World";
        let result = sanitize_html(input);
        assert!(!result.contains("<script>"));
        assert!(result.contains("Hello World"));
    }

    #[test]
    fn test_sanitize_html_removes_javascript() {
        let input = "<img src=x onerror=alert('xss')>";
        let result = sanitize_html(input);
        assert!(!result.contains("onerror"));
        assert!(!result.contains("alert"));
    }

    #[test]
    fn test_sanitize_html_allows_safe_tags() {
        let input = "<p>Hello <strong>World</strong></p>";
        let result = sanitize_html(input);
        assert!(result.contains("<p>"));
        assert!(result.contains("<strong>"));
        assert!(result.contains("</strong>"));
    }

    #[test]
    fn test_sanitize_optional_html_none() {
        let result = sanitize_optional_html(None);
        assert!(result.is_none());
    }

    #[test]
    fn test_sanitize_optional_html_some() {
        let input = Some("<script>alert('xss')</script>Hello");
        let result = sanitize_optional_html(input.as_deref());
        assert!(result.is_some());
        let sanitized = result.unwrap();
        assert!(!sanitized.contains("<script>"));
        assert!(sanitized.contains("Hello"));
    }
}