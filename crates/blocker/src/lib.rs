//! Ad and tracker filtering, resolved before the first packet leaves.
//!
//! WKWebView gives native code no hook for inspecting http(s) requests —
//! `WKURLSchemeHandler` explicitly refuses those schemes, and there is no
//! supported way around it. A Rust-side filter would mean bouncing every
//! request out of WebKit's network process and back, which costs far more
//! than the ads do.
//!
//! So the work runs the other way around. This crate parses Adblock Plus
//! syntax with Brave's `adblock` engine and lowers it into Apple's
//! content-blocking JSON. WebKit compiles that into bytecode it evaluates
//! *inside* the network process: blocked requests never open a socket, and
//! the per-request cost never crosses into Rust at all. We pay the parse
//! once at startup — and usually not even then, see [`ruleset_id`].

use adblock::lists::{FilterSet, ParseOptions, RuleTypes};

/// The list shipped in the binary. See `rules/base.txt` for the rationale
/// behind what is and is not in it.
pub const BUILTIN_RULES: &str = include_str!("../rules/base.txt");

#[derive(Debug)]
pub enum Error {
    /// `adblock` refuses to lower a `FilterSet` that was not built in debug
    /// mode, because it needs the original rule text to report failures.
    Lowering,
    Serialize(serde_json::Error),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Lowering => write!(f, "could not lower filters to content-blocking rules"),
            Error::Serialize(e) => write!(f, "could not serialize content-blocking rules: {e}"),
        }
    }
}

impl std::error::Error for Error {}

/// A compiled ruleset, ready to hand to `WKContentRuleListStore`.
pub struct Ruleset {
    /// Apple content-blocking JSON.
    pub json: String,
    /// Stable identifier derived from the source text. WebKit keys its
    /// on-disk bytecode cache off this, so an unchanged list skips
    /// compilation entirely on the next launch.
    pub id: String,
    /// How many rules survived lowering. Not every ABP filter has a
    /// content-blocking equivalent — cosmetic filters with complex
    /// selectors, `$csp`, and anything needing request bodies are dropped.
    pub rule_count: usize,
}

/// Lower the built-in list.
pub fn builtin() -> Result<Ruleset, Error> {
    compile(&[BUILTIN_RULES])
}

/// Lower an arbitrary set of Adblock Plus lists into one ruleset.
///
/// Sources are concatenated in order; later lists can `@@`-except earlier
/// ones, which is why ordering is preserved rather than sorted.
pub fn compile(sources: &[&str]) -> Result<Ruleset, Error> {
    // `true` = debug mode, which `into_content_blocking` requires.
    let mut set = FilterSet::new(true);

    let opts = ParseOptions {
        // Cosmetic filters become `css-display-none` rules, which WebKit
        // applies without a style recalc storm — cheaper than the JS-side
        // element hiding a userscript blocker would need.
        rule_types: RuleTypes::All,
        ..Default::default()
    };

    for source in sources {
        set.add_filter_list((*source).to_string(), opts);
    }

    let (rules, _used) = set.into_content_blocking().map_err(|()| Error::Lowering)?;
    let rule_count = rules.len();
    let json = serde_json::to_string(&rules).map_err(Error::Serialize)?;

    Ok(Ruleset {
        id: ruleset_id(sources),
        json,
        rule_count,
    })
}

/// Content-addressed identifier for a set of lists.
///
/// FNV-1a rather than a real hash crate: this guards a cache, not a
/// signature, and one less dependency is one less thing to build.
pub fn ruleset_id(sources: &[&str]) -> String {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for source in sources {
        for byte in source.as_bytes() {
            hash ^= u64::from(*byte);
            hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
        }
    }
    format!("vel-rules-{hash:016x}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builtin_list_lowers_to_rules() {
        let set = builtin().expect("built-in list should lower cleanly");
        assert!(
            set.rule_count > 20,
            "expected the built-in list to survive lowering, got {} rules",
            set.rule_count
        );
        assert!(set.json.starts_with('['));
    }

    #[test]
    fn identifier_tracks_content() {
        assert_eq!(ruleset_id(&["a"]), ruleset_id(&["a"]));
        assert_ne!(ruleset_id(&["a"]), ruleset_id(&["b"]));
    }

    /// Regression guard for the failure mode that matters most: a filter
    /// broad enough to swallow the media stream itself. Blocking
    /// `googlevideo.com` or the Twitch CDN turns the browser into a
    /// very fast way to watch a spinner.
    #[test]
    fn no_rule_touches_media_delivery() {
        const NEVER_BLOCK: [&str; 4] = ["googlevideo.com", "ttvnw.net", "video-edge", "akamaized"];
        for line in BUILTIN_RULES.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('!') {
                continue;
            }
            for host in NEVER_BLOCK {
                assert!(
                    !line.contains(host),
                    "rule {line:?} would filter media delivery ({host})"
                );
            }
        }
    }
}
