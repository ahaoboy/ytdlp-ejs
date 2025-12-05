pub mod solver;
pub mod test_data;
pub mod types;
pub mod utils;

pub use solver::main::{process_input, process_input_with_runtime};
pub use solver::solvers::preprocess_player;
pub use solver::{JsRuntime, RuntimeType};
pub use types::{Input, Output, Request, RequestType, Response};

pub fn run(
    player: String,
    runtime: RuntimeType,
    challenges: Vec<String>,
) -> Result<Output, String> {
    let mut n_challenges = Vec::new();
    let mut sig_challenges = Vec::new();

    for request in &challenges {
        let parts: Vec<&str> = request.splitn(2, ':').collect();
        if parts.len() != 2 {
            return Err(format!("Invalid request format: {}", request));
        }

        let req_type = parts[0];
        let challenge = parts[1].to_string();

        match req_type {
            "n" => n_challenges.push(challenge),
            "sig" => sig_challenges.push(challenge),
            _ => {
                return Err(format!("Unsupported request type: {}", req_type));
            }
        }
    }

    let input = Input::Player {
        player,
        requests: vec![
            Request {
                req_type: RequestType::N,
                challenges: n_challenges,
            },
            Request {
                req_type: RequestType::Sig,
                challenges: sig_challenges,
            },
        ],
        output_preprocessed: false,
    };

    let output = process_input_with_runtime(input, runtime);
    Ok(output)
}
