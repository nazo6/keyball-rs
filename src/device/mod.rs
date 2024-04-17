//! Device-specific implementations.

#[cfg(feature = "rp2040")]
mod rp2040;

#[cfg(feature = "rp2040")]
pub use rp2040::*;
