//! JS Challenge Request Director

use crate::builtin::JsRuntimeProvider;
use crate::builtin::preprocessor::preprocess_player;
use crate::provider::{
    JsChallengeError, JsChallengeInput, JsChallengeOutput, JsChallengeRequest, JsChallengeResponse,
};
use crate::registry::RuntimeType;

/// Process input with specified runtime and return output
pub fn process_input(input: JsChallengeInput, runtime_type: RuntimeType) -> JsChallengeOutput {
    match process_internal(input, runtime_type) {
        Ok(output) => output,
        Err(e) => JsChallengeOutput::Error {
            error: e.to_string(),
        },
    }
}

fn process_internal(
    input: JsChallengeInput,
    runtime_type: RuntimeType,
) -> Result<JsChallengeOutput, JsChallengeError> {
    let (preprocessed, should_output, requests) = match &input {
        JsChallengeInput::Player {
            player,
            output_preprocessed,
            requests,
        } => {
            let code = preprocess_player(player)?;
            (code, *output_preprocessed, requests)
        }
        JsChallengeInput::Preprocessed {
            preprocessed_player,
            requests,
        } => (preprocessed_player.clone(), false, requests),
    };

    let mut provider = runtime_type.create_provider(&preprocessed)?;

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
    match provider.solve_challenges(&request.challenge_type, &request.challenges) {
        Ok(data) => JsChallengeResponse::Result { data },
        Err(e) => JsChallengeResponse::Error {
            error: e.to_string(),
        },
    }
}
