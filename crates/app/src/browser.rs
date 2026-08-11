//! Window, layout, and the tab list they display.
//!
//! Everything AppKit-shaped lives here so that [`crate::delegate`] stays a
//! thin translation layer between Objective-C callbacks and these methods.

use std::time::Instant;

use objc2::rc::Retained;
use objc2::runtime::{AnyObject, ProtocolObject};
use objc2::{MainThreadMarker, MainThreadOnly};
use objc2_app_kit::{
    NSBackingStoreType, NSView, NSWindow, NSWindowDelegate, NSWindowOrderingMode,
    NSWindowStyleMask, NSWindowTitleVisibility,
};
use objc2_foundation::{NSPoint, NSRect, NSSize, NSString};
use objc2_web_kit::{WKNavigationDelegate, WKScriptMessageHandler, WKWebView};
use vel_engine::{Host, Rules, Session};
use vel_pro::{Entitlements, Feature};

use crate::hud::{Actions, Chrome};
use crate::omnibox;
use crate::tabs::{self, Swept, Tabs};

/// Everything the browser needs to point new AppKit and WebKit objects back
/// at the application delegate.
///
/// Passed in on each call rather than stored, deliberately. The delegate owns
/// the `Browser`; if the `Browser` retained the delegate in turn, neither
/// would ever be released — and the tab discarder exists precisely to make
/// this process give memory back.
#[derive(Clone, Copy)]
pub struct Wiring<'a> {
    pub nav: &'a ProtocolObject<dyn WKNavigationDelegate>,
    pub msg: &'a ProtocolObject<dyn WKScriptMessageHandler>,
    /// Target for tab-strip buttons, which are rebuilt as tabs come and go.
    pub target: &'a AnyObject,
}

pub struct Browser {
    window: Retained<NSWindow>,
    root: Retained<NSView>,
    chrome: Chrome,
    host: Host,
    rules: Rules,
    tabs: Tabs,
    entitlements: Entitlements,
}

impl Browser {
    pub fn new(
        window_delegate: &ProtocolObject<dyn NSWindowDelegate>,
        target: &AnyObject,
        actions: Actions,
        mtm: MainThreadMarker,
    ) -> Self {
        let content = NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(1280.0, 800.0));
        let style = NSWindowStyleMask::Titled
            | NSWindowStyleMask::Closable
            | NSWindowStyleMask::Miniaturizable
            | NSWindowStyleMask::Resizable
            // Let content run under the title bar; the chrome draws its own.
            | NSWindowStyleMask::FullSizeContentView;

        let window = unsafe {
            NSWindow::initWithContentRect_styleMask_backing_defer(
                NSWindow::alloc(mtm),
                content,
                style,
                NSBackingStoreType::Buffered,
                false,
            )
        };
        let entitlements = Entitlements::load();
        let title = if entitlements.is_supporter() {
            "Vel ✨ VIP"
        } else {
            "Vel"
        };
        window.setTitle(&NSString::from_str(title));
        window.setTitlebarAppearsTransparent(true);
        window.setTitleVisibility(NSWindowTitleVisibility::Hidden);
        window.setMinSize(NSSize::new(480.0, 360.0));
        window.setDelegate(Some(window_delegate));
        // We hold this in a `Retained`; letting AppKit release it on close
        // would leave that pointer dangling.
        unsafe { window.setReleasedWhenClosed(false) };

        let root = NSView::new(mtm);
        // Layer-backed so the whole window composites on one Core Animation
        // pass; a non-layer-backed ancestor forces the WKWebView's layer into
        // a slower software-composited path.
        root.setWantsLayer(true);
        window.setContentView(Some(&root));

        let chrome = Chrome::new(target, actions, mtm);
        root.addSubview(chrome.view());

