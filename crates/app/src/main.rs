//! Vel — a small, fast browser for macOS.
//!
//! The entire program is: bring up an `NSApplication`, hand it a delegate,
//! run. Everything after that is event-driven. There is no render loop, no
//! tick, and no background thread — when nothing is happening, this process
//! is asleep and the only thing running is WebKit.

mod browser;
mod delegate;
mod hud;
mod menu;
mod omnibox;
mod tabs;

use objc2::runtime::ProtocolObject;
use objc2::MainThreadMarker;
use objc2_app_kit::{NSApplication, NSApplicationActivationPolicy};

fn main() {
    let mtm = MainThreadMarker::new().expect("main() must run on the main thread");

    // `vel https://example.com` opens straight to a page; bare `vel` opens
    // the default start page.
    let start = std::env::args()
        .nth(1)
        .map(|arg| omnibox::resolve(&arg))
        .unwrap_or_else(|| omnibox::HOME.to_string());

    let app = NSApplication::sharedApplication(mtm);
    app.setActivationPolicy(NSApplicationActivationPolicy::Regular);

    let delegate = delegate::Delegate::new(start, mtm);
    app.setDelegate(Some(ProtocolObject::from_ref(&*delegate)));

    app.run();
}
