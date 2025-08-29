// Common modules that should work on all targets
pub mod iss;

// WASM-specific modules and exports
#[cfg(target_arch = "wasm32")]
pub mod wasm;

#[cfg(target_arch = "wasm32")]
pub use wasm::*;

// Native-specific exports
#[cfg(not(target_arch = "wasm32"))]
pub use iss::Iss;