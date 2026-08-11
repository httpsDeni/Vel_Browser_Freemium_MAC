//! Issue a supporter key.
//!
//!     cargo run -p vel-pro --example keygen
//!     cargo run -p vel-pro --example keygen -- 1A2B3C4D
//!
//! Give the printed key to whoever donated; they save it to the path this
//! prints. Read the module docs in `lib.rs` before wiring this to anything
//! automated — the key is a convenience for supporters, not a lock.

fn main() {
    let body = std::env::args().nth(1).unwrap_or_else(random_body);

    if body.len() != 8 || !body.chars().all(|c| c.is_ascii_hexdigit()) {
        eprintln!("keygen: expected 8 hex digits, got {body:?}");
        std::process::exit(1);
    }

    let key = vel_pro::make_sponsor_key(&body);
    debug_assert!(vel_pro::verify_sponsor_key(&key));

    println!("{key}");
    if let Some(path) = vel_pro::key_path() {
        println!();
        println!("The supporter saves it with:");
        println!("  mkdir -p {:?}", path.parent().unwrap_or(&path));
        println!("  echo {key} > {path:?}");
    }
}

/// Eight hex digits from the system clock.
///
/// Not a security property — keys are not secrets, see `lib.rs`. This only
/// needs to avoid handing the same string to two different people in a row.
fn random_body() -> String {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_nanos());
    format!("{:08X}", (nanos as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15) >> 32)
}
