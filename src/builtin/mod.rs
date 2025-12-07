//! Built-in JS Challenge Providers
//!
//! This module contains the built-in implementations using enum dispatch
//! for better performance and simpler code.

pub mod polyfill;
pub mod preprocessor;

#[cfg(feature = "qjs")]
pub mod quickjs;

#[cfg(feature = "boa")]
pub mod boa;

#[cfg(feature = "external")]
pub mod deno;

#[cfg(feature = "external")]
pub mod node;

#[cfg(feature = "external")]
pub mod bun;

use crate::{JsChallengeType, provider::JsChallengeError};
use std::collections::HashMap;

/// Enum-based provider for static dispatch (no trait objects)
pub enum JsRuntimeProvider {
    #[cfg(feature = "qjs")]
    QuickJS(quickjs::QuickJSJCP),
    #[cfg(feature = "boa")]
    Boa(Box<boa::BoaJCP>),
    #[cfg(feature = "external")]
    Deno(deno::DenoJCP),
    #[cfg(feature = "external")]
    Node(node::NodeJCP),
    #[cfg(feature = "external")]
    Bun(bun::BunJCP),
}

impl JsRuntimeProvider {
    pub fn solve_n(&mut self, challenge: &str) -> Result<String, JsChallengeError> {
        match self {
            #[cfg(feature = "qjs")]
            Self::QuickJS(p) => p.solve_n(challenge),
            #[cfg(feature = "boa")]
            Self::Boa(p) => p.solve_n(challenge),
            #[cfg(feature = "external")]
            Self::Deno(p) => p.solve("n", challenge),
            #[cfg(feature = "external")]
            Self::Node(p) => p.solve("n", challenge),
            #[cfg(feature = "external")]
            Self::Bun(p) => p.solve("n", challenge),
        }
    }

    pub fn solve_sig(&mut self, challenge: &str) -> Result<String, JsChallengeError> {
        match self {
            #[cfg(feature = "qjs")]
            Self::QuickJS(p) => p.solve_sig(challenge),
            #[cfg(feature = "boa")]
            Self::Boa(p) => p.solve_sig(challenge),
            #[cfg(feature = "external")]
            Self::Deno(p) => p.solve("sig", challenge),
            #[cfg(feature = "external")]
            Self::Node(p) => p.solve("sig", challenge),
            #[cfg(feature = "external")]
            Self::Bun(p) => p.solve("sig", challenge),
        }
    }

    pub fn solve_challenges(
        &mut self,
        req_type: &JsChallengeType,
        challenges: &[String],
    ) -> Result<HashMap<String, String>, JsChallengeError> {
        let mut results = HashMap::with_capacity(challenges.len());
        for challenge in challenges {
            let result = match req_type {
                JsChallengeType::N => self.solve_n(challenge)?,
                JsChallengeType::Sig => self.solve_sig(challenge)?,
            };
            results.insert(challenge.clone(), result);
        }
        Ok(results)
    }
}
