//! Where the store lives and how placed widgets are told to redraw, per platform.

#[cfg(target_vendor = "apple")]
mod apple;
#[cfg(target_vendor = "apple")]
pub(crate) use apple::*;

#[cfg(target_os = "android")]
mod android;
#[cfg(target_os = "android")]
pub(crate) use android::*;

#[cfg(not(any(target_vendor = "apple", target_os = "android")))]
mod other;
#[cfg(not(any(target_vendor = "apple", target_os = "android")))]
pub(crate) use other::*;
