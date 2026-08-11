//! The chrome: one unified bar, in the shape macOS users already know.
//!
//! Everything here is plain AppKit. There is no UI toolkit and no second
//! renderer in the process — an `NSTextField` on an `NSVisualEffectView`
//! costs a few kilobytes and draws on the same Core Animation pass the window
//! is doing anyway. A retained-mode Rust GUI would mean a second GPU context
//! and a repaint loop competing with the thing we actually care about
//! keeping smooth, which is the video.
//!
//! The capsules — the address pill and the selected tab — are
//! `NSVisualEffectView`s rather than layers we fill ourselves. A hand-set
//! `CGColor` is a snapshot of the current appearance and goes wrong the
//! moment the user switches to dark mode or moves the window onto a
//! different backdrop; a material is re-evaluated by AppKit for free.

use objc2::rc::Retained;
use objc2::runtime::{AnyObject, Sel};
use objc2::MainThreadMarker;
use objc2_app_kit::{
    NSButton, NSColor, NSFocusRingType, NSFont, NSImage, NSLineBreakMode,
    NSTextAlignment, NSTextField, NSView, NSVisualEffectBlendingMode, NSVisualEffectMaterial,
    NSVisualEffectState, NSVisualEffectView,
};
use objc2_foundation::{NSPoint, NSRect, NSSize, NSString};

use crate::tabs::Tabs;

/// Height of the toolbar row.
pub const TOOLBAR_H: f64 = 52.0;
/// Height of the tab strip, added only when there is more than one tab.
pub const STRIP_H: f64 = 34.0;
/// Space the window's close/minimise/zoom buttons need on the left.
const TRAFFIC_LIGHTS: f64 = 78.0;
const PAD: f64 = 12.0;
const GAP: f64 = 4.0;
const BTN: f64 = 28.0;
const PILL_H: f64 = 30.0;
const PILL_MAX_W: f64 = 620.0;
const CAPSULE_RADIUS: f64 = 8.0;

/// Selectors the chrome's controls send to the application delegate.
#[derive(Clone, Copy)]
pub struct Actions {
    pub submit: Sel,
    pub back: Sel,
    pub forward: Sel,
    pub new_tab: Sel,
    pub select_tab: Sel,
}

struct TabButton {
    /// Clipping container, one per tab.
    ///
    /// `NSButton` draws its title from the centre outwards and does not
    /// confine it to its own frame, so with more tabs than fit, every label
    /// bled over its neighbours. Line-break mode alone did not hold — the
    /// only thing that reliably cannot overflow is a layer that masks to its
    /// bounds, so each tab gets one and everything else lives inside it.
    slot: Retained<NSView>,
    /// Selection background. Shown only for the active tab.
    capsule: Retained<NSVisualEffectView>,
    button: Retained<NSButton>,
}

pub struct Chrome {
    bar: Retained<NSVisualEffectView>,
    pill: Retained<NSVisualEffectView>,
    field: Retained<NSTextField>,
    back: Retained<NSButton>,
    forward: Retained<NSButton>,
    new_tab: Retained<NSButton>,
    vip_badge: Option<Retained<NSTextField>>,
    strip: Retained<NSView>,
    tabs: Vec<TabButton>,
    /// Tab ids the strip was last built for. Rebuilding is only necessary
    /// when this changes — see [`Chrome::sync`].
    built_for: Vec<u64>,
    actions: Actions,
    mtm: MainThreadMarker,
}

