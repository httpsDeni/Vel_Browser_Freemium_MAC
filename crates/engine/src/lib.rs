//! The web engine layer: WKWebView, configured for video and kept small.
//!
//! Vel does not ship a rendering engine. Bundling one would mean shipping a
//! second copy of everything macOS already has, and — the part that actually
//! matters here — losing the path that makes 4K60 cheap on Apple silicon.
//! System WebKit hands H.264, HEVC, VP9 and AV1 to VideoToolbox, which
//! decodes on the media engine block rather than the GPU or the CPU, and
//! composites the resulting IOSurfaces through Core Animation into Metal
//! without a copy. A bundled Chromium gets none of that for free.
//!
//! What is left for this crate is to configure that engine correctly and
//! stay out of the frame path. See [`config`] for where the performance
//! actually comes from.

pub mod config;
pub mod page;
pub mod rules;
pub mod script;

pub use config::{Host, Session};
pub use page::Page;
pub use rules::Rules;
pub use script::PageEvent;
