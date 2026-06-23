//! Conditional tracing support.
//!
//! When the `tracing` feature is enabled (default), macros delegate to the
//! real [`tracing`] crate. When disabled, every macro expands to **nothing**
//! — no empty blocks, no unused bindings, zero compiled code.
//!
//! # Usage
//!
//! ```ignore
//! use crate::trace::{debug, error, info, trace_span};
//!
//! trace_span!("my_op", input_len = data.len());
//! info!(result = %value, "Operation complete");
//! debug!(?some_var, "Detailed state");
//! error!(%err, "Something went wrong");
//! ```
//!
//! # Span guard
//!
//! `trace_span!` expands to `let _guard = ...` when tracing is on, and to
//! **nothing** (no `let` binding) when off. Because the guard is unused
//! when the feature is disabled there is no dead-code warning either.

// ── debug! ───────────────────────────────────────────────────────────────────

#[cfg(feature = "tracing")]
#[macro_export]
macro_rules! debug {
    ($($tt:tt)*) => {
        ::tracing::debug!($($tt)*)
    };
}

#[cfg(not(feature = "tracing"))]
#[macro_export]
macro_rules! debug {
    ($($tt:tt)*) => {};
}

// ── info! ────────────────────────────────────────────────────────────────────

#[cfg(feature = "tracing")]
#[macro_export]
macro_rules! info {
    ($($tt:tt)*) => {
        ::tracing::info!($($tt)*)
    };
}

#[cfg(not(feature = "tracing"))]
#[macro_export]
macro_rules! info {
    ($($tt:tt)*) => {};
}

// ── error! ───────────────────────────────────────────────────────────────────

#[cfg(feature = "tracing")]
#[macro_export]
macro_rules! error {
    ($($tt:tt)*) => {
        ::tracing::error!($($tt)*)
    };
}

#[cfg(not(feature = "tracing"))]
#[macro_export]
macro_rules! error {
    ($($tt:tt)*) => {};
}

// ── trace_span! ──────────────────────────────────────────────────────────────
//
// When tracing is enabled this expands to:
//   let _trace_guard = ::tracing::info_span!(...).entered();
// When disabled the macro body is completely empty — no binding is created.

#[cfg(feature = "tracing")]
#[macro_export]
macro_rules! trace_span {
    ($($tt:tt)*) => {
        let _trace_guard = ::tracing::info_span!($($tt)*).entered();
    };
}

#[cfg(not(feature = "tracing"))]
#[macro_export]
macro_rules! trace_span {
    ($($tt:tt)*) => {};
}

// ── Re-exports for module-qualified access ───────────────────────────────────
//
// `#[macro_export]` places macros at the crate root, but these `pub use`
// statements also make them available as `crate::trace::debug!()` etc.
// so callers can write `use crate::trace::{debug, info, trace_span};`.

pub use crate::debug;
pub use crate::error;
pub use crate::info;
pub use crate::trace_span;
