//! Tabs, and the policy that decides which ones stop existing.
//!
//! The memory story of this browser lives here. A tab is a URL, a title, and
//! *optionally* a live [`Page`]. Tabs the user has not looked at in a while
//! give up their `Page`, which releases the WKWebView and lets WebKit tear
//! down the WebContent process behind it — the only thing that actually
//! returns hundreds of megabytes rather than merely promising to. Reviving a
//! tab reloads its URL, which on a warm HTTP cache is a fraction of a second.
//!
//! Backgrounded tabs go through three states, and it is worth keeping them
//! straight because they trade switching speed against resources in that
//! order:
//!
//! 1. **Hidden.** Still in the view hierarchy. WebKit sees a hidden view as
//!    not visible, so the page goes to `document.hidden` and stops animating,
//!    but everything stays warm. Switching back is instant, and this is where
//!    a tab lives for the first [`PARK_AFTER`].
//! 2. **Parked** — detached from the hierarchy. Detached *and* idle is the
//!    state WebKit's `WKInactiveSchedulingPolicySuspend` acts on, so this is
//!    what actually suspends JavaScript and layout. Coming back costs a
//!    re-render.
//! 3. **Discarded** after [`IDLE_BEFORE_DISCARD`]. The `Page` is dropped, the
//!    WebContent process goes away, and the memory is genuinely returned.
//!    Coming back costs a reload.
//!
//! Tier 1 exists because tier 2 used to be the only option, and paying a
//! re-render every time the user pressed Cmd+Shift+] was the wrong trade for
//! a tab they had left two seconds ago.

use std::time::{Duration, Instant};

use objc2_foundation::NSRect;
use objc2_web_kit::WKWebView;
use vel_engine::{Host, Page, Rules};

/// How long a tab stays instantly switchable after you leave it.
///
/// Sized to cover switching back and forth between two tabs — comparing two
/// pages, reading one while another loads — which is where a re-render is
/// most obvious and least justified.
pub const PARK_AFTER: Duration = Duration::from_secs(45);

/// How long a tab must go untouched before it is eligible for discarding.
///
/// Five minutes is chosen to sit outside the rhythm of actually using tabs.
/// Anything shorter and a tab you alt-tabbed away from reloads when you come
/// back, which reads as the browser losing your place.
pub const IDLE_BEFORE_DISCARD: Duration = Duration::from_secs(5 * 60);

/// What one pass of the sweeper did.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Swept {
    pub parked: usize,
    pub discarded: usize,
}

pub struct Tab {
    pub id: u64,
    pub url: String,
    pub title: String,
    /// Reported by the injected script, not by WebKit — there is no public
    /// API for it. See `vel_engine::script`.
    pub audible: bool,
    pub touched: Instant,
    page: Option<Page>,
}

impl Tab {
    pub fn is_live(&self) -> bool {
        self.page.is_some()
    }

    pub fn page(&self) -> Option<&Page> {
        self.page.as_ref()
    }

    /// What to show on the tab button.
    pub fn label(&self) -> String {
        let text = if self.title.is_empty() {
            crate::omnibox::for_display(&self.url)
        } else {
            self.title.clone()
        };
        if self.audible {
            format!("♪ {text}")
        } else {
            text
        }
    }
}

/// The discard decision, kept free of AppKit so it can be tested directly.
pub fn should_discard(live: bool, active: bool, audible: bool, idle: Duration) -> bool {
    // Audible tabs are exempt at any age: discarding the podcast someone is
    // listening to in the background is the single most annoying thing a
    // memory manager can do.
    live && !active && !audible && idle >= IDLE_BEFORE_DISCARD
}

/// Which tab a Cmd+N shortcut addresses.
///
/// 1 through 8 select that tab directly. 9 selects the *last* tab whatever
/// the count — the convention Safari and Chrome both follow. That
/// inconsistency is deliberate: matching what people's fingers already do
/// matters more here than being internally tidy.
///
/// A number past the end selects nothing rather than clamping, so Cmd+5 with
/// three tabs open is a no-op instead of a surprise jump.
pub fn numbered_index(n: usize, count: usize) -> Option<usize> {
    if count == 0 || n == 0 {
        return None;
    }
    if n >= 9 {
        return Some(count - 1);
    }
    (n <= count).then(|| n - 1)
}

/// The parking decision — tier 1 to tier 2.
///
/// Audible tabs *are* parked. Detaching does not stop playback, and WebKit
/// explicitly does not treat a view that is playing media as idle, so the
/// suspend policy leaves it running; all parking costs it is a re-render if
/// the user comes back.
pub fn should_park(live: bool, attached: bool, active: bool, idle: Duration) -> bool {
    live && attached && !active && idle >= PARK_AFTER
}

