//! The Objective-C face of the application.
//!
//! One class plays every delegate role — application, window, navigation,
//! script messages — and every menu action targets it. That is not laziness:
//! each extra delegate class is another object with another strong reference
//! into the tab graph, and this browser's whole memory argument rests on
//! being able to drop a tab and have it actually go away. One object with one
//! `RefCell` is a graph you can reason about.

use std::cell::RefCell;

use objc2::rc::Retained;
use objc2::runtime::{AnyObject, ProtocolObject};
use objc2::{define_class, msg_send, sel, DefinedClass, MainThreadMarker, MainThreadOnly, Message};
use objc2_app_kit::{NSApplication, NSApplicationDelegate, NSWindowDelegate};
use objc2_foundation::{NSNotification, NSObject, NSObjectProtocol, NSString, NSTimer};
use objc2_web_kit::{
    WKNavigation, WKNavigationAction, WKNavigationDelegate, WKScriptMessage, WKScriptMessageHandler,
    WKUIDelegate, WKUserContentController, WKWebView, WKWebViewConfiguration, WKWindowFeatures,
};
use vel_engine::script::{self, PageEvent};

use crate::browser::{Browser, Wiring};
use crate::hud::Actions;
use vel_pro::{Entitlements, Feature};
use crate::{menu, omnibox};

/// How often the discarder looks for cold tabs.
///
/// Coarse on purpose. The sweep is O(tabs) and the threshold it enforces is
/// measured in minutes, so checking more often would only burn wakeups —
/// and a timer that fires constantly is exactly the kind of background cost
/// this browser is supposed to avoid.
const SWEEP_INTERVAL: f64 = 60.0;

pub struct Ivars {
    browser: RefCell<Option<Browser>>,
    /// URL to open at launch, taken from the command line.
    start_url: String,
}