        Self {
            window,
            root,
            chrome,
            host: Host::new(Session::Persistent, mtm),
            rules: Rules::new(),
            tabs: Tabs::new(),
            entitlements,
        }
    }

    pub fn rules(&self) -> &Rules {
        &self.rules
    }

    pub fn entitlements(&self) -> Entitlements {
        self.entitlements
    }

    /// Re-read the licence, after a background check changed it.
    pub fn reload_entitlements(&mut self) -> Entitlements {
        self.entitlements = Entitlements::load();
        self.entitlements
    }

    pub fn chrome_text(&self) -> String {
        self.chrome.text()
    }

    pub fn present(&self) {
        self.window.center();
        self.window.makeKeyAndOrderFront(None);
    }

    // -- tabs ---------------------------------------------------------------

    pub fn open_tab(&mut self, url: &str, wiring: Wiring<'_>) {
        let index = self.tabs.open(url.to_string());
        self.tabs.activate(index);
        self.show_active(wiring);
    }

    pub fn close_active(&mut self, wiring: Wiring<'_>) {
        if self.tabs.is_empty() {
            return;
        }
        let index = self.tabs.active_index();
        if let Some(page) = self.tabs.get(index).and_then(|t| t.page()) {
            page.view().removeFromSuperview();
        }
        self.tabs.close(index);

        if self.tabs.is_empty() {
            self.window.close();
            return;
        }
        self.show_active(wiring);
    }

    pub fn select_tab(&mut self, index: usize, wiring: Wiring<'_>) {
        if index >= self.tabs.len() || index == self.tabs.active_index() {
            return;
        }
        self.tabs.activate(index);
        self.show_active(wiring);
    }

    /// Handle Cmd+N. See [`tabs::numbered_index`] for what N means.
    pub fn select_numbered_tab(&mut self, n: usize, wiring: Wiring<'_>) {
        if let Some(index) = tabs::numbered_index(n, self.tabs.len()) {
            self.select_tab(index, wiring);
        }
    }

    pub fn cycle_tab(&mut self, forward: bool, wiring: Wiring<'_>) {
        let count = self.tabs.len();
        if count < 2 {
            return;
        }
        let current = self.tabs.active_index();
        let next = if forward {
            (current + 1) % count
        } else {
            (current + count - 1) % count
        };
        self.select_tab(next, wiring);
    }

    /// Put the active tab's web view on screen, building it if it was
    /// discarded.
    ///
    /// Background tabs are *hidden*, not detached. Detaching every tab you
    /// switch away from is what made switching slow: it drops the view out of
    /// the hierarchy, WebKit suspends the page, and coming back has to
    /// rebuild the layer tree and re-render before anything appears. Hiding
    /// keeps all of that warm, and the sweeper detaches tabs later once they
    /// have actually gone quiet — see [`crate::tabs`].
    pub fn show_active(&mut self, wiring: Wiring<'_>) {
        let Some(index) = (!self.tabs.is_empty()).then(|| self.tabs.active_index()) else {
            return;
        };

        let frame = self.web_frame();
        let url = self.tabs.get(index).map(|t| t.url.clone());

        if let Some(page) = self.tabs.revive(index, &self.host, &self.rules, frame) {
            // Freshly built: wire it up, then load. Delegates go on first so
            // the load's own callbacks are not missed.
            page.set_navigation_delegate(wiring.nav);
            page.set_message_handler(wiring.msg);
            if let Some(url) = &url {
                page.load(url);
            }
        }

        for (i, tab) in self.tabs.iter().enumerate() {
            let Some(page) = tab.page() else { continue };
            if i == index {
                page.set_frame(frame);
                if !page.is_attached() {
                    // Explicitly below the chrome. Plain `addSubview` appends,
                    // which would put every newly shown page *above* the bar —
                    // and anything the page overdraws (a floating button, a
                    // scrollbar, a rubber-band overscroll) then paints across
                    // the toolbar.
                    self.root.addSubview_positioned_relativeTo(
                        page.view(),
                        NSWindowOrderingMode::Below,
                        Some(self.chrome.view()),
                    );
                }
                page.set_hidden(false);
            } else if page.is_attached() {
                page.set_hidden(true);
            }
        }

        if let Some(page) = self.tabs.active_page() {
            // Typing should go to the page, not stay in the address bar.
            self.window.makeFirstResponder(Some(page.view()));
        }

        self.sync_chrome(wiring);
    }

    /// Move cold tabs down a tier. See [`crate::tabs`].
    ///
    /// Supporter feature. Without it, background tabs still stop animating —
    /// that comes free from hiding them — but nothing is ever detached or
    /// discarded, so a long-lived window keeps every tab it has opened.
    pub fn sweep(&mut self, wiring: Wiring<'_>) -> Swept {
        if !self.entitlements.allows(Feature::MemorySaver) {
            return Swept::default();
        }
        let swept = self.tabs.sweep(Instant::now());
        // Parking changes nothing the chrome shows — a parked tab still reads
        // as live — so only a discard is worth a repaint.
        if swept.discarded > 0 {
            self.sync_chrome(wiring);
        }
        swept
    }

    // -- navigation ---------------------------------------------------------

    pub fn navigate(&mut self, input: &str, wiring: Wiring<'_>) {
        let url = omnibox::resolve(input);
        if self.tabs.is_empty() {
            self.open_tab(&url, wiring);
            return;
        }
        let index = self.tabs.active_index();
        self.tabs.set_url(index, url.clone());
        self.tabs.touch_active();

        // The tab may have been discarded while the user was typing in it.
        self.show_active(wiring);
        if let Some(page) = self.tabs.active_page() {
            page.load(&url);
        }
    }

    /// Run something on the active page, if there is one.
    pub fn with_active_page(&mut self, f: impl FnOnce(&vel_engine::Page)) {
        self.tabs.touch_active();
        if let Some(page) = self.tabs.active_page() {
            f(page);
        }
    }

    // -- state coming back from WebKit --------------------------------------

    pub fn page_changed(&mut self, view: &WKWebView, wiring: Wiring<'_>) {
        if let Some(index) = self.tabs.index_of_view(view) {
            self.refresh(index, wiring);
        }
    }

    /// Re-read a tab's title and URL from its web view.
    ///
    /// Driven both by the navigation delegate and by the injected script's
    /// same-document nudge. The values always come from WebKit, never from the
    /// page — a page that could hand us a URL to display could spoof the
    /// address bar.
    pub fn refresh_by_id(&mut self, tab_id: u64, wiring: Wiring<'_>) {
        if let Some(index) = self.tabs.index_of_id(tab_id) {
            self.refresh(index, wiring);
        }
    }

    fn refresh(&mut self, index: usize, wiring: Wiring<'_>) {
        let (title, url) = match self.tabs.get(index).and_then(|t| t.page()) {
            Some(page) => (page.title(), page.url()),
            None => return,
        };
        if let Some(title) = title {
            self.tabs.set_title(index, title);
        }
        if let Some(url) = url {
            self.tabs.set_url(index, url);
        }
        self.sync_chrome(wiring);
    }

    pub fn set_audible(&mut self, tab_id: u64, audible: bool, wiring: Wiring<'_>) {
        let Some(index) = self.tabs.index_of_id(tab_id) else {
            return;
        };
        if self.tabs.get(index).is_some_and(|t| t.audible == audible) {
            return;
        }
        self.tabs.set_audible(index, audible);
        self.sync_chrome(wiring);
    }

    // -- chrome -------------------------------------------------------------

    pub fn focus_address(&self) {
        self.window.makeFirstResponder(Some(self.chrome.field()));
        // Select what is there, so typing replaces the current URL instead of
        // inserting into it. Focusing without this is the difference between
        // Cmd+L behaving like every other browser and behaving like a text
        // editor that happens to hold a URL.
        self.chrome.select_address();
    }

    /// Refresh the address bar and tab strip from the tab list.
    pub fn sync_chrome(&mut self, wiring: Wiring<'_>) {
        if let Some(tab) = self.tabs.active_tab() {
            self.chrome.set_text(&omnibox::for_display(&tab.url));
        }
        let (back, forward) = self
            .tabs
            .active_page()
            .map_or((false, false), |p| (p.can_go_back(), p.can_go_forward()));
        self.chrome.set_history(back, forward);

        // Syncing the strip can change the chrome's height — the strip
        // appears with the second tab — so lay out afterwards, not before.
        self.chrome.sync(&self.tabs, wiring.target);
        self.layout();
    }

    pub fn layout(&self) {
        let bounds = self.root.bounds();
        let (w, h) = (bounds.size.width, bounds.size.height);
        let chrome_h = self.chrome.height(self.tabs.len());

        self.chrome.view().setFrame(NSRect::new(
            NSPoint::new(0.0, h - chrome_h),
            NSSize::new(w, chrome_h),
        ));
        self.chrome.layout(w, self.tabs.len());

        if let Some(page) = self.tabs.active_page() {
            page.set_frame(self.web_frame());
        }
    }

    fn web_frame(&self) -> NSRect {
        let bounds = self.root.bounds();
        let chrome_h = self.chrome.height(self.tabs.len());
        NSRect::new(
            NSPoint::new(0.0, 0.0),
            NSSize::new(bounds.size.width, (bounds.size.height - chrome_h).max(0.0)),
        )
    }

}