impl Chrome {
    pub fn new(target: &AnyObject, actions: Actions, mtm: MainThreadMarker) -> Self {
        let bar = NSVisualEffectView::new(mtm);
        bar.setMaterial(NSVisualEffectMaterial::HeaderView);
        // Blend with the page behind it rather than the desktop: the bar sits
        // over web content, not over the wallpaper.
        bar.setBlendingMode(NSVisualEffectBlendingMode::WithinWindow);
        bar.setState(NSVisualEffectState::FollowsWindowActiveState);

        let back = symbol_button("chevron.left", target, actions.back, mtm);
        let forward = symbol_button("chevron.right", target, actions.forward, mtm);
        let new_tab = symbol_button("plus", target, actions.new_tab, mtm);
        bar.addSubview(&back);
        bar.addSubview(&forward);
        bar.addSubview(&new_tab);

        let pill = capsule(NSVisualEffectMaterial::ContentBackground, PILL_H / 2.0, mtm);
        bar.addSubview(&pill);

        let field = NSTextField::new(mtm);
        field.setPlaceholderString(Some(&NSString::from_str("Search or enter address")));
        // The pill already draws the field's background and edge, so the
        // field itself is undecorated — otherwise there would be two borders
        // saying the same thing.
        field.setBezeled(false);
        field.setBordered(false);
        field.setDrawsBackground(false);
        field.setFocusRingType(NSFocusRingType::None);
        field.setFont(Some(&NSFont::systemFontOfSize(13.0)));
        field.setUsesSingleLineMode(true);
        field.setAlignment(NSTextAlignment::Center);
        unsafe {
            field.setTarget(Some(target));
            field.setAction(Some(actions.submit));
        }
        pill.addSubview(&field);

        let entitlements = vel_pro::Entitlements::load();
        let vip_badge = if entitlements.is_supporter() {
            let label = NSTextField::new(mtm);
            label.setStringValue(&NSString::from_str("✨ VIP"));
            label.setBezeled(false);
            label.setBordered(false);
            label.setDrawsBackground(false);
            label.setFont(Some(&NSFont::boldSystemFontOfSize(11.0)));
            label.setTextColor(Some(&NSColor::colorWithRed_green_blue_alpha(1.0, 0.78, 0.2, 1.0)));
            label.setAlignment(NSTextAlignment::Center);
            pill.addSubview(&label);
            Some(label)
        } else {
            None
        };

        let strip = NSView::new(mtm);
        bar.addSubview(&strip);

        Self {
            bar,
            pill,
            field,
            back,
            forward,
            new_tab,
            vip_badge,
            strip,
            tabs: Vec::new(),
            built_for: Vec::new(),
            actions,
            mtm,
        }
    }

    pub fn view(&self) -> &NSView {
        &self.bar
    }

    pub fn field(&self) -> &NSTextField {
        &self.field
    }

    /// Total chrome height for a given tab count.
    pub fn height(&self, tab_count: usize) -> f64 {
        if tab_count > 1 {
            TOOLBAR_H + STRIP_H
        } else {
            TOOLBAR_H
        }
    }

    pub fn set_text(&self, text: &str) {
        self.field.setStringValue(&NSString::from_str(text));
    }

    pub fn text(&self) -> String {
        self.field.stringValue().to_string()
    }

    pub fn select_address(&self) {
        unsafe { self.field.selectText(None) };
    }

    pub fn set_history(&self, can_back: bool, can_forward: bool) {
        self.back.setEnabled(can_back);
        self.forward.setEnabled(can_forward);
    }

    /// Bring the tab strip in line with the tab list.
    ///
    /// This runs on every tab switch, every title change and every navigation,
    /// so the common case has to be cheap. Creating and destroying an
    /// `NSButton` per tab each time was measurable on switches; now views are
    /// only built when the set of tab *ids* changes, and selecting a tab or
    /// renaming one just writes a title and a tint into views that already
    /// exist.
    pub fn sync(&mut self, tabs: &Tabs, target: &AnyObject) {
        let ids: Vec<u64> = tabs.iter().map(|t| t.id).collect();
        if ids != self.built_for {
            self.rebuild(tabs, target);
            self.built_for = ids;
        }
        self.restyle(tabs);
    }

