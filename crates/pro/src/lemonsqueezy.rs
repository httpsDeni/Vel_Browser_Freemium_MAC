//! Talking to the Lemon Squeezy licence API.
//!
//! Three things about this module are deliberate.
//!
//! **It ships no credential.** The licence endpoints
//! (`/v1/licenses/activate`, `/validate`, `/deactivate`) take no API key —
//! they are meant to be called from client apps. If a change here ever seems
//! to need a token, it is the wrong change: anyone can read a token out of
//! the binary, and a leaked Lemon Squeezy API key can read your orders and
//! your customers.
//!
//! **It checks the product id.** `/validate` answers for the whole platform,
//! not for your store. Without comparing `meta.product_id`, a valid licence
//! for somebody else's product would unlock Vel. When
//! [`crate::config::is_configured`] is false we refuse everything rather than
//! accept everything.
//!
//! **It distinguishes "no" from "could not ask".** A refusal from the server
//! is authoritative and downgrades the tier. A timeout, a captive portal, or
//! a plane is not, and must leave a supporter alone — see [`Verdict`].
//!
//! Networking is `NSURLSession`, so this adds no HTTP stack to the binary,
//! and the session's delegate queue is the main queue so callbacks land where
//! the rest of the app already lives.

use block2::RcBlock;
use objc2_foundation::{
    NSData, NSError, NSMutableURLRequest, NSOperationQueue, NSString, NSURLResponse, NSURLSession,
    NSURLSessionConfiguration, NSURL,
};

use crate::config;

const ACTIVATE_URL: &str = "https://api.lemonsqueezy.com/v1/licenses/activate";
const VALIDATE_URL: &str = "https://api.lemonsqueezy.com/v1/licenses/validate";

/// Seconds before a licence call gives up.
///
/// Short on purpose: nothing waits on this, so a long timeout would only mean
/// holding a socket open for a user who is already browsing happily.
const TIMEOUT: f64 = 15.0;

/// What the server said, or that it did not say anything.
#[derive(Debug, Clone)]
pub struct Verdict {
    /// Whether this licence entitles the user to supporter features.
    pub granted: bool,
    /// Set by a successful activation; needed to validate this device later.
    pub instance_id: Option<String>,
    /// Whether the server actually answered.
    ///
    /// The distinction matters more than `granted` does. `granted: false` with
    /// `authoritative: false` means "we could not reach Lemon Squeezy", and the
    /// caller must keep whatever entitlement it already had. Only an
    /// authoritative refusal may take features away — otherwise a supporter on
    /// a train loses their browser.
    pub authoritative: bool,
    /// Human-readable detail, for logs and the status menu item.
    pub detail: String,
}

impl Verdict {
    fn unreachable(detail: impl Into<String>) -> Self {
        Self {
            granted: false,
            instance_id: None,
            authoritative: false,
            detail: detail.into(),
        }
    }

    fn refused(detail: impl Into<String>) -> Self {
        Self {
            granted: false,
            instance_id: None,
            authoritative: true,
            detail: detail.into(),
        }
    }
}

/// Claim this device against a licence key.
///
/// Run once, when a key first appears. The returned `instance_id` is what
/// makes the product's activation limit mean anything — without it every
/// install of Vel would look like the same one to Lemon Squeezy.
pub fn activate(key: &str, on_done: impl Fn(Verdict) + 'static) {
    let body = format!(
        "license_key={}&instance_name={}",
        form_encode(key),
        form_encode(config::INSTANCE_NAME)
    );
    post(ACTIVATE_URL, &body, move |json| {
        on_done(read_verdict(json, "activated"))
    });
}

/// Re-check a licence Vel has already activated.
pub fn validate(key: &str, instance_id: Option<&str>, on_done: impl Fn(Verdict) + 'static) {
    let mut body = format!("license_key={}", form_encode(key));
    if let Some(id) = instance_id {
        body.push_str(&format!("&instance_id={}", form_encode(id)));
    }
    post(VALIDATE_URL, &body, move |json| {
        on_done(read_verdict(json, "valid"))
    });
}

/// Turn a licence API response into a verdict.
///
/// `flag` is the field that means success — `activated` on one endpoint,
/// `valid` on the other; the rest of the envelope is identical.
fn read_verdict(json: Result<serde_json::Value, String>, flag: &str) -> Verdict {
    let json = match json {
        Ok(json) => json,
        // A transport failure is never a refusal.
        Err(e) => return Verdict::unreachable(e),
    };

    if !config::is_configured() {
        return Verdict::refused(
            "Lemon Squeezy is not configured in this build (see crates/pro/src/config.rs)",
        );
    }

    if !json.get(flag).and_then(serde_json::Value::as_bool).unwrap_or(false) {
        let reason = json
            .get("error")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("licence rejected");
        return Verdict::refused(reason.to_string());
    }

    // The licence is real — but is it a licence for *this* product?
    let product = json
        .pointer("/meta/product_id")
        .and_then(serde_json::Value::as_u64);
    if product != Some(config::PRODUCT_ID) {
        return Verdict::refused(format!(
            "licence belongs to product {} , not {}",
            product.map_or("unknown".to_string(), |p| p.to_string()),
            config::PRODUCT_ID
        ));
    }

    let status = json
        .pointer("/license_key/status")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("unknown");
    // `inactive` is a key that exists but has never been activated, which is
    // the normal state right before activation succeeds.
    if !matches!(status, "active" | "inactive") {
        return Verdict::refused(format!("licence is {status}"));
    }

    Verdict {
        granted: true,
        instance_id: json
            .pointer("/instance/id")
            .and_then(serde_json::Value::as_str)
            .map(str::to_string),
        authoritative: true,
        detail: format!("licence {status}"),
    }
}

