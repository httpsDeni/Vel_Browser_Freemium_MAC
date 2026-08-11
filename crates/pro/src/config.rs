//! Everything that changes when you point Vel at your own accounts.
//!
//! This is the only file you edit to set up funding. Nothing here is a secret:
//! the Lemon Squeezy licence endpoints take no API key, so a Vel build ships
//! no credential of any kind. Do not add one — if a future change seems to
//! need an API token in the client, it is the wrong change, because anyone can
//! read it straight out of the binary.
//!
//! # Setting it up
//!
//! See the "Funding" section of the README for the click-by-click version.
//! The short form: create the product in Lemon Squeezy, turn licence keys on,
//! then paste the three values below.

/// GitHub Sponsors profile. Sponsors are issued keys by hand — GitHub has no
/// licence API, so there is nothing to validate against.
pub const SPONSORS_URL: &str = "https://github.com/sponsors/httpsDeni";

/// Lemon Squeezy checkout link for the supporter product.
///
/// From the dashboard: Products → your product → Share → copy the buy link.
pub const CHECKOUT_URL: &str = "https://vel.lemonsqueezy.com/checkout/buy/28763ca3-b0e6-43d6-af26-037f6febc669";

/// Lemon Squeezy store id, from Settings → Stores.
pub const STORE_ID: u64 = 2008042;

/// Lemon Squeezy product id, from the product's URL in the dashboard.
///
/// **This one matters for correctness, not just for links.** `/licenses/validate`
/// checks a key against all of Lemon Squeezy, not against your store — so
/// without comparing `meta.product_id` here, a valid licence for somebody
/// else's product would unlock Vel. [`is_configured`] returns false while this
/// is zero, and the validator refuses to grant anything rather than accepting
/// every licence on the platform.
pub const PRODUCT_ID: u64 = 1284222;

/// Has someone filled in the values above?
///
/// Until they have, Lemon Squeezy keys are rejected. Failing closed is the
/// only safe default: the alternative is a build that treats any licence from
/// any seller as a valid Vel licence.
pub const fn is_configured() -> bool {
    PRODUCT_ID != 0 && STORE_ID != 0
}

/// How long a cached validation is trusted before Vel re-checks.
///
/// Thirty days is a deliberate choice about what this check is for. It is long
/// enough that a supporter who is offline, travelling, or simply not thinking
/// about licences never notices it, and that is the point — see the module
/// docs in `lib.rs` on why this is not anti-piracy. It only needs to
/// eventually notice a refund.
pub const REVALIDATE_AFTER_DAYS: u64 = 30;

/// Name Vel reports when activating a device.
pub const INSTANCE_NAME: &str = "Vel for macOS";
