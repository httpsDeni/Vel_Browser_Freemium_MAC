//! The two shapes a supporter key can have, and where each comes from.
//!
//! GitHub Sponsors and Lemon Squeezy solve the same problem in different
//! ways, so Vel accepts a key from either:
//!
//! - **Lemon Squeezy** mints a licence key automatically on purchase and can
//!   be asked about it later. Those are UUIDs, and they mean something — see
//!   [`crate::lemonsqueezy`].
//! - **GitHub Sponsors** has no licence API at all. There is nothing to check
//!   a sponsor's key against, so those are issued by hand with `keygen` and
//!   carry only a checksum. That asymmetry is real and worth knowing about:
//!   a sponsor key is a courtesy, a Lemon Squeezy key is a receipt.

/// Where a key came from, determined by its shape alone.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Source {
    /// `VEL-XXXXXXXX-CCCC`, issued by hand to a GitHub Sponsor.
    Sponsor,
    /// A Lemon Squeezy licence key, `8-4-4-4-12` hex.
    LemonSqueezy,
}

/// Classify a key by shape. `None` means it is not a key at all.
///
/// This is a syntax check, never an entitlement decision — a well-formed
/// Lemon Squeezy key still has to be validated against the API before it
/// grants anything.
pub fn classify(key: &str) -> Option<Source> {
    let key = key.trim();
    if verify_sponsor_key(key) {
        Some(Source::Sponsor)
    } else if is_uuid(key) {
        Some(Source::LemonSqueezy)
    } else {
        None
    }
}

/// Check a sponsor key's shape.
///
/// `VEL-XXXXXXXX-CCCC`, where the last group is an FNV-1a check value over
/// the first. This catches a mistyped or truncated key so that someone who
/// donated gets "that key looks wrong" instead of silently staying on the
/// free tier — which is the entire job. See the module docs in `lib.rs` on
/// why it is not more than this.
pub fn verify_sponsor_key(key: &str) -> bool {
    let key = key.trim();
    let Some(rest) = key.strip_prefix("VEL-") else {
        return false;
    };
    let Some((body, check)) = rest.split_once('-') else {
        return false;
    };
    if body.len() != 8 || check.len() != 4 || !body.chars().all(|c| c.is_ascii_hexdigit()) {
        return false;
    }
    check.eq_ignore_ascii_case(&checksum(body))
}

/// Build a well-formed sponsor key from an 8-hex-digit body.
pub fn make_sponsor_key(body: &str) -> String {
    format!("VEL-{}-{}", body.to_uppercase(), checksum(body))
}

fn checksum(body: &str) -> String {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in body.to_uppercase().as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    format!("{:04X}", (hash & 0xFFFF) as u16)
}

/// Lemon Squeezy licence keys are plain UUIDs.
fn is_uuid(key: &str) -> bool {
    let groups: Vec<&str> = key.split('-').collect();
    groups.len() == 5
        && [8, 4, 4, 4, 12] == groups.iter().map(|g| g.len()).collect::<Vec<_>>()[..]
        && groups
            .iter()
            .all(|g| g.chars().all(|c| c.is_ascii_hexdigit()))
}

#[cfg(test)]
mod tests {
    use super::*;

    const LS_KEY: &str = "38b1460a-5104-4067-a91d-77b872934d51";

    #[test]
    fn generated_sponsor_keys_verify() {
        for body in ["00000000", "DEADBEEF", "0123abcd"] {
            let key = make_sponsor_key(body);
            assert!(verify_sponsor_key(&key), "{key} should verify");
            assert_eq!(classify(&key), Some(Source::Sponsor));
        }
    }

    #[test]
    fn mistyped_sponsor_keys_are_rejected() {
        let good = make_sponsor_key("DEADBEEF");
        // A transposed character must not pass — catching this is the only
        // thing the checksum is for.
        assert!(!verify_sponsor_key(&good.replace("DEADBEEF", "DAEDBEEF")));
        for bad in [
            "",
            "VEL-DEADBEEF",
            "VEL-DEADBEEF-0000",
            "DEADBEEF-1234",
            "VEL-SHORT-1234",
            "VEL-NOTHEXAA-1234",
        ] {
            assert!(!verify_sponsor_key(bad), "{bad:?} should be rejected");
        }
    }

    #[test]
    fn lemon_squeezy_keys_are_recognised() {
        assert_eq!(classify(LS_KEY), Some(Source::LemonSqueezy));
        assert_eq!(classify(&LS_KEY.to_uppercase()), Some(Source::LemonSqueezy));
    }

    #[test]
    fn the_two_formats_never_collide() {
        // A sponsor key must never be mistaken for a licence to validate
        // online, and vice versa — they take completely different paths.
        assert_eq!(classify(&make_sponsor_key("DEADBEEF")), Some(Source::Sponsor));
        assert!(!verify_sponsor_key(LS_KEY));
    }

    #[test]
    fn junk_is_not_a_key() {
        for bad in [
            "",
            "hello",
            "38b1460a-5104-4067-a91d",                  // too few groups
            "38b1460a-5104-4067-a91d-77b872934d5",      // last group short
            "38b1460g-5104-4067-a91d-77b872934d51",     // 'g' is not hex
        ] {
            assert_eq!(classify(bad), None, "{bad:?} should not classify");
        }
    }

    #[test]
    fn keys_survive_being_saved_by_hand() {
        // The key arrives in a file someone edited in TextEdit.
        assert_eq!(classify(&format!("  {LS_KEY}\n")), Some(Source::LemonSqueezy));
        let sponsor = make_sponsor_key("DEADBEEF");
        assert_eq!(classify(&format!("\t{sponsor}  \n")), Some(Source::Sponsor));
    }
}
