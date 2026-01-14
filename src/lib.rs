//! EJS - JavaScript Challenge Solver Library

pub mod builtin;
pub mod director;
pub mod provider;
pub mod registry;
pub mod test_data;

// Re-export public API
pub use builtin::preprocessor::preprocess_player;
pub use director::{process_input };
pub use provider::{
    JsChallengeError, JsChallengeInput, JsChallengeOutput, JsChallengeRequest, JsChallengeResponse,
    JsChallengeType,
};
pub use registry::RuntimeType;

/// Run challenge solver with the specified runtime
pub fn run(
    player: String,
    runtime: RuntimeType,
    challenges: Vec<String>,
) -> Result<JsChallengeOutput, JsChallengeError> {
    let mut n_challenges = Vec::new();
    let mut sig_challenges = Vec::new();

    for request in &challenges {
        let parts: Vec<&str> = request.splitn(2, ':').collect();
        if parts.len() != 2 {
            return Err(JsChallengeError::Parse(format!(
                "Invalid request format: {}",
                request
            )));
        }

        match parts[0] {
            "n" => n_challenges.push(parts[1].to_string()),
            "sig" => sig_challenges.push(parts[1].to_string()),
            t => return Err(JsChallengeError::Parse(format!("Unsupported type: {}", t))),
        }
    }

    let input = JsChallengeInput::Player {
        player,
        requests: vec![
            JsChallengeRequest {
                challenge_type: JsChallengeType::N,
                challenges: n_challenges,
            },
            JsChallengeRequest {
                challenge_type: JsChallengeType::Sig,
                challenges: sig_challenges,
            },
        ],
        output_preprocessed: false,
    };

    Ok(process_input(input, runtime))
}
