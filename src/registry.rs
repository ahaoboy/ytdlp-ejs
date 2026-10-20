//! JS Challenge Provider Registry
//!
//! The variants and matches are written out explicitly: there are few
//! engines, and a readable match is easier to maintain than generated code.
//! Adding a runtime means adding its module, one variant here, and the
//! corresponding match arms.

use crate::provider::JsChallengeError;
use crate::runtime::ChallengeSolver;

/// Runtime type for JavaScript execution.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeType {
    #[cfg(feature = "quickjs-oxide")]
    QuickJsOxide,
    #[cfg(feature = "qjs")]
    QuickJS,
    #[cfg(feature = "boa")]
    Boa,
    #[cfg(feature = "external")]
    Node,
    #[cfg(feature = "external")]
    Deno,
    #[cfg(feature = "external")]
    Bun,
}

/// Concrete challenge provider selected by [`RuntimeType`].
pub enum JsRuntimeProvider {
    #[cfg(feature = "quickjs-oxide")]
    QuickJsOxide(crate::builtin::quickjs_oxide::QuickJsOxideJCP),
    #[cfg(feature = "qjs")]
    QuickJS(crate::builtin::quickjs::QuickJSJCP),
    #[cfg(feature = "boa")]
    Boa(Box<crate::builtin::boa::BoaJCP>),
    #[cfg(feature = "external")]
    Node(crate::builtin::node::NodeJCP),
    #[cfg(feature = "external")]
    Deno(crate::builtin::deno::DenoJCP),
    #[cfg(feature = "external")]
    Bun(crate::builtin::bun::BunJCP),
}

impl ChallengeSolver for JsRuntimeProvider {
    fn solve_n(&mut self, challenge: &str) -> Result<String, JsChallengeError> {
        match self {
            #[cfg(feature = "quickjs-oxide")]
            Self::QuickJsOxide(provider) => provider.solve_n(challenge),
            #[cfg(feature = "qjs")]
            Self::QuickJS(provider) => provider.solve_n(challenge),
            #[cfg(feature = "boa")]
            Self::Boa(provider) => provider.solve_n(challenge),
            #[cfg(feature = "external")]
            Self::Node(provider) => provider.solve_n(challenge),
            #[cfg(feature = "external")]
            Self::Deno(provider) => provider.solve_n(challenge),
            #[cfg(feature = "external")]
            Self::Bun(provider) => provider.solve_n(challenge),
        }
    }

    fn solve_sig(&mut self, challenge: &str) -> Result<String, JsChallengeError> {
        match self {
            #[cfg(feature = "quickjs-oxide")]
            Self::QuickJsOxide(provider) => provider.solve_sig(challenge),
            #[cfg(feature = "qjs")]
            Self::QuickJS(provider) => provider.solve_sig(challenge),
            #[cfg(feature = "boa")]
            Self::Boa(provider) => provider.solve_sig(challenge),
            #[cfg(feature = "external")]
            Self::Node(provider) => provider.solve_sig(challenge),
            #[cfg(feature = "external")]
            Self::Deno(provider) => provider.solve_sig(challenge),
            #[cfg(feature = "external")]
            Self::Bun(provider) => provider.solve_sig(challenge),
        }
    }
}

impl RuntimeType {
    /// Parse a canonical runtime name (case-insensitive).
    pub fn parse(name: &str) -> Option<Self> {
        match name.to_ascii_lowercase().as_str() {
            #[cfg(feature = "quickjs-oxide")]
            "quickjs-oxide" => Some(Self::QuickJsOxide),
            #[cfg(feature = "qjs")]
            "qjs" => Some(Self::QuickJS),
            #[cfg(feature = "boa")]
            "boa" => Some(Self::Boa),
            #[cfg(feature = "external")]
            "node" => Some(Self::Node),
            #[cfg(feature = "external")]
            "deno" => Some(Self::Deno),
            #[cfg(feature = "external")]
            "bun" => Some(Self::Bun),
            _ => None,
        }
    }

    /// Names of every enabled runtime, in default-priority order.
    pub fn available_runtimes() -> Vec<&'static str> {
        vec![
            #[cfg(feature = "quickjs-oxide")]
            "quickjs-oxide",
            #[cfg(feature = "qjs")]
            "qjs",
            #[cfg(feature = "boa")]
            "boa",
            #[cfg(feature = "external")]
            "node",
            #[cfg(feature = "external")]
            "deno",
            #[cfg(feature = "external")]
            "bun",
        ]
    }

    /// The first enabled embedded runtime, then the first enabled external
    /// one.
    #[allow(unreachable_code)]
    pub fn default_runtime() -> Option<Self> {
        #[cfg(feature = "quickjs-oxide")]
        return Some(Self::QuickJsOxide);
        #[cfg(feature = "qjs")]
        return Some(Self::QuickJS);
        #[cfg(feature = "boa")]
        return Some(Self::Boa);
        #[cfg(feature = "external")]
        return Some(Self::Node);
        None
    }

    /// Create the concrete provider for this runtime.
    pub fn create_provider(self, code: &str) -> Result<JsRuntimeProvider, JsChallengeError> {
        match self {
            #[cfg(feature = "quickjs-oxide")]
            Self::QuickJsOxide => Ok(JsRuntimeProvider::QuickJsOxide(
                crate::builtin::quickjs_oxide::QuickJsOxideJCP::new(code)?,
            )),
            #[cfg(feature = "qjs")]
            Self::QuickJS => Ok(JsRuntimeProvider::QuickJS(
                crate::builtin::quickjs::QuickJSJCP::new(code)?,
            )),
            #[cfg(feature = "boa")]
            Self::Boa => Ok(JsRuntimeProvider::Boa(Box::new(
                crate::builtin::boa::BoaJCP::new(code)?,
            ))),
            #[cfg(feature = "external")]
            Self::Node => Ok(JsRuntimeProvider::Node(crate::builtin::node::NodeJCP::new(
                code,
            ))),
            #[cfg(feature = "external")]
            Self::Deno => Ok(JsRuntimeProvider::Deno(crate::builtin::deno::DenoJCP::new(
                code,
            ))),
            #[cfg(feature = "external")]
            Self::Bun => Ok(JsRuntimeProvider::Bun(crate::builtin::bun::BunJCP::new(
                code,
            ))),
        }
    }
}