pub struct Tabs {
    items: Vec<Tab>,
    active: usize,
    next_id: u64,
}

impl Tabs {
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            active: 0,
            next_id: 1,
        }
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn active_index(&self) -> usize {
        self.active
    }

    pub fn iter(&self) -> impl Iterator<Item = &Tab> {
        self.items.iter()
    }

    pub fn get(&self, index: usize) -> Option<&Tab> {
        self.items.get(index)
    }

    pub fn active_tab(&self) -> Option<&Tab> {
        self.items.get(self.active)
    }

    pub fn active_page(&self) -> Option<&Page> {
        self.active_tab().and_then(Tab::page)
    }

    /// Append a tab. It has no `Page` until something calls [`Tabs::revive`].
    pub fn open(&mut self, url: String) -> usize {
        let id = self.next_id;
        self.next_id += 1;
        self.items.push(Tab {
            id,
            url,
            title: String::new(),
            audible: false,
            touched: Instant::now(),
            page: None,
        });
        self.items.len() - 1
    }

    /// Close a tab and pick a sensible neighbour to focus.
    pub fn close(&mut self, index: usize) {
        if index >= self.items.len() {
            return;
        }
        self.items.remove(index);
        if self.active >= self.items.len() {
            self.active = self.items.len().saturating_sub(1);
        } else if index < self.active {
            self.active -= 1;
        }
    }

    pub fn activate(&mut self, index: usize) {
        if index < self.items.len() {
            self.active = index;
            self.items[index].touched = Instant::now();
        }
    }

    pub fn touch_active(&mut self) {
        if let Some(tab) = self.items.get_mut(self.active) {
            tab.touched = Instant::now();
        }
    }

    pub fn index_of_id(&self, id: u64) -> Option<usize> {
        self.items.iter().position(|t| t.id == id)
    }

    /// Which tab does this web view belong to?
    ///
    /// Navigation callbacks hand back a `WKWebView` and nothing else, so the
    /// mapping is by object identity.
    pub fn index_of_view(&self, view: &WKWebView) -> Option<usize> {
        self.items
            .iter()
            .position(|t| t.page().is_some_and(|p| std::ptr::eq(p.view(), view)))
    }

    pub fn set_title(&mut self, index: usize, title: String) {
        if let Some(tab) = self.items.get_mut(index) {
            tab.title = title;
        }
    }

    pub fn set_url(&mut self, index: usize, url: String) {
        if let Some(tab) = self.items.get_mut(index) {
            tab.url = url;
        }
    }

    pub fn set_audible(&mut self, index: usize, audible: bool) {
        if let Some(tab) = self.items.get_mut(index) {
            tab.audible = audible;
        }
    }

    /// Make sure a tab has a live page, creating and loading one if not.
    ///
    /// Returns `Some` only when a page was actually created, so the caller
    /// knows it still has to attach delegates — doing that unconditionally
    /// would re-register the script message handler on every tab switch.
    pub fn revive(
        &mut self,
        index: usize,
        host: &Host,
        rules: &Rules,
        frame: NSRect,
    ) -> Option<&mut Page> {
        let tab = self.items.get_mut(index)?;
        if tab.page.is_some() {
            return None;
        }
        let page = Page::new(host, rules, tab.id, frame);
        tab.page = Some(page);
        tab.touched = Instant::now();
        tab.page.as_mut()
    }

    /// Release a tab's page, keeping enough to rebuild it.
    pub fn discard(&mut self, index: usize) -> bool {
        let Some(tab) = self.items.get_mut(index) else {
            return false;
        };
        // Capture the live URL first: the tab may have navigated since it was
        // opened, and reviving it to a stale address would lose the user's
        // place more thoroughly than the reload already does.
        if let Some(page) = &tab.page {
            if let Some(url) = page.url() {
                tab.url = url;
            }
            page.view().removeFromSuperview();
        }
        tab.page.take().is_some()
    }

    /// Detach a tab's view, keeping the page alive.
    pub fn park(&mut self, index: usize) -> bool {
        let Some(tab) = self.items.get_mut(index) else {
            return false;
        };
        let Some(page) = &tab.page else {
            return false;
        };
        if !page.is_attached() {
            return false;
        }
        page.detach();
        true
    }

    /// Move every eligible tab down a tier.
    ///
    /// Discards are decided before parks so that a tab cold enough for both
    /// goes straight to discarded rather than being parked on the way.
    pub fn sweep(&mut self, now: Instant) -> Swept {
        let mut swept = Swept::default();

        let cold: Vec<usize> = self
            .items
            .iter()
            .enumerate()
            .filter(|(i, t)| {
                should_discard(t.is_live(), *i == self.active, t.audible, now - t.touched)
            })
            .map(|(i, _)| i)
            .collect();
        swept.discarded = cold.iter().filter(|i| self.discard(**i)).count();

        let idle: Vec<usize> = self
            .items
            .iter()
            .enumerate()
            .filter(|(i, t)| {
                let attached = t.page().is_some_and(|p| p.is_attached());
                should_park(t.is_live(), attached, *i == self.active, now - t.touched)
            })
            .map(|(i, _)| i)
            .collect();
        swept.parked = idle.iter().filter(|i| self.park(**i)).count();

        swept
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const COLD: Duration = Duration::from_secs(600);
    const WARM: Duration = Duration::from_secs(30);
    const TICK: Duration = Duration::from_secs(1);

    #[test]
    fn cold_background_tabs_are_discarded() {
        assert!(should_discard(true, false, false, COLD));
    }

    #[test]
    fn the_active_tab_is_never_discarded() {
        assert!(!should_discard(true, true, false, COLD));
    }

    #[test]
    fn audible_tabs_survive_indefinitely() {
        assert!(!should_discard(true, false, true, COLD * 100));
    }

    #[test]
    fn recently_used_tabs_are_left_alone() {
        assert!(!should_discard(true, false, false, WARM));
    }

    #[test]
    fn already_discarded_tabs_are_not_revisited() {
        assert!(!should_discard(false, false, false, COLD));
    }

    /// The switching-speed guarantee: a tab you just left keeps its views, so
    /// coming back is an unhide rather than a re-render.
    #[test]
    fn a_tab_just_left_stays_attached() {
        assert!(!should_park(true, true, false, Duration::from_secs(5)));
        assert!(!should_park(true, true, false, PARK_AFTER - TICK));
    }

    #[test]
    fn tabs_park_once_they_go_quiet() {
        assert!(should_park(true, true, false, PARK_AFTER));
        // The active tab and already-parked tabs are both no-ops.
        assert!(!should_park(true, true, true, COLD));
        assert!(!should_park(true, false, false, COLD));
    }

    /// Parking a tab does not stop its audio, so unlike discarding it applies
    /// to audible tabs too.
    #[test]
    fn audible_tabs_still_park() {
        assert!(should_park(true, true, false, COLD));
    }

    #[test]
    fn closing_keeps_the_selection_on_the_same_tab() {
        let mut tabs = Tabs::new();
        for _ in 0..4 {
            tabs.open("about:blank".into());
        }
        tabs.activate(2);

        // Closing to the left of the active tab shifts it down one.
        tabs.close(0);
        assert_eq!(tabs.active_index(), 1);
        assert_eq!(tabs.len(), 3);

        // Closing to the right leaves it where it is.
        tabs.close(2);
        assert_eq!(tabs.active_index(), 1);

        // Closing the last tab clamps rather than running off the end.
        tabs.close(1);
        tabs.close(0);
        assert_eq!(tabs.active_index(), 0);
        assert!(tabs.is_empty());
    }

    #[test]
    fn cmd_number_addresses_tabs_directly() {
        assert_eq!(numbered_index(1, 5), Some(0));
        assert_eq!(numbered_index(3, 5), Some(2));
        assert_eq!(numbered_index(5, 5), Some(4));
    }

    #[test]
    fn cmd_nine_is_always_the_last_tab() {
        assert_eq!(numbered_index(9, 3), Some(2));
        assert_eq!(numbered_index(9, 12), Some(11));
        assert_eq!(numbered_index(9, 1), Some(0));
    }

    #[test]
    fn cmd_number_past_the_end_does_nothing() {
        assert_eq!(numbered_index(5, 3), None);
        assert_eq!(numbered_index(1, 0), None);
        assert_eq!(numbered_index(0, 3), None);
    }

    #[test]
    fn ids_are_stable_across_removals() {
        let mut tabs = Tabs::new();
        tabs.open("a".into());
        tabs.open("b".into());
        let second = tabs.get(1).unwrap().id;
        tabs.close(0);
        // The script message handler routes by id, so a shifted index must
        // not change which tab an id refers to.
        assert_eq!(tabs.index_of_id(second), Some(0));
    }
}
