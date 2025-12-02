/// Setup code to simulate browser environment for YouTube player execution
pub const SETUP_CODE: &str = include_str!("init_web.js");

/// Intl polyfill for QuickJS compatibility
pub const INTL_POLYFILL: &str = include_str!("intl_polyfill.js");