/// POST a form body and hand the parsed JSON to `on_done` on the main thread.
fn post(url: &str, body: &str, on_done: impl Fn(Result<serde_json::Value, String>) + 'static) {
    let Some(url) = NSURL::URLWithString(&NSString::from_str(url)) else {
        on_done(Err("bad endpoint URL".into()));
        return;
    };

    let request = NSMutableURLRequest::requestWithURL_cachePolicy_timeoutInterval(
        &url,
        objc2_foundation::NSURLRequestCachePolicy::ReloadIgnoringLocalCacheData,
        TIMEOUT,
    );
    request.setHTTPMethod(&NSString::from_str("POST"));
    request.setValue_forHTTPHeaderField(
        Some(&NSString::from_str("application/json")),
        &NSString::from_str("Accept"),
    );
    request.setValue_forHTTPHeaderField(
        Some(&NSString::from_str("application/x-www-form-urlencoded")),
        &NSString::from_str("Content-Type"),
    );
    request.setHTTPBody(Some(&NSData::with_bytes(body.as_bytes())));

    // Ephemeral: a licence check has no business writing cookies or a cache to
    // disk, and it shares nothing with the browsing session.
    let config = NSURLSessionConfiguration::ephemeralSessionConfiguration();
    let session = unsafe {
        NSURLSession::sessionWithConfiguration_delegate_delegateQueue(
            &config,
            None,
            // Completion on the main queue, so callers can touch UI without
            // hopping threads themselves.
            Some(&NSOperationQueue::mainQueue()),
        )
    };

    let handler = RcBlock::new(
        move |data: *mut NSData, _response: *mut NSURLResponse, error: *mut NSError| {
            if let Some(error) = unsafe { error.as_ref() } {
                on_done(Err(error.localizedDescription().to_string()));
                return;
            }
            let Some(data) = (unsafe { data.as_ref() }) else {
                on_done(Err("empty response".into()));
                return;
            };
            match serde_json::from_slice::<serde_json::Value>(&data.to_vec()) {
                Ok(json) => on_done(Ok(json)),
                Err(e) => on_done(Err(format!("unreadable response: {e}"))),
            }
        },
    );

    let task = unsafe { session.dataTaskWithRequest_completionHandler(&request, &handler) };
    task.resume();
}

/// Percent-encode one form field.
fn form_encode(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for byte in text.as_bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(*byte as char)
            }
            b' ' => out.push('+'),
            _ => out.push_str(&format!("%{byte:02X}")),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn ok_body(product: u64, status: &str) -> serde_json::Value {
        json!({
            "valid": true,
            "error": null,
            "license_key": { "id": 1, "status": status, "key": "x" },
            "instance": { "id": "inst-123", "name": "Vel for macOS" },
            "meta": { "store_id": 1, "product_id": product }
        })
    }

    /// The one that protects a supporter: a network failure must never look
    /// like a refusal, or people lose features on a bad connection.
    #[test]
    fn transport_failure_is_not_authoritative() {
        let v = read_verdict(Err("offline".into()), "valid");
        assert!(!v.granted);
        assert!(!v.authoritative, "offline must not revoke anything");
    }

    #[test]
    fn a_rejected_licence_is_authoritative() {
        let body = json!({ "valid": false, "error": "license_key not found" });
        let v = read_verdict(Ok(body), "valid");
        assert!(!v.granted);
        assert!(v.authoritative);
        assert!(v.detail.contains("not found"));
    }

    /// Without this check any valid Lemon Squeezy licence — for any product
    /// by any seller — would unlock Vel.
    #[test]
    fn a_licence_for_another_product_is_refused() {
        if !config::is_configured() {
            return; // covered by `unconfigured_builds_refuse_everything`
        }
        let v = read_verdict(Ok(ok_body(config::PRODUCT_ID + 1, "active")), "valid");
        assert!(!v.granted, "another seller's licence must not unlock Vel");
        assert!(v.authoritative);
    }

    #[test]
    fn unconfigured_builds_refuse_everything() {
        if config::is_configured() {
            return;
        }
        // Fail closed: with no product id to compare against, accepting the
        // response would accept every licence on the platform.
        let v = read_verdict(Ok(ok_body(1234, "active")), "valid");
        assert!(!v.granted);
        assert!(v.detail.contains("not configured"));
    }

    #[test]
    fn expired_and_disabled_licences_are_refused() {
        if !config::is_configured() {
            return;
        }
        for status in ["expired", "disabled"] {
            let v = read_verdict(Ok(ok_body(config::PRODUCT_ID, status)), "valid");
            assert!(!v.granted, "{status} must not grant");
            assert!(v.authoritative);
        }
    }

    #[test]
    fn form_encoding_escapes_what_it_must() {
        assert_eq!(form_encode("Vel for macOS"), "Vel+for+macOS");
        assert_eq!(form_encode("a&b=c"), "a%26b%3Dc");
        // Licence keys must survive untouched.
        let key = "38b1460a-5104-4067-a91d-77b872934d51";
        assert_eq!(form_encode(key), key);
    }
}
