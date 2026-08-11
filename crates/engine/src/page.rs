//! One live web view.
//!
//! A [`Page`] is deliberately cheap to drop. Tab discarding works by
//! releasing the `Page` and keeping only the URL, which is what actually
//! returns the WebContent process's memory to the system — so everything
//! here has to come apart cleanly, including the reference cycle noted in
//! [`Page::teardown`].

use objc2::rc::Retained;
use objc2::runtime::ProtocolObject;
use objc2::{MainThreadMarker, MainThreadOnly};
use objc2_foundation::{NSRect, NSString, NSURLRequest, NSURL};
use objc2_web_kit::{
    WKNavigationDelegate, WKScriptMessageHandler, WKUIDelegate, WKUserContentController,
    WKUserScript, WKUserScriptInjectionTime, WKWebView,
};

use crate::config::Host;
use crate::rules::Rules;
use crate::script;

pub struct Page {
    view: Retained<WKWebView>,
    content: Retained<WKUserContentController>,
    rules: Rules,
    /// Set once a message handler is installed, so `teardown` knows whether
    /// there is a cycle to break.
    handler_installed: bool,
}

impl Page {
    pub fn new(host: &Host, rules: &Rules, tab_id: u64, frame: NSRect) -> Self {
        let mtm = host.mtm();
        let (config, content) = host.configuration();

        // The script goes on before the view exists, so it is present for the
        // very first navigation rather than the second.
        let source = NSString::from_str(&script::for_tab(tab_id));
        let user_script = unsafe {
            WKUserScript::initWithSource_injectionTime_forMainFrameOnly(
                WKUserScript::alloc(mtm),
                &source,
                WKUserScriptInjectionTime::AtDocumentStart,
                true,
            )
        };
        unsafe { content.addUserScript(&user_script) };

        // Same reasoning for the blocklist: attach before the first request.
        rules.attach(&content);

        let view = unsafe {
            WKWebView::initWithFrame_configuration(WKWebView::alloc(mtm), frame, &config)
        };

        unsafe {
            // Two-finger swipe for back/forward, the gesture macOS users
            // already have in their hands.
            view.setAllowsBackForwardNavigationGestures(true);
            // Web Inspector in debug builds only. It is not free: an
            // inspectable view keeps extra bookkeeping alive per page.
            view.setInspectable(cfg!(debug_assertions));
        }

        Self {
            view,
            content,
            rules: rules.clone(),
            handler_installed: false,
        }
    }

    pub fn view(&self) -> &WKWebView {
        &self.view
    }

    pub fn set_navigation_delegate(&self, delegate: &ProtocolObject<dyn WKNavigationDelegate>) {
        unsafe { self.view.setNavigationDelegate(Some(delegate)) };
    }

    pub fn set_ui_delegate(&self, delegate: &ProtocolObject<dyn WKUIDelegate>) {
        unsafe { self.view.setUIDelegate(Some(delegate)) };
    }

    pub fn set_message_handler(&mut self, handler: &ProtocolObject<dyn WKScriptMessageHandler>) {
        unsafe {
            self.content
                .addScriptMessageHandler_name(handler, &NSString::from_str(script::CHANNEL));
        }
        self.handler_installed = true;
    }

    pub fn load(&self, url: &str) -> bool {
        let Some(url) = NSURL::URLWithString(&NSString::from_str(url)) else {
            return false;
        };
        let request = NSURLRequest::requestWithURL(&url);
        unsafe { self.view.loadRequest(&request) };
        true
    }

    pub fn url(&self) -> Option<String> {
        unsafe { self.view.URL() }
            .and_then(|u| u.absoluteString())
            .map(|s| s.to_string())
    }

    pub fn title(&self) -> Option<String> {
        unsafe { self.view.title() }
            .map(|t| t.to_string())
            .filter(|t| !t.is_empty())
    }

    pub fn is_loading(&self) -> bool {
        unsafe { self.view.isLoading() }
    }

    pub fn progress(&self) -> f64 {
        unsafe { self.view.estimatedProgress() }
    }

    pub fn can_go_back(&self) -> bool {
        unsafe { self.view.canGoBack() }
    }

    pub fn can_go_forward(&self) -> bool {
        unsafe { self.view.canGoForward() }
    }

    pub fn go_back(&self) {
        unsafe { self.view.goBack() };
    }

    pub fn go_forward(&self) {
        unsafe { self.view.goForward() };
    }

    /// Plain reload, reusing the HTTP cache.
    pub fn reload(&self) {
        unsafe { self.view.reload() };
    }

    /// Reload ignoring caches — Cmd+Shift+R.
    pub fn reload_from_origin(&self) {
        unsafe { self.view.reloadFromOrigin() };
    }

    pub fn stop(&self) {
        unsafe { self.view.stopLoading() };
    }

    pub fn toggle_pip(&self) {
        self.eval(script::TOGGLE_PIP);
    }

    pub fn eval(&self, js: &str) {
        unsafe {
            self.view
                .evaluateJavaScript_completionHandler(&NSString::from_str(js), None);
        }
    }

    /// Detach everything that points back at the application.
    ///
    /// `WKUserContentController` retains its script message handlers
    /// *strongly*. Our handler is the application delegate, which owns the
    /// tab list, which owns this page, which owns the controller — a cycle
    /// that would leak the entire WebContent process on every discarded tab,
    /// which is precisely the memory the discarder is trying to reclaim.
    /// Dropping a `Page` runs this first.
    pub fn teardown(&mut self) {
        unsafe {
            self.view.setNavigationDelegate(None);
            self.view.setUIDelegate(None);
            self.view.stopLoading();
            if self.handler_installed {
                self.content
                    .removeScriptMessageHandlerForName(&NSString::from_str(script::CHANNEL));
                self.handler_installed = false;
            }
            self.content.removeAllUserScripts();
            self.content.removeAllContentRuleLists();
        }
        self.rules.detach(&self.content);
    }

    pub fn set_frame(&self, frame: NSRect) {
        self.view.setFrame(frame);
    }

    /// Hide or show without leaving the view hierarchy.
    ///
    /// This is the cheap half of backgrounding a tab. WebKit still sees a
    /// hidden view as not visible, so the page goes to `document.hidden`,
    /// `requestAnimationFrame` stops and timers are throttled — but the web
    /// process, its layer tree and its rendered tiles all stay put, so coming
    /// back is an unhide rather than a re-render.
    pub fn set_hidden(&self, hidden: bool) {
        self.view.setHidden(hidden);
    }

    pub fn is_attached(&self) -> bool {
        unsafe { self.view.superview() }.is_some()
    }

    /// Take the view out of the hierarchy.
    ///
    /// The expensive half. Detached *and* not visible is the state
    /// `WKInactiveSchedulingPolicySuspend` acts on, so this is what actually
    /// suspends JavaScript and layout — at the cost of a real re-render when
    /// the tab comes back. Worth it for a tab that has been idle a while,
    /// not for one the user is switching between.
    pub fn detach(&self) {
        self.view.removeFromSuperview();
    }

    pub fn mtm(&self) -> MainThreadMarker {
        MainThreadMarker::from(&*self.view)
    }
}

impl Drop for Page {
    fn drop(&mut self) {
        self.teardown();
    }
}
