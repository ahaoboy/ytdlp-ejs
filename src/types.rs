use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Input {
    Player {
        player: String,
        requests: Vec<Request>,
        output_preprocessed: bool,
    },
    Preprocessed {
        preprocessed_player: String,
        requests: Vec<Request>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Request {
    #[serde(rename = "type")]
    pub req_type: RequestType,
    pub challenges: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RequestType {
    N,
    Sig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Response {
    Result { data: HashMap<String, String> },
    Error { error: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Output {
    Result {
        #[serde(skip_serializing_if = "Option::is_none")]
        preprocessed_player: Option<String>,
        responses: Vec<Response>,
    },
    Error {
        error: String,
    },
}
