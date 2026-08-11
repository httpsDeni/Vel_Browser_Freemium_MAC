//! The menu bar.
//!
//! It exists mostly to carry keyboard shortcuts: macOS routes Cmd-key
//! equivalents through the main menu, so an item here is how a shortcut gets
//! registered at all. The Edit menu is not decoration either — without
//! `cut:`/`copy:`/`paste:`/`selectAll:` in the menu, those shortcuts do
//! nothing in the address field, because the field relies on the responder
//! chain reaching those standard selectors.

use objc2::rc::Retained;
use objc2::runtime::{AnyObject, Sel};
use objc2::{sel, MainThreadMarker, MainThreadOnly};
use objc2_app_kit::{NSApplication, NSEventModifierFlags, NSMenu, NSMenuItem};
use objc2_foundation::NSString;
use vel_pro::{Entitlements, Feature};

const CMD: NSEventModifierFlags = NSEventModifierFlags::Command;

pub fn install(
    app: &NSApplication,
    target: &AnyObject,
    entitlements: Entitlements,
    mtm: MainThreadMarker,
) {
    let main = NSMenu::new(mtm);

    // --- application -------------------------------------------------------
    let app_menu = NSMenu::new(mtm);
    item(&app_menu, "About Vel", Some(sel!(orderFrontStandardAboutPanel:)), "", CMD, None, mtm);
    app_menu.addItem(&NSMenuItem::separatorItem(mtm));
    if entitlements.is_supporter() {
        // No call to action for someone who already paid.
        let status = item(&app_menu, "Supporter — thank you", None, "", CMD, None, mtm);
        status.setEnabled(false);
    } else {
        item(
            &app_menu,
            "Sponsor Vel on GitHub…",
            Some(sel!(openSponsors:)),
            "",
            CMD,
            Some(target),
            mtm,
        );
        item(
            &app_menu,
            "Buy a Supporter Licence…",
            Some(sel!(openCheckout:)),
            "",
            CMD,
            Some(target),
            mtm,
        );
    }
    app_menu.addItem(&NSMenuItem::separatorItem(mtm));
    item(&app_menu, "Hide Vel", Some(sel!(hide:)), "h", CMD, None, mtm);
    item(
        &app_menu,
        "Hide Others",
        Some(sel!(hideOtherApplications:)),
        "h",
        CMD | NSEventModifierFlags::Option,
        None,
        mtm,
    );
    app_menu.addItem(&NSMenuItem::separatorItem(mtm));
    item(&app_menu, "Quit Vel", Some(sel!(terminate:)), "q", CMD, None, mtm);
    submenu(&main, "Vel", app_menu, mtm);

    // --- file --------------------------------------------------------------
    let file = NSMenu::new(mtm);
    item(&file, "New Tab", Some(sel!(newTab:)), "t", CMD, Some(target), mtm);
    item(&file, "Close Tab", Some(sel!(closeTab:)), "w", CMD, Some(target), mtm);
    submenu(&main, "File", file, mtm);

    // --- edit --------------------------------------------------------------
    // Nil-targeted so they travel the responder chain to whatever has focus.
    let edit = NSMenu::new(mtm);
    item(&edit, "Undo", Some(sel!(undo:)), "z", CMD, None, mtm);
    item(&edit, "Redo", Some(sel!(redo:)), "z", CMD | NSEventModifierFlags::Shift, None, mtm);
    edit.addItem(&NSMenuItem::separatorItem(mtm));
    item(&edit, "Cut", Some(sel!(cut:)), "x", CMD, None, mtm);
    item(&edit, "Copy", Some(sel!(copy:)), "c", CMD, None, mtm);
    item(&edit, "Paste", Some(sel!(paste:)), "v", CMD, None, mtm);
    item(&edit, "Select All", Some(sel!(selectAll:)), "a", CMD, None, mtm);
    submenu(&main, "Edit", edit, mtm);

    // --- view --------------------------------------------------------------
    let view = NSMenu::new(mtm);
    item(&view, "Open Location…", Some(sel!(focusAddress:)), "l", CMD, Some(target), mtm);
    view.addItem(&NSMenuItem::separatorItem(mtm));
    item(&view, "Reload Page", Some(sel!(reloadPage:)), "r", CMD, Some(target), mtm);
    item(
        &view,
        "Reload Ignoring Cache",
        Some(sel!(forceReload:)),
        "r",
        CMD | NSEventModifierFlags::Shift,
        Some(target),
        mtm,
    );
    item(&view, "Stop", Some(sel!(stopPage:)), ".", CMD, Some(target), mtm);
    view.addItem(&NSMenuItem::separatorItem(mtm));
    // Locked items stay enabled and say why, rather than greying out with no
    // explanation. Choosing one opens the donation page.
    item(
        &view,
        &gated("Picture in Picture", Feature::PictureInPicture, entitlements),
        Some(sel!(togglePip:)),
        "p",
        CMD | NSEventModifierFlags::Shift,
        Some(target),
        mtm,
    );
    item(
        &view,
        "Enter Full Screen",
        Some(sel!(toggleFullScreen:)),
        "f",
        CMD | NSEventModifierFlags::Control,
        None,
        mtm,
    );
    submenu(&main, "View", view, mtm);

    // --- history -----------------------------------------------------------
    let history = NSMenu::new(mtm);
    item(&history, "Back", Some(sel!(goBackPage:)), "[", CMD, Some(target), mtm);
    item(&history, "Forward", Some(sel!(goForwardPage:)), "]", CMD, Some(target), mtm);
    submenu(&main, "History", history, mtm);

    // --- window ------------------------------------------------------------
    let window = NSMenu::new(mtm);
    item(&window, "Minimize", Some(sel!(performMiniaturize:)), "m", CMD, None, mtm);
    window.addItem(&NSMenuItem::separatorItem(mtm));
    item(
        &window,
        "Show Next Tab",
        Some(sel!(nextTab:)),
        "]",
        CMD | NSEventModifierFlags::Shift,
        Some(target),
        mtm,
    );
    item(
        &window,
        "Show Previous Tab",
        Some(sel!(previousTab:)),
        "[",
        CMD | NSEventModifierFlags::Shift,
        Some(target),
        mtm,
    );

    // Cmd+1..9. These have to be real menu items: on macOS the main menu is
    // what registers a key equivalent, and a shortcut with no item behind it
    // simply never fires. Each carries its number as the item's tag, so one
    // action serves all nine.
    window.addItem(&NSMenuItem::separatorItem(mtm));
    for n in 1..=9 {
        let title = if n == 9 {
            "Show Last Tab".to_string()
        } else {
            format!("Show Tab {n}")
        };
        let entry = item(
            &window,
            &title,
            Some(sel!(selectNumberedTab:)),
            &n.to_string(),
            CMD,
            Some(target),
            mtm,
        );
        entry.setTag(n);
    }

    submenu(&main, "Window", window, mtm);

    app.setMainMenu(Some(&main));
}

/// Append a hint to a menu title when the feature behind it is locked.
fn gated(title: &str, feature: Feature, entitlements: Entitlements) -> String {
    if entitlements.allows(feature) {
        title.to_string()
    } else {
        format!("{title} (Supporter)")
    }
}

fn submenu(main: &NSMenu, title: &str, menu: Retained<NSMenu>, mtm: MainThreadMarker) {
    let holder = NSMenuItem::new(mtm);
    menu.setTitle(&NSString::from_str(title));
    holder.setSubmenu(Some(&menu));
    main.addItem(&holder);
}

fn item(
    menu: &NSMenu,
    title: &str,
    action: Option<Sel>,
    key: &str,
    mods: NSEventModifierFlags,
    target: Option<&AnyObject>,
    mtm: MainThreadMarker,
) -> Retained<NSMenuItem> {
    let entry = unsafe {
        NSMenuItem::initWithTitle_action_keyEquivalent(
            NSMenuItem::alloc(mtm),
            &NSString::from_str(title),
            action,
            &NSString::from_str(key),
        )
    };
    entry.setKeyEquivalentModifierMask(mods);
    if let Some(target) = target {
        unsafe { entry.setTarget(Some(target)) };
    }
    menu.addItem(&entry);
    entry
}
