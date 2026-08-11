//! What Vel remembers between launches about a validated licence.
//!
//! The cache is what makes the licence check invisible. Startup reads this
//! file and nothing else — no network call is on the path to the first
//! window — and the online re-check happens afterwards, in the background,
//! at most once every [`crate::config::REVALIDATE_AFTER_DAYS`].
//!
//! It is a plain text file on purpose. Someone who wants to forge it can, and
//! that is fine (see the module docs in `lib.rs`); meanwhile a supporter whose
//! licence is misbehaving can open it, read it, and tell you what it says.

use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::config;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Activation {
    /// The key this record is about. A different key invalidates the record.
    pub key: String,
    /// Device instance from Lemon Squeezy, needed to re-validate this machine.
    pub instance_id: Option<String>,
    /// Unix seconds when the server last confirmed the licence.
    pub validated_at: u64,
}

impl Activation {
    pub fn new(key: &str, instance_id: Option<String>) -> Self {
        Self {
            key: key.trim().to_string(),
            instance_id,
            validated_at: now(),
        }
    }

    pub fn matches(&self, key: &str) -> bool {
        self.key == key.trim()
    }

    /// Has this been confirmed recently enough to trust without re-asking?
    pub fn is_fresh(&self) -> bool {
        let age = now().saturating_sub(self.validated_at);
        age < config::REVALIDATE_AFTER_DAYS * 24 * 60 * 60
    }

    pub fn load() -> Option<Self> {
        let text = std::fs::read_to_string(path()?).ok()?;
        Self::parse(&text)
    }

    pub fn save(&self) -> std::io::Result<()> {
        let path = path().ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::NotFound, "no application support dir")
        })?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(path, self.render())
    }

    fn render(&self) -> String {
        format!(
            "key={}\ninstance={}\nvalidated_at={}\n",
            self.key,
            self.instance_id.as_deref().unwrap_or(""),
            self.validated_at
        )
    }

    fn parse(text: &str) -> Option<Self> {
        let mut key = None;
        let mut instance = None;
        let mut validated_at = None;
        for line in text.lines() {
            let Some((field, value)) = line.split_once('=') else {
                continue;
            };
            let value = value.trim();
            match field.trim() {
                "key" => key = Some(value.to_string()),
                "instance" => instance = (!value.is_empty()).then(|| value.to_string()),
                "validated_at" => validated_at = value.parse().ok(),
                _ => {}
            }
        }
        Some(Self {
            key: key?,
            instance_id: instance,
            // A record with no readable timestamp is treated as ancient
            // rather than discarded: the licence is probably still good, and
            // the next background check will settle it.
            validated_at: validated_at.unwrap_or(0),
        })
    }
}

/// Forget the cached activation.
pub fn clear() {
    if let Some(path) = path() {
        let _ = std::fs::remove_file(path);
    }
}

pub fn path() -> Option<PathBuf> {
    Some(crate::support_dir()?.join("activation"))
}

fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_secs())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_record_survives_a_round_trip() {
        let record = Activation::new("38b1460a-5104-4067-a91d-77b872934d51", Some("i-1".into()));
        let parsed = Activation::parse(&record.render()).expect("should parse");
        assert_eq!(parsed, record);
    }

    #[test]
    fn a_record_without_an_instance_round_trips_too() {
        let record = Activation::new("k", None);
        let parsed = Activation::parse(&record.render()).expect("should parse");
        assert_eq!(parsed.instance_id, None);
    }

    #[test]
    fn a_record_is_only_about_its_own_key() {
        let record = Activation::new("key-one", None);
        assert!(record.matches("key-one"));
        assert!(record.matches("  key-one\n"));
        assert!(!record.matches("key-two"));
    }

    #[test]
    fn freshness_expires() {
        let mut record = Activation::new("k", None);
        assert!(record.is_fresh());
        record.validated_at = now() - (config::REVALIDATE_AFTER_DAYS + 1) * 24 * 60 * 60;
        assert!(!record.is_fresh());
    }

    /// A truncated or hand-edited file must not panic or read as fresh.
    #[test]
    fn junk_is_handled() {
        assert!(Activation::parse("").is_none());
        assert!(Activation::parse("nonsense").is_none());
        let partial = Activation::parse("key=abc\n").expect("key alone is enough");
        assert_eq!(partial.validated_at, 0);
        assert!(!partial.is_fresh(), "an undated record must re-check");
    }
}