    fn rebuild(&mut self, tabs: &Tabs, target: &AnyObject) {
        for tab in self.tabs.drain(..) {
            tab.slot.removeFromSuperview();
        }
        if tabs.len() < 2 {
            return;
        }

        for index in 0..tabs.len() {
            let slot = NSView::new(self.mtm);
            slot.setWantsLayer(true);
            if let Some(layer) = slot.layer() {
                layer.setMasksToBounds(true);
                layer.setCornerRadius(CAPSULE_RADIUS);
                layer.setBorderWidth(1.0);
            }

            let capsule = capsule(NSVisualEffectMaterial::Selection, CAPSULE_RADIUS, self.mtm);
            let button = unsafe {
                NSButton::buttonWithTitle_target_action(
                    &NSString::from_str(""),
                    Some(target),
                    Some(self.actions.select_tab),
                    self.mtm,
                )
            };
            button.setBordered(false);
            button.setFont(Some(&NSFont::systemFontOfSize(12.0)));
            button.setTag(index as isize);
            button.setAlignment(NSTextAlignment::Center);
            // Truncation lives on the cell. Setting it on the button alone
            // does not reach the object that actually lays out the title.
            if let Some(cell) = button.cell() {
                cell.setLineBreakMode(NSLineBreakMode::ByTruncatingTail);
                cell.setTruncatesLastVisibleLine(true);
            }

            // The capsule and the label are siblings inside the slot, not
            // nested: the capsule is hidden for inactive tabs, and a hidden
            // view takes its subviews with it — which would leave every
            // unselected tab invisible and unclickable. Capsule first so it
            // sits behind the label.
            slot.addSubview(&capsule);
            slot.addSubview(&button);
            self.strip.addSubview(&slot);
            self.tabs.push(TabButton {
                slot,
                capsule,
                button,
            });
        }
    }

    fn restyle(&self, tabs: &Tabs) {
        for (index, entry) in self.tabs.iter().enumerate() {
            let Some(tab) = tabs.get(index) else { continue };
            let active = index == tabs.active_index();

            entry.button.setTitle(&NSString::from_str(&tab.label()));
            // Only the selected tab is filled, but every tab is outlined —
            // without an edge, a row of centred labels reads as one strip of
            // text rather than as separate tabs.
            entry.capsule.setHidden(!active);
            if let Some(layer) = entry.slot.layer() {
                // Resolved here rather than once at construction: these are
                // dynamic system colours, and `CGColor` snapshots whatever
                // appearance is current at the moment it is asked. `restyle`
                // runs on every switch, navigation and title change, so a
                // light/dark flip is picked up on the next interaction.
                let edge = if active {
                    NSColor::separatorColor()
                } else {
                    NSColor::quaternaryLabelColor()
                };
                layer.setBorderColor(Some(&edge.CGColor()));
            }

            // Three tints, and the third is the one that matters: a discarded
            // tab is still a tab, and dimming it is the only signal the user
            // gets that clicking it costs a reload.
            let tint = if active {
                NSColor::labelColor()
            } else if tab.is_live() {
                NSColor::secondaryLabelColor()
            } else {
                NSColor::tertiaryLabelColor()
            };
            entry.button.setContentTintColor(Some(&tint));
        }
    }

