//! Getting a content-blocking ruleset into WebKit.
//!
//! Compilation is genuinely expensive — WebKit turns the JSON into a
//! bytecode program, and a large list takes the better part of a second —
//! but it also caches the result on disk under the identifier we supply.
//! Because our identifiers are content-addressed (see
//! `vel_blocker::ruleset_id`), a launch with an unchanged list finds the
//! compiled form and skips straight to it. Compilation only happens the
//! first time a given list is seen.

use std::cell::RefCell;
use std::rc::Rc;

use block2::RcBlock;
use objc2::rc::Retained;
use objc2::{MainThreadMarker, Message};
use objc2_foundation::NSString;
use objc2_web_kit::{WKContentRuleList, WKContentRuleListStore, WKUserContentController};

/// Holds the compiled ruleset once WebKit hands it back.
///
/// Compilation is asynchronous, so tabs opened during startup exist before
/// the list does. They register here and get the list applied as soon as it
/// lands; tabs created afterwards pick it up at construction. Either way no
/// tab has to wait for the blocker to be ready before it can load.
#[derive(Clone, Default)]
pub struct Rules {
    inner: Rc<RefCell<State>>,
}

#[derive(Default)]
struct State {
    list: Option<Retained<WKContentRuleList>>,
    pending: Vec<Retained<WKUserContentController>>,
}

impl Rules {
    pub fn new() -> Self {
        Self::default()
    }

    /// Apply the ruleset to a content controller, now or when it arrives.
    pub fn attach(&self, controller: &WKUserContentController) {
        let mut state = self.inner.borrow_mut();
        match &state.list {
            Some(list) => unsafe { controller.addContentRuleList(list) },
            None => state.pending.push(controller.retain()),
        }
    }

    /// Forget a controller that is going away before the rules landed.
    pub fn detach(&self, controller: &WKUserContentController) {
        self.inner
            .borrow_mut()
            .pending
            .retain(|c| !std::ptr::eq(&**c, controller));
    }

    pub fn is_ready(&self) -> bool {
        self.inner.borrow().list.is_some()
    }

    /// Look the ruleset up in WebKit's cache, compiling it only if absent.
    ///
    /// `on_ready` reports the outcome: `Ok(true)` if the list came from
    /// cache, `Ok(false)` if it had to be compiled, `Err` with WebKit's
    /// message if it could not be loaded at all. A failure here is not fatal
    /// — the browser runs unfiltered rather than not at all.
    pub fn load(
        &self,
        identifier: &str,
        json: &str,
        mtm: MainThreadMarker,
        on_ready: impl Fn(Result<bool, String>) + 'static,
    ) {
        let Some(store) = (unsafe { WKContentRuleListStore::defaultStore(mtm) }) else {
            on_ready(Err("no default content rule list store".into()));
            return;
        };

        let id = NSString::from_str(identifier);
        let json = NSString::from_str(json);
        let on_ready = Rc::new(on_ready);

        // Second leg: compile, used when the lookup misses.
        let compile = {
            let this = self.clone();
            let store = store.clone();
            let id = id.clone();
            let on_ready = on_ready.clone();
            move || {
                let this = this.clone();
                let on_ready = on_ready.clone();
                let handler = RcBlock::new(move |list: *mut WKContentRuleList, err: *mut objc2_foundation::NSError| {
                    if let Some(list) = unsafe { list.as_ref() } {
                        this.install(list);
                        on_ready(Ok(false));
                    } else {
                        on_ready(Err(describe(err)));
                    }
                });
                unsafe {
                    store.compileContentRuleListForIdentifier_encodedContentRuleList_completionHandler(
                        Some(&id),
                        Some(&json),
                        Some(&handler),
                    );
                }
            }
        };

        // First leg: ask the cache.
        let this = self.clone();
        let lookup = RcBlock::new(move |list: *mut WKContentRuleList, _err: *mut objc2_foundation::NSError| {
            match unsafe { list.as_ref() } {
                Some(list) => {
                    this.install(list);
                    on_ready(Ok(true));
                }
                None => compile(),
            }
        });

        unsafe {
            store.lookUpContentRuleListForIdentifier_completionHandler(Some(&id), Some(&lookup));
        }
    }

    fn install(&self, list: &WKContentRuleList) {
        let mut state = self.inner.borrow_mut();
        state.list = Some(list.retain());
        for controller in state.pending.drain(..) {
            unsafe { controller.addContentRuleList(list) };
        }
    }
}

fn describe(err: *mut objc2_foundation::NSError) -> String {
    match unsafe { err.as_ref() } {
        Some(e) => e.localizedDescription().to_string(),
        None => "unknown error".into(),
    }
}
