//! Built-in JS Challenge Providers
//!
//! Each backend implements [`crate::runtime::ChallengeSolver`] and exposes a
//! `new` constructor. The concrete provider enum and its dispatch live in
//! `registry`, so this module only declares the enabled backends.

pub mod polyfill;
pub mod preprocessor;

#[cfg(feature = "qjs")]
pub mod quickjs;

#[cfg(feature = "quickjs-oxide")]
pub mod quickjs_oxide;

#[cfg(feature = "boa")]
pub mod boa;

#[cfg(feature = "external")]
pub mod deno;

#[cfg(feature = "external")]
pub mod node;

#[cfg(feature = "external")]
pub mod bun;