define_class!(
    #[unsafe(super(NSObject))]
    #[thread_kind = MainThreadOnly]
    #[name = "VelDelegate"]
    #[ivars = Ivars]
    pub struct Delegate;

    unsafe impl NSObjectProtocol for Delegate {}

    unsafe impl NSApplicationDelegate for Delegate {
        #[unsafe(method(applicationDidFinishLaunching:))]
        fn did_finish_launching(&self, _notification: &NSNotification) {
            let mtm = self.mtm();
            let app = NSApplication::sharedApplication(mtm);
            let entitlements = Entitlements::load();
            menu::install(&app, self.as_any(), entitlements, mtm);

            let browser = Browser::new(
                ProtocolObject::from_ref(self),
                self.as_any(),
                Actions {
                    submit: sel!(submitAddress:),
                    back: sel!(goBackPage:),
                    forward: sel!(goForwardPage:),
                    new_tab: sel!(newTab:),
                    select_tab: sel!(selectTab:),
                },
                mtm,
            );

            // Start the blocklist compiling before the first tab exists.
            // Tabs created in the meantime register with `Rules` and have the
            // list applied the moment it lands, so nothing waits on this.
            //
            // `ruleset()` is `None` on the free tier — the application has no
            // dependency on the blocking engine at all, only on `vel-pro`,
            // which decides whether to hand it over.
            load_blocklist(browser.rules(), entitlements, mtm);

            browser.present();
            *self.ivars().browser.borrow_mut() = Some(browser);

            let start = self.ivars().start_url.clone();
            self.with(|browser, wiring| browser.open_tab(&start, wiring));

            unsafe {
                NSTimer::scheduledTimerWithTimeInterval_target_selector_userInfo_repeats(
                    SWEEP_INTERVAL,
                    self.as_any(),
                    sel!(sweepTabs:),
                    None,
                    true,
                );
            }

            // Licence check, after the window is up and never on the way to
            // it. Sponsor keys and fresh licences return without touching the
            // network; only a first activation or a stale one goes online.
            let this = self.retain();
            vel_pro::refresh(move |outcome| match outcome {
                vel_pro::Refreshed::Unchanged(why) => eprintln!("vel: {why}"),
                vel_pro::Refreshed::Changed(_, detail) => {
                    eprintln!("vel: licence changed — {detail}");
                    this.apply_entitlements();
                }
            });

            app.activate();
        }

        #[unsafe(method(applicationShouldTerminateAfterLastWindowClosed:))]
        fn terminate_after_last_window(&self, _app: &NSApplication) -> bool {
            true
        }
    }

    unsafe impl NSWindowDelegate for Delegate {
        #[unsafe(method(windowDidResize:))]
        fn window_did_resize(&self, _notification: &NSNotification) {
            // Layout only — no tab state changes — so this takes the cheap
            // borrow and never rebuilds the tab strip.
            if let Ok(slot) = self.ivars().browser.try_borrow() {
                if let Some(browser) = slot.as_ref() {
                    browser.layout();
                }
            }
        }
    }

    unsafe impl WKNavigationDelegate for Delegate {
        #[unsafe(method(webView:didFinishNavigation:))]
        fn did_finish_navigation(&self, view: &WKWebView, _navigation: Option<&WKNavigation>) {
            self.with(|browser, wiring| browser.page_changed(view, wiring));
        }

        #[unsafe(method(webView:didStartProvisionalNavigation:))]
        fn did_start_navigation(&self, view: &WKWebView, _navigation: Option<&WKNavigation>) {
            // Reflect the new URL in the address bar as soon as the
            // navigation commits, rather than when the page finishes.
            self.with(|browser, wiring| browser.page_changed(view, wiring));
        }
    }

    unsafe impl WKUIDelegate for Delegate {
        #[unsafe(method(webView:createWebViewWithConfiguration:forNavigationAction:windowFeatures:))]
        fn create_web_view(
            &self,
            _view: &WKWebView,
            _configuration: &WKWebViewConfiguration,
            action: &WKNavigationAction,
            _features: &WKWindowFeatures,
        ) -> Option<&WKWebView> {
            let request = unsafe { action.request() };
            if let Some(url) = request.URL() {
                if let Some(url_str) = url.absoluteString() {
                    let text = url_str.to_string();
                    if !text.is_empty() {
                        self.with(|browser, wiring| browser.open_tab(&text, wiring));
                    }
                }
            }
            None
        }
    }

    unsafe impl WKScriptMessageHandler for Delegate {
        #[unsafe(method(userContentController:didReceiveScriptMessage:))]
        fn did_receive_message(
            &self,
            _controller: &WKUserContentController,
            message: &WKScriptMessage,
        ) {
            let body = unsafe { message.body() };
            let Ok(text) = body.downcast::<NSString>() else {
                return;
            };
            match script::parse_event(&text.to_string()) {
                Some(PageEvent::Audible { tab, playing }) => {
                    self.with(|browser, wiring| browser.set_audible(tab, playing, wiring));
                }
                Some(PageEvent::StateChanged { tab }) => {
                    self.with(|browser, wiring| browser.refresh_by_id(tab, wiring));
                }
                None => {}
            }
        }
    }

    /// Menu and control actions.
    impl Delegate {
        #[unsafe(method(submitAddress:))]
        fn submit_address(&self, _sender: Option<&AnyObject>) {
            let Some(text) = self.with(|browser, _| browser.chrome_text()) else {
                return;
            };
            self.with(|browser, wiring| browser.navigate(&text, wiring));
        }

        #[unsafe(method(newTab:))]
        fn new_tab(&self, _sender: Option<&AnyObject>) {
            self.with(|browser, wiring| browser.open_tab(omnibox::HOME, wiring));
            self.with(|browser, _| browser.focus_address());
        }

        #[unsafe(method(closeTab:))]
        fn close_tab(&self, _sender: Option<&AnyObject>) {
            self.with(|browser, wiring| browser.close_active(wiring));
        }

        #[unsafe(method(selectTab:))]
        fn select_tab(&self, sender: Option<&AnyObject>) {
            let Some(sender) = sender else { return };
            let index: isize = unsafe { msg_send![sender, tag] };
            if index < 0 {
                return;
            }
            self.with(|browser, wiring| browser.select_tab(index as usize, wiring));
        }

        #[unsafe(method(selectNumberedTab:))]
        fn select_numbered_tab(&self, sender: Option<&AnyObject>) {
            let Some(sender) = sender else { return };
            // The menu item carries its number in its tag.
            let n: isize = unsafe { msg_send![sender, tag] };
            if n < 1 {
                return;
            }
            self.with(|browser, wiring| browser.select_numbered_tab(n as usize, wiring));
        }

        #[unsafe(method(nextTab:))]
        fn next_tab(&self, _sender: Option<&AnyObject>) {
            self.with(|browser, wiring| browser.cycle_tab(true, wiring));
        }

        #[unsafe(method(previousTab:))]
        fn previous_tab(&self, _sender: Option<&AnyObject>) {
            self.with(|browser, wiring| browser.cycle_tab(false, wiring));
        }

        #[unsafe(method(focusAddress:))]
        fn focus_address(&self, _sender: Option<&AnyObject>) {
            self.with(|browser, _| browser.focus_address());
        }

        #[unsafe(method(reloadPage:))]
        fn reload_page(&self, _sender: Option<&AnyObject>) {
            self.with(|browser, _| browser.with_active_page(|page| page.reload()));
        }

        #[unsafe(method(forceReload:))]
        fn force_reload(&self, _sender: Option<&AnyObject>) {
            self.with(|browser, _| browser.with_active_page(|page| page.reload_from_origin()));
        }

        #[unsafe(method(stopPage:))]
        fn stop_page(&self, _sender: Option<&AnyObject>) {
            self.with(|browser, _| browser.with_active_page(|page| page.stop()));
        }

        #[unsafe(method(goBackPage:))]
        fn go_back_page(&self, _sender: Option<&AnyObject>) {
            self.with(|browser, _| browser.with_active_page(|page| page.go_back()));
        }

        #[unsafe(method(goForwardPage:))]
        fn go_forward_page(&self, _sender: Option<&AnyObject>) {
            self.with(|browser, _| browser.with_active_page(|page| page.go_forward()));
        }

        #[unsafe(method(togglePip:))]
        fn toggle_pip(&self, _sender: Option<&AnyObject>) {
            self.with(|browser, wiring| {
                if browser.entitlements().allows(Feature::PictureInPicture) {
                    browser.with_active_page(|page| page.toggle_pip());
                } else {
                    browser.open_tab(vel_pro::funding_url(), wiring);
                }
            });
        }

        #[unsafe(method(splitScreen:))]
        fn split_screen(&self, _sender: Option<&AnyObject>) {
            let mtm = self.mtm();
            self.with(|browser, wiring| {
                if browser.entitlements().allows(Feature::DualView) {
                    browser.snap_left(mtm);
                } else {
                    browser.open_tab(vel_pro::funding_url(), wiring);
                }
            });
        }

        /// The two funding pages, and the only marketing anywhere in the app.
        #[unsafe(method(openSponsors:))]
        fn open_sponsors(&self, _sender: Option<&AnyObject>) {
            self.with(|browser, wiring| browser.open_tab(vel_pro::SPONSORS_URL, wiring));
        }

        #[unsafe(method(openCheckout:))]
        fn open_checkout(&self, _sender: Option<&AnyObject>) {
            self.with(|browser, wiring| browser.open_tab(vel_pro::CHECKOUT_URL, wiring));
        }

        #[unsafe(method(sweepTabs:))]
        fn sweep_tabs(&self, _sender: Option<&AnyObject>) {
            if let Some(swept) = self.with(|browser, wiring| browser.sweep(wiring)) {
                if swept.parked > 0 || swept.discarded > 0 {
                    eprintln!(
                        "vel: parked {}, discarded {} idle tab(s)",
                        swept.parked, swept.discarded
                    );
                }
            }
        }
    }
);

