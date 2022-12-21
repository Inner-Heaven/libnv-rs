//! Fake definitions good enough to cross-build libnv's docs
//!
//! docs.rs does all of its builds on Linux, so the usual build script fails.
//! As a workaround, we skip the usual build script when doing cross-builds, and
//! define these stubs instead.
pub type nvlist_t = *const i32;
