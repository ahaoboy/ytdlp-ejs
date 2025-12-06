//! Public API for JS Challenge Provider
//!
//! This module provides the core types for JavaScript challenge solving.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

/// Error type for JS Challenge operations
#[derive(Debug, Error)]
pub enum JsChallengeError {
    #[error("Parse error: {0}")]
    Parse(String),
    #[error("Preprocess error: {0}")]
    Preprocess(String),
    #[error("Runtime error: {0}")]
    Runtime(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// Type of JavaScript challenge
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum JsChallengeType {
    N,
    Sig,
}

impl JsChallengeType {
    /// Convert to static string slice
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::N => "n",
            Self::Sig => "sig",
        }
    }
}

/// A request to solve a JavaScript challenge
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsChallengeRequest {
    #[serde(rename = "type")]
    pub challenge_type: JsChallengeType,
    pub challenges: Vec<String>,
}

/// Response from solving a JavaScript challenge
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum JsChallengeResponse {
    Result { data: HashMap<String, String> },
    Error { error: String },
}

/// Input format for the challenge solver
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum JsChallengeInput {
    Player {
        player: String,
        requests: Vec<JsChallengeRequest>,
        output_preprocessed: bool,
    },
    Preprocessed {
        preprocessed_player: String,
        requests: Vec<JsChallengeRequest>,
    },
}

/// Output format from the challenge solver
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum JsChallengeOutput {
    Result {
        #[serde(skip_serializing_if = "Option::is_none")]
        preprocessed_player: Option<String>,
        responses: Vec<JsChallengeResponse>,
    },
    Error {
        error: String,
    },
}