/// Hand the blocklist to WebKit, if this copy is entitled to one.
fn load_blocklist(rules: &vel_engine::Rules, entitlements: Entitlements, mtm: MainThreadMarker) {
    let rules = rules.clone();
    match entitlements.ruleset() {
        Some(Ok(set)) => {
            let count = set.rule_count;
            rules.load(&set.id, &set.json, mtm, move |outcome| match outcome {
                Ok(cached) => eprintln!(
                    "vel: {count} blocking rules {}",
                    if cached { "loaded from cache" } else { "compiled" }
                ),
                Err(e) => eprintln!("vel: blocklist unavailable, running unfiltered: {e}"),
            });
        }
        Some(Err(e)) => eprintln!("vel: could not build blocklist, running unfiltered: {e}"),
        None => eprintln!("vel: free tier — running without content blocking"),
    }
}

impl Delegate {
    pub fn new(start_url: String, mtm: MainThreadMarker) -> Retained<Self> {
        let this = Self::alloc(mtm).set_ivars(Ivars {
            browser: RefCell::new(None),
            start_url,
        });
        unsafe { msg_send![super(this), init] }
    }

    fn mtm(&self) -> MainThreadMarker {
        MainThreadMarker::from(self)
    }

    /// Re-apply the licence mid-session.
    ///
    /// Runs when a background check upgrades or downgrades the tier, so
    /// somebody who has just paid does not have to relaunch. Tabs opened
    /// before the ruleset landed are still queued inside `Rules` and get it
    /// attached retroactively.
    fn apply_entitlements(&self) {
        let mtm = self.mtm();
        let Some((entitlements, rules)) =
            self.with(|browser, _| (browser.reload_entitlements(), browser.rules().clone()))
        else {
            return;
        };

        menu::install(
            &NSApplication::sharedApplication(mtm),
            self.as_any(),
            entitlements,
            mtm,
        );

        if entitlements.allows(Feature::ContentBlocking) && !rules.is_ready() {
            load_blocklist(&rules, entitlements, mtm);
        }
    }

    /// This object as a plain `id`, for target/action and timers.
    fn as_any(&self) -> &AnyObject {
        self
    }

    /// Run something against the browser, if it exists and is not already
    /// borrowed.
    ///
    /// `try_borrow_mut` rather than `borrow_mut`: AppKit can re-enter a
    /// delegate during a call — closing the last tab closes the window, which
    /// dispatches further delegate messages — and in a release build with
    /// `panic = "abort"` a `RefCell` double-borrow would take the whole
    /// process down. Dropping the inner action is a bad outcome; aborting on
    /// a keystroke is a worse one.
    fn with<R>(&self, f: impl FnOnce(&mut Browser, Wiring<'_>) -> R) -> Option<R> {
        let mut slot = match self.ivars().browser.try_borrow_mut() {
            Ok(slot) => slot,
            Err(_) => {
                debug_assert!(false, "re-entered the browser state");
                return None;
            }
        };
        let browser = slot.as_mut()?;
        let wiring = Wiring {
            nav: ProtocolObject::from_ref(self),
            ui: ProtocolObject::from_ref(self),
            msg: ProtocolObject::from_ref(self),
            target: self.as_any(),
        };
        Some(f(browser, wiring))
    }
}
