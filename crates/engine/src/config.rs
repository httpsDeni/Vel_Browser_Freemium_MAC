//! Web view configuration.
//!
//! This is where nearly all of the browser's performance character is set.
//! Vel does not render anything itself — WebKit owns the frame loop and
//! VideoToolbox owns the decode — so "make video fast" means "hand WebKit a
//! configuration that lets it take the fast path it already has", then get
//! out of the way.

use objc2::rc::Retained;
use objc2::MainThreadMarker;
use objc2_foundation::NSString;
use objc2_web_kit::{
    WKAudiovisualMediaTypes, WKContentMode, WKInactiveSchedulingPolicy, WKUserContentController,
    WKWebViewConfiguration, WKWebpagePreferences, WKWebsiteDataStore,
};

/// Safari's user-agent suffix.
///
/// This is not cosmetic. WKWebView's default UA carries no `Version/… Safari/…`
/// token, and YouTube reads that token when it picks a delivery format: an
/// unrecognised WebKit gets served conservative VP9, while Safari gets the
/// stream WebKit can hand to VideoToolbox for fixed-function decode. On an
/// M-series chip that is the difference between a video that costs ~2% CPU
/// and one that pins a performance core. Twitch gates its HEVC/low-latency
/// paths on the same token.
const UA_SUFFIX: &str = "Version/18.5 Safari/605.1.15";

/// Whether a view keeps browsing state after the process exits.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Session {
    /// Cookies, local storage and the HTTP cache persist across launches.
    Persistent,
    /// Everything lives in memory and dies with the window.
    Private,
}

/// Process-wide state shared by every tab.
///
/// Only the data store lives here, and it is shared for two reasons: a login
/// in one tab should be a login in all of them, and a single shared HTTP
/// cache means the second YouTube tab does not refetch what the first
/// already has.
///
/// Note there is no process pool. `WKProcessPool` is deprecated precisely
/// because WebKit now decides process reuse itself — same-origin tabs land
/// in one WebContent process and idle ones are reclaimed under memory
/// pressure without being asked. Creating pools here would be ceremony with
/// no effect.
pub struct Host {
    store: Retained<WKWebsiteDataStore>,
    mtm: MainThreadMarker,
}

impl Host {
    pub fn new(session: Session, mtm: MainThreadMarker) -> Self {
        let store = match session {
            Session::Persistent => unsafe { WKWebsiteDataStore::defaultDataStore(mtm) },
            Session::Private => unsafe { WKWebsiteDataStore::nonPersistentDataStore(mtm) },
        };
        Self { store, mtm }
    }

    pub fn mtm(&self) -> MainThreadMarker {
        self.mtm
    }

    /// Build a configuration for one tab.
    ///
    /// The user content controller is created per tab rather than shared,
    /// because the injected script carries the tab's id — see
    /// [`crate::script`]. Controllers are cheap; processes are not.
    pub fn configuration(&self) -> (Retained<WKWebViewConfiguration>, Retained<WKUserContentController>) {
        let mtm = self.mtm;
        let config = unsafe { WKWebViewConfiguration::new(mtm) };
        let content = unsafe { WKUserContentController::new(mtm) };

        unsafe {
            config.setWebsiteDataStore(&self.store);
            config.setUserContentController(&content);
            config.setApplicationNameForUserAgent(Some(&NSString::from_str(UA_SUFFIX)));

            // Autoplay. The empty set means "no media type needs a user
            // gesture", which is what makes a YouTube link start playing on
            // arrival instead of parking on a poster frame. Sites that
            // shouldn't autoplay are handled by the blocklist, not by
            // penalising every video on the web.
            config.setMediaTypesRequiringUserActionForPlayback(WKAudiovisualMediaTypes(0));
            config.setAllowsAirPlayForMediaPlayback(true);

            // Paint partial content as it arrives. Withholding the first
            // frame until layout settles trades real latency for a marginally
            // tidier load, which is the wrong trade on a video page.
            config.setSuppressesIncrementalRendering(false);

            // HSTS-preloaded hosts get upgraded before the plaintext request
            // is made, rather than after a redirect round trip.
            config.setUpgradeKnownHostsToHTTPS(true);

            let page = WKWebpagePreferences::new(mtm);
            // Desktop content mode: mobile YouTube caps resolution and drops
            // the 60fps ladder entirely.
            page.setPreferredContentMode(WKContentMode::Desktop);
            page.setAllowsContentJavaScript(true);
            config.setDefaultWebpagePreferences(Some(&page));

            let prefs = config.preferences();

            // The tab-suspension primitive. `Suspend` tells WebKit to stop
            // running JavaScript and doing layout for a web view that is idle
            // *and detached from the view hierarchy* — which is exactly the
            // state a background tab is put into. WebKit exempts views that
            // are playing media or still loading, so a backgrounded video
            // keeps its audio.
            prefs.setInactiveSchedulingPolicy(WKInactiveSchedulingPolicy::Suspend);

            // Fullscreen video, and the presentation-mode API that Vel's
            // picture-in-picture shortcut drives.
            prefs.setElementFullscreenEnabled(true);

            prefs.setFraudulentWebsiteWarningEnabled(true);
            // Pop-ups only on a real click, never on script initiative.
            prefs.setJavaScriptCanOpenWindowsAutomatically(false);
        }

        (config, content)
    }
}