    /// Position everything inside a chrome area `width` wide.
    pub fn layout(&self, width: f64, tab_count: usize) {
        let height = self.height(tab_count);
        let row_mid = height - TOOLBAR_H / 2.0;
        let btn_y = row_mid - BTN / 2.0;

        let mut x = TRAFFIC_LIGHTS;
        for button in [&self.back, &self.forward] {
            button.setFrame(NSRect::new(
                NSPoint::new(x, btn_y),
                NSSize::new(BTN, BTN),
            ));
            x += BTN + GAP;
        }
        let nav_right = x;

        let new_tab_x = (width - PAD - BTN).max(nav_right);
        self.new_tab.setFrame(NSRect::new(
            NSPoint::new(new_tab_x, btn_y),
            NSSize::new(BTN, BTN),
        ));

        // The pill is centred in the window, then pushed off centre only as
        // far as it must be to clear the buttons on a narrow window.
        let left = nav_right + GAP;
        let right = new_tab_x - GAP;
        let room = (right - left).max(80.0);
        let pill_w = room.min(PILL_MAX_W);
        let centred = (width - pill_w) / 2.0;
        let pill_x = centred.clamp(left, (right - pill_w).max(left));
        self.pill.setFrame(NSRect::new(
            NSPoint::new(pill_x, row_mid - PILL_H / 2.0),
            NSSize::new(pill_w, PILL_H),
        ));
        if let Some(badge) = &self.vip_badge {
            let badge_w = 48.0;
            badge.setFrame(NSRect::new(
                NSPoint::new((pill_w - badge_w - 6.0).max(0.0), (PILL_H - 18.0) / 2.0),
                NSSize::new(badge_w, 18.0),
            ));
            self.field.setFrame(NSRect::new(
                NSPoint::new(PAD, (PILL_H - 18.0) / 2.0),
                NSSize::new((pill_w - PAD * 2.0 - badge_w).max(0.0), 18.0),
            ));
        } else {
            self.field.setFrame(NSRect::new(
                NSPoint::new(PAD, (PILL_H - 18.0) / 2.0),
                NSSize::new((pill_w - PAD * 2.0).max(0.0), 18.0),
            ));
        }

        let has_strip = tab_count > 1;
        self.strip.setHidden(!has_strip);
        if !has_strip {
            return;
        }
        self.strip.setFrame(NSRect::new(
            NSPoint::new(0.0, 0.0),
            NSSize::new(width, STRIP_H),
        ));

        // Tabs divide the full width evenly, the way Safari's do — a fixed
        // maximum would leave a ragged gap on the right.
        let count = self.tabs.len().max(1) as f64;
        let each = ((width - PAD * 2.0).max(0.0)) / count;
        for (i, entry) in self.tabs.iter().enumerate() {
            // A little air between slots so neighbouring outlines read as two
            // edges rather than one thick line.
            let slot = NSRect::new(
                NSPoint::new(PAD + i as f64 * each + 2.0, 3.0),
                NSSize::new((each - 4.0).max(0.0), STRIP_H - 6.0),
            );
            entry.slot.setFrame(slot);
            // Children fill the slot, which is what clips them.
            let inner = NSRect::new(NSPoint::new(0.0, 0.0), slot.size);
            entry.capsule.setFrame(inner);
            entry.button.setFrame(inner);
        }
    }
}

/// A rounded, self-theming background panel.
fn capsule(
    material: NSVisualEffectMaterial,
    radius: f64,
    mtm: MainThreadMarker,
) -> Retained<NSVisualEffectView> {
    let view = NSVisualEffectView::new(mtm);
    view.setMaterial(material);
    view.setBlendingMode(NSVisualEffectBlendingMode::WithinWindow);
    view.setState(NSVisualEffectState::Active);
    view.setWantsLayer(true);
    if let Some(layer) = view.layer() {
        layer.setCornerRadius(radius);
        layer.setMasksToBounds(true);
    }
    view
}

fn symbol_button(
    symbol: &str,
    target: &AnyObject,
    action: Sel,
    mtm: MainThreadMarker,
) -> Retained<NSButton> {
    let image = NSImage::imageWithSystemSymbolName_accessibilityDescription(
        &NSString::from_str(symbol),
        None,
    );
    let button = match image {
        Some(image) => unsafe {
            NSButton::buttonWithImage_target_action(&image, Some(target), Some(action), mtm)
        },
        // No SF Symbol by that name on this system: fall back to a titled
        // button rather than an invisible one.
        None => unsafe {
            NSButton::buttonWithTitle_target_action(
                &NSString::from_str(symbol),
                Some(target),
                Some(action),
                mtm,
            )
        },
    };
    button.setBordered(false);
    button.setContentTintColor(Some(&NSColor::labelColor()));
    button
}
