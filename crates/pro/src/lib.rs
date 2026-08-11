//! Supporter features, and the one place the free/paid line is drawn.
//!
//! # How the separation is enforced
//!
//! Not by discipline — by the dependency graph. `vel-app` depends on this
//! crate and **not** on `vel-blocker`; this crate owns that dependency and
//! only hands the ruleset over when the entitlement allows it. So a free
//! build cannot reach the blocking engine even by accident: there is no path
//! to it that does not pass through [`Entitlements`]. To move a feature
//! across the line, change [`required_tier`] — that function is the whole
//! policy.
//!
//! # Two ways to become a supporter
//!
//! - **Lemon Squeezy** mints a licence key on purchase and can be asked about
//!   it afterwards, so those keys are checked against the API — see
//!   [`lemonsqueezy`] and [`refresh`].
//! - **GitHub Sponsors** has no licence API, so sponsor keys are issued by
//!   hand with the `keygen` example and carry only a checksum.
//!
//! # What this is not
//!
//! It is not copy protection, and it should not be described as such. Vel is
//! MIT-licensed and the source is public: anyone can delete these checks and
//! rebuild in under a minute, and that is a legitimate thing for them to do
//! with MIT code. A donation model works because people who get value from
//! something choose to pay for it, not because they cannot avoid it. The
//! checks here exist to make unlocking *deliberate and pleasant for people
//! who paid* — they catch typos and refunds; they stop nobody.
//!
//! Two consequences follow, and both are load-bearing:
//!
//! - **No credential ships in the binary.** The Lemon Squeezy licence
//!   endpoints need no API key. Do not add one later: it would be readable in
//!   the binary, and a leaked Lemon Squeezy key exposes your orders and
//!   customers.
//! - **Being offline never costs anyone anything.** Only an authoritative
//!   refusal downgrades a tier; an unreachable server leaves it alone.

use std::path::PathBuf;

pub mod cache;
pub mod config;
pub mod key;
pub mod lemonsqueezy;

pub use config::{CHECKOUT_URL, SPONSORS_URL};
pub use key::{classify, make_sponsor_key, verify_sponsor_key, Source};
pub use vel_blocker::Ruleset;

/// Everything that sits behind the line.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Feature {
    /// The built-in ad and tracker blocklist.
    ContentBlocking,
    /// Loading your own Adblock Plus lists on top of the built-in one.
    CustomFilters,
    /// Detaching and then discarding cold tabs to give memory back.
    MemorySaver,
    /// The picture-in-picture shortcut.
    PictureInPicture,
}

