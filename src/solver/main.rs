use super::runtime::{create_solver, JsRuntime, RuntimeType};
use super::solvers::preprocess_player;
use crate::types::{Input, Output, Request, RequestType, Response};

/// Process input with specified runtime and return output
pub fn process_input_with_runtime(input: Input, runtime_type: RuntimeType) -> Output {
    let (preprocessed, should_output_preprocessed) = match &input {
        Input::Player {
            player,
            output_preprocessed,
            ..
        } => match preprocess_player(player) {
            Ok(code) => (code, *output_preprocessed),
            Err(e) => {
                return Output::Error {
                    error: format!("Failed to preprocess player: {}", e),
                };
            }
        },
        Input::Preprocessed {
            preprocessed_player,
            ..
        } => (preprocessed_player.clone(), false),
    };

    let requests = match &input {
        Input::Player { requests, .. } => requests,
        Input::Preprocessed { requests, .. } => requests,
    };

    let solver = match create_solver(runtime_type, &preprocessed) {
        Ok(s) => s,
        Err(e) => {
            return Output::Error {
                error: format!("Failed to create solvers: {}", e),
            };
        }
    };

    let responses: Vec<Response> = requests
        .iter()
        .map(|req| process_request(solver.as_ref(), req))
        .collect();

    Output::Result {
        preprocessed_player: if should_output_preprocessed {
            Some(preprocessed)
        } else {
            None
        },
        responses,
    }
}

/// Process input with default runtime and return output (main entry point)
pub fn process_input(input: Input) -> Output {
    process_input_with_runtime(input, RuntimeType::default())
}

fn process_request(solver: &dyn JsRuntime, request: &Request) -> Response {
    let req_type_str = match request.req_type {
        RequestType::N => "n",
        RequestType::Sig => "sig",
    };

    // Check if solver is available
    let has_solver = match request.req_type {
        RequestType::N => solver.has_n(),
        RequestType::Sig => solver.has_sig(),
    };

    if !has_solver {
        return Response::Error {
            error: format!("Failed to extract {} function", req_type_str),
        };
    }

    match solver.solve_challenges(req_type_str, &request.challenges) {
        Ok(data) => Response::Result { data },
        Err(e) => Response::Error { error: e },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::Path;

    #[derive(Debug)]
    struct TestCase {
        player: String,
        n_tests: Vec<(String, String)>,
        sig_tests: Vec<(String, String)>,
    }

    fn get_test_cases() -> Vec<TestCase> {
        vec![
            TestCase {
                player: "3d3ba064".to_string(),
                n_tests: vec![
                    ("ZdZIqFPQK-Ty8wId".to_string(), "qmtUsIz04xxiNW".to_string()),
                    ("4GMrWHyKI5cEvhDO".to_string(), "N9gmEX7YhKTSmw".to_string()),
                ],
                sig_tests: vec![(
                    "gN7a-hudCuAuPH6fByOk1_GNXN0yNMHShjZXS2VOgsEItAJz0tipeavEOmNdYN-wUtcEqD3bCXjc0iyKfAyZxCBGgIARwsSdQfJ2CJtt".to_string(),
                    "ttJC2JfQdSswRAIgGBCxZyAfKyi0cjXCb3gqEctUw-NYdNmOEvaepit0zJAtIEsgOV2SXZjhSHMNy0NXNG_1kNyBf6HPuAuCduh-a7O".to_string(),
                )],
            },
            TestCase {
                player: "5ec65609".to_string(),
                n_tests: vec![(
                    "0eRGgQWJGfT5rFHFj".to_string(),
                    "4SvMpDQH-vBJCw".to_string(),
                )],
                sig_tests: vec![(
                    "AAJAJfQdSswRQIhAMG5SN7-cAFChdrE7tLA6grH0rTMICA1mmDc0HoXgW3CAiAQQ4=CspfaF_vt82XH5yewvqcuEkvzeTsbRuHssRMyJQ=I".to_string(),
                    "AJfQdSswRQIhAMG5SN7-cAFChdrE7tLA6grI0rTMICA1mmDc0HoXgW3CAiAQQ4HCspfaF_vt82XH5yewvqcuEkvzeTsbRuHssRMyJQ==".to_string(),
                )],
            },
            TestCase {
                player: "6742b2b9".to_string(),
                n_tests: vec![
                    ("_HPB-7GFg1VTkn9u".to_string(), "qUAsPryAO_ByYg".to_string()),
                    ("K1t_fcB6phzuq2SF".to_string(), "Y7PcOt3VE62mog".to_string()),
                ],
                sig_tests: vec![(
                    "MMGZJMUucirzS_SnrSPYsc85CJNnTUi6GgR5NKn-znQEICACojE8MHS6S7uYq4TGjQX_D4aPk99hNU6wbTvorvVVMgIARwsSdQfJAA".to_string(),
                    "AJfQdSswRAIgMVVvrovTbw6UNh99kPa4D_XQjGT4qYu7S6SHM8EjoCACIEQnz-nKN5RgG6iUTnNJC58csYPSrnS_SzricuUMJZGM".to_string(),
                )],
            },
        ]
    }

    fn get_player_variants() -> Vec<&'static str> {
        vec![
            "main", "tcc", "tce", "es5", "es6", "tv", "tv_es6", "phone", "tablet",
        ]
    }

    fn get_player_path(player: &str, variant: &str) -> String {
        format!("ts/src/yt/solver/test/players/{}-{}", player, variant)
    }

    #[test]
    fn test_solvers() {
        for test_case in get_test_cases() {
            for variant in get_player_variants() {
                let path = get_player_path(&test_case.player, variant);
                let path = Path::new(&path);

                if !path.exists() {
                    continue;
                }

                let content = fs::read_to_string(path).expect("Failed to read player file");

                let input = Input::Player {
                    player: content,
                    requests: vec![
                        Request {
                            req_type: RequestType::N,
                            challenges: test_case.n_tests.iter().map(|(i, _)| i.clone()).collect(),
                        },
                        Request {
                            req_type: RequestType::Sig,
                            challenges: test_case
                                .sig_tests
                                .iter()
                                .map(|(i, _)| i.clone())
                                .collect(),
                        },
                    ],
                    output_preprocessed: false,
                };

                let output = process_input(input);

                match output {
                    Output::Result { responses, .. } => {
                        // Check n results
                        if let Response::Result { data } = &responses[0] {
                            for (input, expected) in &test_case.n_tests {
                                assert_eq!(
                                    data.get(input),
                                    Some(expected),
                                    "n test failed for {} {}: input={}",
                                    test_case.player,
                                    variant,
                                    input
                                );
                            }
                        } else {
                            panic!(
                                "n response was an error for {} {}",
                                test_case.player, variant
                            );
                        }

                        // Check sig results
                        if let Response::Result { data } = &responses[1] {
                            for (input, expected) in &test_case.sig_tests {
                                assert_eq!(
                                    data.get(input),
                                    Some(expected),
                                    "sig test failed for {} {}: input={}",
                                    test_case.player,
                                    variant,
                                    input
                                );
                            }
                        } else {
                            panic!(
                                "sig response was an error for {} {}",
                                test_case.player, variant
                            );
                        }
                    }
                    Output::Error { error } => {
                        panic!(
                            "Processing failed for {} {}: {}",
                            test_case.player, variant, error
                        );
                    }
                }
            }
        }
    }
}
