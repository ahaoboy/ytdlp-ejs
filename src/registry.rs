//! JS Challenge Provider Registry
//!
//! This module manages runtime types and provider creation using enum dispatch.

use crate::builtin::JsRuntimeProvider;
use crate::provider::JsChallengeError;

/// Runtime type for JavaScript execution
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeType {
    #[cfg(feature = "qjs")]
    QuickJS,
    #[cfg(feature = "boa")]
    Boa,
    #[cfg(feature = "external")]
    Deno,
    #[cfg(feature = "external")]
    Node,
    #[cfg(feature = "external")]
    Bun,
}

impl RuntimeType {
    /// Parse runtime type from string
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            #[cfg(feature = "qjs")]
            "qjs" | "quickjs" => Some(Self::QuickJS),
            #[cfg(feature = "external")]
            "deno" => Some(Self::Deno),
            #[cfg(feature = "boa")]
            "boa" => Some(Self::Boa),
            #[cfg(feature = "external")]
            "node" | "nodejs" => Some(Self::Node),
            #[cfg(feature = "external")]
            "bun" => Some(Self::Bun),
            _ => None,
        }
    }

    /// Get list of available runtime names
    pub fn available_runtimes() -> Vec<&'static str> {
        vec![
            #[cfg(feature = "qjs")]
            "qjs",
            #[cfg(feature = "external")]
            "deno",
            #[cfg(feature = "boa")]
            "boa",
            #[cfg(feature = "external")]
            "node",
            #[cfg(feature = "external")]
            "bun",
        ]
    }

    /// Create a provider instance for the specified runtime type
    pub fn create_provider(&self, code: &str) -> Result<JsRuntimeProvider, JsChallengeError> {
        match self {
            #[cfg(feature = "qjs")]
            RuntimeType::QuickJS => Ok(JsRuntimeProvider::QuickJS(
                crate::builtin::quickjs::QuickJSJCP::new(code)?,
            )),
            #[cfg(feature = "boa")]
            RuntimeType::Boa => Ok(JsRuntimeProvider::Boa(Box::new(
                crate::builtin::boa::BoaJCP::new(code)?,
            ))),
            #[cfg(feature = "external")]
            RuntimeType::Deno => Ok(JsRuntimeProvider::Deno(crate::builtin::deno::DenoJCP::new(
                code,
            ))),
            #[cfg(feature = "external")]
            RuntimeType::Node => Ok(JsRuntimeProvider::Node(crate::builtin::node::NodeJCP::new(
                code,
            ))),
            #[cfg(feature = "external")]
            RuntimeType::Bun => Ok(JsRuntimeProvider::Bun(crate::builtin::bun::BunJCP::new(
                code,
            ))),
        }
    }
}
