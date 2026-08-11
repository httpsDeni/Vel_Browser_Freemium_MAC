//! Deciding what the user meant when they typed into the one text field.
//!
//! A single field for URLs and searches is a guess, and the failure modes are
//! asymmetric: sending a private-looking URL to a search engine leaks it,
//! while treating a search as a hostname just fails visibly and harmlessly.
//! So the rules below only navigate when the input really looks like an
//! address, and search otherwise.

pub const HOME: &str = "https://duckduckgo.com/";
const SEARCH_PREFIX: &str = "https://duckduckgo.com/?q=";

/// Schemes we are willing to put in the address bar.
///
/// The omission that matters is `javascript:` — typing (or, far more likely,
/// pasting something someone else wrote) a `javascript:` URL would run it
/// against whatever page is loaded, which is the classic self-XSS delivery
/// mechanism. `data:` is out for the same reason: it renders attacker-chosen
/// markup under an origin the user cannot inspect.
const NAVIGABLE: [&str; 4] = ["https://", "http://", "file://", "about:"];

pub fn resolve(input: &str) -> String {
    let text = input.trim();
    if text.is_empty() {
        return HOME.to_string();
    }

    let lower = text.to_ascii_lowercase();
    if NAVIGABLE.iter().any(|s| lower.starts_with(s)) {
        return text.to_string();
    }

    // A scheme we do not navigate to is not an address; treat it as a query
    // rather than silently dropping it.
    if !has_scheme(&lower) && looks_like_host(text) {
        let scheme = if is_loopback(text) { "http://" } else { "https://" };
        return format!("{scheme}{text}");
    }

    format!("{SEARCH_PREFIX}{}", percent_encode(text))
}

/// Does this start with `something:` that could be a URL scheme?
fn has_scheme(lower: &str) -> bool {
    match lower.find(':') {
        // A colon inside the first label is far more likely a port
        // (`localhost:3000`) than a scheme, so require a non-digit after it.
        Some(i) if i > 0 => {
            let rest = &lower[i + 1..];
            lower[..i].chars().all(|c| c.is_ascii_alphanumeric() || c == '+' || c == '-' || c == '.')
                && !rest.starts_with(|c: char| c.is_ascii_digit())
        }
        _ => false,
    }
}

fn is_loopback(text: &str) -> bool {
    let host = host_part(text);
    host == "localhost" || host == "127.0.0.1" || host == "[::1]" || host == "0.0.0.0"
}

fn host_part(text: &str) -> &str {
    let end = text
        .find(['/', '?', '#'])
        .unwrap_or(text.len());
    let host = &text[..end];
    // Strip the port, but not the colons inside a bracketed IPv6 literal.
    match host.rfind(':') {
        Some(i) if !host.ends_with(']') && host[i + 1..].chars().all(|c| c.is_ascii_digit()) => {
            &host[..i]
        }
        _ => host,
    }
}

fn looks_like_host(text: &str) -> bool {
    if text.contains(char::is_whitespace) {
        return false;
    }
    if is_loopback(text) {
        return true;
    }

    let host = host_part(text);
    let Some((_, tld)) = host.rsplit_once('.') else {
        return false;
    };

    // "1.5" or "version 2.0" should search; "example.com" should not. An
    // all-numeric last label is only an address if the whole thing is an IPv4
    // literal.
    if tld.chars().all(|c| c.is_ascii_digit()) {
        return is_ipv4(host);
    }

    tld.len() >= 2 && tld.chars().all(|c| c.is_ascii_alphabetic())
}

fn is_ipv4(host: &str) -> bool {
    let mut parts = 0;
    for part in host.split('.') {
        parts += 1;
        if part.is_empty() || part.len() > 3 || !part.chars().all(|c| c.is_ascii_digit()) {
            return false;
        }
        if part.parse::<u16>().map_or(true, |n| n > 255) {
            return false;
        }
    }
    parts == 4
}

/// Percent-encode a search query.
///
/// Hand-rolled rather than pulling in a URL crate: the whole encoder is
/// twelve lines and this is the only place the browser builds a URL from
/// untrusted text.
fn percent_encode(text: &str) -> String {
    let mut out = String::with_capacity(text.len() + 8);
    for byte in text.as_bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(*byte as char)
            }
            b' ' => out.push('+'),
            _ => out.push_str(&format!("%{byte:02X}")),
        }
    }
    out
}

/// Shorten a URL for display in the address bar.
///
/// Showing the full URL of a YouTube watch page means showing forty
/// characters of tracking parameters, which pushes the part the user
/// actually reads — the origin — off the left edge.
pub fn for_display(url: &str) -> String {
    let trimmed = url
        .strip_prefix("https://")
        .unwrap_or(url)
        .trim_end_matches('/');
    trimmed.strip_prefix("www.").unwrap_or(trimmed).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plain_hostnames_navigate() {
        assert_eq!(resolve("youtube.com"), "https://youtube.com");
        assert_eq!(resolve("www.twitch.tv/videos"), "https://www.twitch.tv/videos");
        assert_eq!(resolve("  example.co.uk  "), "https://example.co.uk");
    }

    #[test]
    fn explicit_schemes_pass_through() {
        assert_eq!(resolve("http://example.com"), "http://example.com");
        assert_eq!(resolve("about:blank"), "about:blank");
    }

    #[test]
    fn loopback_stays_plaintext() {
        assert_eq!(resolve("localhost:3000"), "http://localhost:3000");
        assert_eq!(resolve("127.0.0.1:8080/x"), "http://127.0.0.1:8080/x");
    }

    #[test]
    fn prose_searches() {
        assert_eq!(resolve("rust lifetimes"), "https://duckduckgo.com/?q=rust+lifetimes");
        // A bare version number is not a hostname.
        assert!(resolve("1.5").starts_with(SEARCH_PREFIX));
        assert!(resolve("what is 2.0").starts_with(SEARCH_PREFIX));
    }

    /// The one that has security consequences: a `javascript:` URL in the
    /// address bar must never be navigated to.
    #[test]
    fn script_urls_are_never_navigated() {
        for hostile in [
            "javascript:alert(document.cookie)",
            "JavaScript:fetch('//evil.example')",
            "data:text/html,<script>alert(1)</script>",
        ] {
            let out = resolve(hostile);
            assert!(
                out.starts_with(SEARCH_PREFIX),
                "{hostile:?} resolved to {out:?} instead of a search"
            );
        }
    }

    #[test]
    fn queries_are_encoded() {
        assert_eq!(resolve("a&b=c"), "https://duckduckgo.com/?q=a%26b%3Dc");
    }

    #[test]
    fn display_drops_noise() {
        assert_eq!(for_display("https://www.youtube.com/"), "youtube.com");
        assert_eq!(for_display("http://localhost:3000"), "http://localhost:3000");
    }
}