impl Feature {
    pub const ALL: [Feature; 4] = [
        Feature::ContentBlocking,
        Feature::CustomFilters,
        Feature::MemorySaver,
        Feature::PictureInPicture,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Feature::ContentBlocking => "Ad & tracker blocking",
            Feature::CustomFilters => "Custom filter lists",
            Feature::MemorySaver => "Memory saver",
            Feature::PictureInPicture => "Picture in Picture",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub enum Tier {
    /// A complete browser: tabs, shortcuts, hardware-decoded video, the lot.
    /// Nothing here is crippled — the free tier is the browser.
    #[default]
    Free,
    /// Someone donated.
    Supporter,
}

/// **The policy.** Everything else in this crate is plumbing.
///
/// Moving a feature between tiers means editing this function and nothing
/// else. Note what is deliberately *not* here: page rendering, tabs,
/// shortcuts, the address bar, hardware video decode. A free Vel is a browser
/// someone would choose to use; if the free tier were annoying enough to
/// pressure people into paying, the donations would be extracted rather than
/// given, and this is not that kind of project.
pub fn required_tier(feature: Feature) -> Tier {
    match feature {
        Feature::ContentBlocking
        | Feature::CustomFilters
        | Feature::MemorySaver
        | Feature::PictureInPicture => Tier::Supporter,
    }
}

/// What this copy of Vel is allowed to do.
#[derive(Debug, Clone, Copy, Default)]
pub struct Entitlements {
    tier: Tier,
}

impl Entitlements {
    /// Work out the tier without touching the network.
    ///
    /// Startup calls this, so it reads only local files. A Lemon Squeezy key
    /// grants nothing until [`refresh`] has confirmed it at least once on this
    /// machine; after that the cached answer is trusted, including when it is
    /// stale, so that being offline never takes features away.
    pub fn load() -> Self {
        let Some(key) = stored_key() else {
            return Self::default();
        };
        let tier = match classify(&key) {
            // Nothing to check against — GitHub has no licence API.
            Some(Source::Sponsor) => Tier::Supporter,
            Some(Source::LemonSqueezy) => match cache::Activation::load() {
                Some(record) if record.matches(&key) => Tier::Supporter,
                _ => Tier::Free,
            },
            None => Tier::Free,
        };
        Self { tier }
    }

    pub fn with_tier(tier: Tier) -> Self {
        Self { tier }
    }

    pub fn tier(self) -> Tier {
        self.tier
    }

    pub fn is_supporter(self) -> bool {
        self.tier >= Tier::Supporter
    }

    pub fn allows(self, feature: Feature) -> bool {
        self.tier >= required_tier(feature)
    }

    /// The blocklist — the only route to it from the application.
    ///
    /// Returns `None` for the free tier, and the caller simply runs
    /// unfiltered. This is why `vel-app` has no `vel-blocker` dependency of
    /// its own.
    pub fn ruleset(self) -> Option<Result<Ruleset, vel_blocker::Error>> {
        self.allows(Feature::ContentBlocking)
            .then(vel_blocker::builtin)
    }

    /// Extra user lists, layered over the built-in one.
    pub fn ruleset_with(self, extra: &[&str]) -> Option<Result<Ruleset, vel_blocker::Error>> {
        if !self.allows(Feature::CustomFilters) {
            return self.ruleset();
        }
        let mut sources = vec![vel_blocker::BUILTIN_RULES];
        sources.extend_from_slice(extra);
        Some(vel_blocker::compile(&sources))
    }
}

/// How a background licence check ended.
#[derive(Debug, Clone)]
pub enum Refreshed {
    /// Nothing needed doing, or nothing could be done.
    Unchanged(String),
    /// The tier changed. The application should re-read its entitlements.
    Changed(Entitlements, String),
}

/// Bring a Lemon Squeezy licence up to date, in the background.
///
/// Call once after the window is on screen — never before, and never
/// blocking. `on_done` runs on the main thread.
///
/// The state machine is small but the edges matter:
///
/// - Sponsor key, or no key: nothing to ask anybody, returns immediately.
/// - Licence not yet seen on this machine: activate, which is what makes the
///   product's activation limit mean anything.
/// - Licence seen and fresh: nothing to do.
/// - Licence seen but stale: validate.
///
/// A refusal clears the cache and downgrades. An unreachable server changes
/// nothing at all.
pub fn refresh(on_done: impl Fn(Refreshed) + 'static) {
    let Some(key) = stored_key() else {
        on_done(Refreshed::Unchanged("no supporter key".into()));
        return;
    };

    match classify(&key) {
        Some(Source::Sponsor) => {
            on_done(Refreshed::Unchanged("sponsor key".into()));
            return;
        }
        None => {
            on_done(Refreshed::Unchanged("key not recognised".into()));
            return;
        }
        Some(Source::LemonSqueezy) => {}
    }

    if !config::is_configured() {
        on_done(Refreshed::Unchanged(
            "Lemon Squeezy not configured in this build".into(),
        ));
        return;
    }

    let record = cache::Activation::load().filter(|r| r.matches(&key));
    if record.as_ref().is_some_and(cache::Activation::is_fresh) {
        on_done(Refreshed::Unchanged("licence still fresh".into()));
        return;
    }

    let settle = {
        let key = key.clone();
        move |verdict: lemonsqueezy::Verdict| {
            if verdict.granted {
                let record = cache::Activation::new(&key, verdict.instance_id.clone());
                if let Err(e) = record.save() {
                    on_done(Refreshed::Unchanged(format!("could not save licence: {e}")));
                    return;
                }
                on_done(Refreshed::Changed(
                    Entitlements::with_tier(Tier::Supporter),
                    verdict.detail.clone(),
                ));
            } else if verdict.authoritative {
                // The server actually said no — a refund, a revoked key, a
                // licence for another product. Forget it.
                cache::clear();
                on_done(Refreshed::Changed(
                    Entitlements::with_tier(Tier::Free),
                    verdict.detail.clone(),
                ));
            } else {
                // Could not reach Lemon Squeezy. Keep whatever we had.
                on_done(Refreshed::Unchanged(format!(
                    "licence not re-checked: {}",
                    verdict.detail
                )));
            }
        }
    };

    match record {
        Some(record) => lemonsqueezy::validate(&key, record.instance_id.as_deref(), settle),
        None => lemonsqueezy::activate(&key, settle),
    }
}

/// Where to send somebody who wants to become a supporter.
///
/// Prefers Lemon Squeezy when it is configured, because that path issues a key
/// automatically the moment they pay. GitHub Sponsors needs you to send one by
/// hand, so it is the fallback rather than the default — and it is what an
/// unconfigured build offers, since the placeholder checkout link goes nowhere.
pub fn funding_url() -> &'static str {
    if config::is_configured() {
        config::CHECKOUT_URL
    } else {
        config::SPONSORS_URL
    }
}

/// The supporter key, as typed into the file.
pub fn stored_key() -> Option<String> {
    let text = std::fs::read_to_string(key_path()?).ok()?;
    let key = text.trim().to_string();
    (!key.is_empty()).then_some(key)
}

/// Where a supporter key lives.
pub fn key_path() -> Option<PathBuf> {
    Some(support_dir()?.join("supporter.key"))
}

pub(crate) fn support_dir() -> Option<PathBuf> {
    let home = std::env::var_os("HOME")?;
    Some(PathBuf::from(home).join("Library/Application Support/Vel"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn free_tier_gets_the_browser_but_not_the_extras() {
        let free = Entitlements::default();
        assert_eq!(free.tier(), Tier::Free);
        for feature in Feature::ALL {
            assert!(!free.allows(feature), "{feature:?} should need a supporter");
        }
        assert!(free.ruleset().is_none());
    }

    #[test]
    fn supporters_get_everything() {
        let paid = Entitlements::with_tier(Tier::Supporter);
        for feature in Feature::ALL {
            assert!(paid.allows(feature), "{feature:?} should be unlocked");
        }
        assert!(paid.ruleset().is_some());
    }

    #[test]
    fn custom_filters_fall_back_rather_than_failing() {
        // A free user asking for custom lists gets no lists, not an error;
        // a supporter gets theirs layered on the built-in one.
        assert!(Entitlements::default().ruleset_with(&["||x.test^"]).is_none());
        let paid = Entitlements::with_tier(Tier::Supporter);
        let set = paid.ruleset_with(&["||x.test^"]).expect("should build");
        assert!(set.expect("should compile").rule_count > 20);
    }
}
