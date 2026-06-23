//! JS Challenge Request Director

use crate::builtin::JsRuntimeProvider;
use crate::builtin::preprocessor::preprocess_player;
use crate::provider::{
    JsChallengeError, JsChallengeInput, JsChallengeOutput, JsChallengeRequest, JsChallengeResponse,
};
use crate::registry::RuntimeType;
use crate::trace::{debug, error, info, trace_span};

/// Process input with specified runtime and return output
pub fn process_input(input: JsChallengeInput, runtime_type: RuntimeType) -> JsChallengeOutput {
    match process_internal(input, runtime_type) {
        Ok(output) => output,
        Err(e) => {
            error!(%e, "Processing failed");
            JsChallengeOutput::Error {
                error: e.to_string(),
            }
        }
    }
}

fn process_internal(
    input: JsChallengeInput,
    runtime_type: RuntimeType,
) -> Result<JsChallengeOutput, JsChallengeError> {
    trace_span!("process_internal", ?runtime_type);

    let (preprocessed, should_output, requests) = match &input {
        JsChallengeInput::Player {
            player,
            output_preprocessed,
            requests,
        } => {
            info!(player_len = player.len(), "Preprocessing player code");
            let code = preprocess_player(player)?;
            debug!(preprocessed_len = code.len(), "Preprocessing complete");
            (code, *output_preprocessed, requests)
        }
        JsChallengeInput::Preprocessed {
            preprocessed_player,
            requests,
        } => {
            debug!(
                preprocessed_len = preprocessed_player.len(),
                "Using preprocessed player code"
            );
            (preprocessed_player.clone(), false, requests)
        }
    };

    debug!(?runtime_type, ?requests, "Creating JS runtime provider");
    let mut provider = runtime_type.create_provider(&preprocessed)?;
    info!(?runtime_type, "Runtime provider ready");

    let responses: Vec<JsChallengeResponse> = requests
        .iter()
        .map(|req| process_request(&mut provider, req))
        .collect();

    Ok(JsChallengeOutput::Result {
        preprocessed_player: if should_output {
            Some(preprocessed)
        } else {
            None
        },
        responses,
    })
}

fn process_request(
    provider: &mut JsRuntimeProvider,
    request: &JsChallengeRequest,
) -> JsChallengeResponse {
    trace_span!(
        "process_request",
        req_type = %request.challenge_type.as_str(),
        count = request.challenges.len()
    );

    debug!(?request.challenges, "Solving challenges");
    match provider.solve_challenges(&request.challenge_type, &request.challenges) {
        Ok(data) => {
            info!(results = data.len(), ?data, "Challenges solved");
            JsChallengeResponse::Result { data }
        }
        Err(e) => {
            error!(%e, "Challenge solving failed");
            JsChallengeResponse::Error {
                error: e.to_string(),
            }
        }
    }
}
