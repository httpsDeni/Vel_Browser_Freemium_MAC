//! The one script Vel injects into every page.
//!
//! Kept to a few hundred bytes on purpose: it runs on every navigation of
//! every frame, so anything expensive here is a tax on every page load. It
//! does two jobs that have no native equivalent in the public WebKit SDK.

/// Name of the `window.webkit.messageHandlers` channel.
pub const CHANNEL: &str = "vel";

/// Build the injected source for one tab.
///
/// The tab's id is baked into the source rather than recovered on the native
/// side, because `WKScriptMessage` identifies the *page* that sent a message,
/// not the tab the application filed it under. Substituting it here costs one
/// small allocation per tab and removes a lookup from the message path.
///
/// Messages are pipe-separated strings, not objects. A posted JS object
/// arrives as an `NSDictionary` of bridged `NSNumber`s and `NSString`s, and
/// unpacking that costs more allocations than the message carries
/// information. `"a|7|1"` decodes with a `split`.
pub fn for_tab(tab_id: u64) -> String {
    format!(
        r#"(function(){{
if (window.__vel) return; window.__vel = 1;
var TAB = {tab_id};
function send(m) {{ try {{ window.webkit.messageHandlers.{CHANNEL}.postMessage(m); }} catch (e) {{}} }}

/* Audibility.
   The tab discarder needs to know whether this page is making noise, because
   a tab playing audio must survive any amount of time in the background.
   WKWebView has no public property for it, so the page reports it. Media
   events do not bubble, hence capture-phase listeners on document. */
var audible = false;
function recheck() {{
  var now = false, m = document.querySelectorAll('video, audio');
  for (var i = 0; i < m.length; i++) {{
    if (!m[i].paused && !m[i].ended && !m[i].muted && m[i].volume > 0) {{ now = true; break; }}
  }}
  if (now !== audible) {{ audible = now; send('a|' + TAB + '|' + (audible ? 1 : 0)); }}
}}
['play','pause','ended','volumechange','emptied'].forEach(function (e) {{
  document.addEventListener(e, recheck, true);
}});

/* Same-document navigation.
   WKNavigationDelegate has no callback for it — pushState fires nothing — so
   on any single-page site, which is to say YouTube and Twitch, the tab title
   and the address bar would freeze on whatever was loaded first.

   The message deliberately carries no payload beyond the tab id. It is a
   nudge, not a report: the native side re-reads title and URL from the web
   view itself. A page can call postMessage on this channel with anything it
   likes, and letting it hand us a URL string to display would be an address
   bar spoof. WebKit's own URL cannot be forged. */
var pending = 0;
function changed() {{
  clearTimeout(pending);
  pending = setTimeout(function () {{ send('s|' + TAB); }}, 120);
}}
['pushState','replaceState'].forEach(function (m) {{
  var original = history[m];
  history[m] = function () {{ var r = original.apply(this, arguments); changed(); return r; }};
}});
window.addEventListener('popstate', changed);
window.addEventListener('hashchange', changed);
/* Watch the title element only — never the document.
   A MutationObserver on documentElement with subtree:true would run its
   callback on every DOM mutation on the page, which on YouTube is hundreds a
   second. Observing the one node that matters costs nothing, and head's
   childList catches single-page apps that replace the title node wholesale
   rather than editing it. */
var titleWatcher = new MutationObserver(changed);
function watchTitle() {{
  var t = document.querySelector('title');
  if (t) titleWatcher.observe(t, {{ childList: true, characterData: true, subtree: true }});
}}
watchTitle();
if (document.head) {{
  new MutationObserver(watchTitle).observe(document.head, {{ childList: true }});
}}

/* Picture-in-picture.
   Targets the video the user is actually watching: the largest one that is
   playing, else the largest one present.

   webkitSetPresentationMode is WebKit's own API and comes first deliberately.
   The standardised requestPictureInPicture() requires a transient user
   activation, and evaluateJavaScript: from native code cannot supply one — a
   keyboard shortcut driving the standard API would simply be rejected. The
   legacy Safari API has no such requirement. */
window.__velPip = function () {{
  var vids = [].slice.call(document.querySelectorAll('video')).filter(function (v) {{ return v.readyState > 0; }});
  if (!vids.length) return false;
  function area(v) {{ return (v.videoWidth * v.videoHeight) || (v.clientWidth * v.clientHeight); }}
  var playing = vids.filter(function (v) {{ return !v.paused; }});
  var target = (playing.length ? playing : vids).sort(function (a, b) {{ return area(b) - area(a); }})[0];
  if (typeof target.webkitSetPresentationMode === 'function') {{
    target.webkitSetPresentationMode(
      target.webkitPresentationMode === 'picture-in-picture' ? 'inline' : 'picture-in-picture');
    return true;
  }}
  if (document.pictureInPictureElement) {{ document.exitPictureInPicture(); return true; }}
  if (target.requestPictureInPicture) {{ target.requestPictureInPicture().catch(function () {{}}); return true; }}
  return false;
}};
}})();"#
    )
}

/// Expression the app evaluates for the picture-in-picture shortcut.
pub const TOGGLE_PIP: &str = "window.__velPip && window.__velPip()";

/// Messages the injected script can send, already decoded.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PageEvent {
    /// The page started or stopped producing sound.
    Audible { tab: u64, playing: bool },
    /// The page navigated within itself, or retitled. Carries no detail on
    /// purpose — the receiver re-reads title and URL from the web view.
    StateChanged { tab: u64 },
}

/// Decode one message from the injected script.
///
/// Anything unrecognised returns `None`. This parser reads data that a web
/// page can reach — a page cannot forge the channel, but it *can* call
/// `postMessage` on it with whatever it likes — so nothing here may panic or
/// trust a field. Note in particular that the tab id is untrusted: see
/// `Browser::set_audible`, which resolves it through the tab list rather
/// than indexing with it.
pub fn parse_event(raw: &str) -> Option<PageEvent> {
    let mut parts = raw.split('|');
    let kind = parts.next()?;
    let tab = parts.next()?.parse().ok()?;
    match kind {
        "a" => Some(PageEvent::Audible {
            tab,
            playing: parts.next()? == "1",
        }),
        "s" => Some(PageEvent::StateChanged { tab }),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_audibility() {
        assert_eq!(
            parse_event("a|7|1"),
            Some(PageEvent::Audible { tab: 7, playing: true })
        );
        assert_eq!(
            parse_event("a|7|0"),
            Some(PageEvent::Audible { tab: 7, playing: false })
        );
    }

    #[test]
    fn decodes_state_changes() {
        assert_eq!(parse_event("s|4"), Some(PageEvent::StateChanged { tab: 4 }));
    }

    #[test]
    fn rejects_junk_without_panicking() {
        for junk in ["", "a", "a|", "a|x|1", "a|-1|1", "z|1|1", "a|99999999999999999999|1", "s|x"] {
            assert_eq!(parse_event(junk), None, "{junk:?} should not decode");
        }
    }

    #[test]
    fn tab_id_reaches_the_script() {
        assert!(for_tab(42).contains("var TAB = 42;"));
    }
}
