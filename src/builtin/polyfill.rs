//! Polyfill code for JavaScript runtime compatibility
//!
//! This module provides setup code and polyfills needed to simulate
//! a browser environment for YouTube player execution.

/// Setup code to simulate browser environment for YouTube player execution
pub const SETUP_CODE: &str = include_str!("polyfill/setup.js");

/// Intl polyfill for QuickJS compatibility
pub const INTL_POLYFILL: &str = include_str!("polyfill/intl.js");
