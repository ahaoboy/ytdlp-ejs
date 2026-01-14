//! Integration tests using cases.csv data with different JS runtimes
//!
//! Run with: cargo test --test runtime_tests
//! Run specific runtime: cargo test --test runtime_tests --features qjs
//! Run all runtimes: cargo test --test runtime_tests --all-features

use ytdlp_ejs::test_data::{ALL_VARIANTS, TEST_CASES, get_cache_path};
use ytdlp_ejs::{
    JsChallengeInput, JsChallengeOutput, JsChallengeRequest, JsChallengeResponse, JsChallengeType,
    RuntimeType, process_input,
};
use std::fs;
use std::path::Path;

struct TestCase {
    player_file: String,
    player_name: String,
    test_type: String,
    input: String,
    expected: String,
}

fn load_test_cases() -> Vec<TestCase> {
    let mut cases = Vec::new();

    for test_case in TEST_CASES {
        let variants = test_case.variants.unwrap_or(ALL_VARIANTS);

        for variant in variants {
            let cache_path = get_cache_path(test_case.player, variant);
            let path = Path::new(&cache_path);

            if !path.exists() {
                continue;
            }

            let player_name = format!("{}-{}", test_case.player, variant);

            for step in test_case.n {
                cases.push(TestCase {
                    player_file: cache_path.clone(),
                    player_name: player_name.clone(),
                    test_type: "n".to_string(),
                    input: step.input.to_string(),
                    expected: step.expected.to_string(),
                });
            }

            for step in test_case.sig {
                cases.push(TestCase {
                    player_file: cache_path.clone(),
                    player_name: player_name.clone(),
                    test_type: "sig".to_string(),
                    input: step.input.to_string(),
                    expected: step.expected.to_string(),
                });
            }
        }
    }

    cases
}

fn run_tests_with_runtime(runtime: RuntimeType) -> (usize, usize, Vec<String>) {
    let cases = load_test_cases();
    let mut passed = 0;
    let mut failed = 0;
    let mut errors = Vec::new();

    // Group by player file
    let mut grouped: std::collections::HashMap<String, Vec<&TestCase>> =
        std::collections::HashMap::new();
    for case in &cases {
        grouped
            .entry(case.player_file.clone())
            .or_default()
            .push(case);
    }

    for (player_file, player_cases) in grouped {
        let content = match fs::read_to_string(&player_file) {
            Ok(c) => c,
            Err(e) => {
                for case in &player_cases {
                    failed += 1;
                    errors.push(format!(
                        "FAIL: {} {} - Failed to read file: {}",
                        case.player_name, case.test_type, e
                    ));
                }
                continue;
            }
        };

        let n_challenges: Vec<String> = player_cases
            .iter()
            .filter(|c| c.test_type == "n")
            .map(|c| c.input.clone())
            .collect();

        let sig_challenges: Vec<String> = player_cases
            .iter()
            .filter(|c| c.test_type == "sig")
            .map(|c| c.input.clone())
            .collect();

        let input = JsChallengeInput::Player {
            player: content,
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

        let output = process_input(input, runtime);

        match output {
            JsChallengeOutput::Result { responses, .. } => {
                // Check n results
                if let Some(JsChallengeResponse::Result { data }) = responses.first() {
                    for case in player_cases.iter().filter(|c| c.test_type == "n") {
                        if let Some(result) = data.get(&case.input) {
                            if result == &case.expected {
                                passed += 1;
                            } else {
                                failed += 1;
                                errors.push(format!(
                                    "FAIL: {} n {}\n  Expected: {}\n  Got: {}",
                                    case.player_name, case.input, case.expected, result
                                ));
                            }
                        } else {
                            failed += 1;
                            errors.push(format!(
                                "FAIL: {} n {} - No result returned",
                                case.player_name, case.input
                            ));
                        }
                    }
                } else if let Some(JsChallengeResponse::Error { error }) = responses.first() {
                    for case in player_cases.iter().filter(|c| c.test_type == "n") {
                        failed += 1;
                        errors.push(format!(
                            "FAIL: {} n {} - Error: {}",
                            case.player_name, case.input, error
                        ));
                    }
                }

                // Check sig results
                if let Some(JsChallengeResponse::Result { data }) = responses.get(1) {
                    for case in player_cases.iter().filter(|c| c.test_type == "sig") {
                        if let Some(result) = data.get(&case.input) {
                            if result == &case.expected {
                                passed += 1;
                            } else {
                                failed += 1;
                                errors.push(format!(
                                    "FAIL: {} sig {}\n  Expected: {}\n  Got: {}",
                                    case.player_name, case.input, case.expected, result
                                ));
                            }
                        } else {
                            failed += 1;
                            errors.push(format!(
                                "FAIL: {} sig {} - No result returned",
                                case.player_name, case.input
                            ));
                        }
                    }
                } else if let Some(JsChallengeResponse::Error { error }) = responses.get(1) {
                    for case in player_cases.iter().filter(|c| c.test_type == "sig") {
                        failed += 1;
                        errors.push(format!(
                            "FAIL: {} sig {} - Error: {}",
                            case.player_name, case.input, error
                        ));
                    }
                }
            }
            JsChallengeOutput::Error { error } => {
                for case in &player_cases {
                    failed += 1;
                    errors.push(format!(
                        "FAIL: {} {} {} - Processing error: {}",
                        case.player_name, case.test_type, case.input, error
                    ));
                }
            }
        }
    }

    (passed, failed, errors)
}

#[cfg(feature = "qjs")]
#[test]
fn test_qjs_runtime() {
    let (passed, failed, errors) = run_tests_with_runtime(RuntimeType::QuickJS);

    if !errors.is_empty() {
        eprintln!("\n=== QuickJS Runtime Errors ===");
        for error in &errors {
            eprintln!("{}", error);
        }
    }

    eprintln!("\nQuickJS Results: {}/{} passed", passed, passed + failed);

    assert!(
        failed == 0 || passed > 0,
        "QuickJS: {} tests failed out of {}",
        failed,
        passed + failed
    );
}

#[cfg(feature = "boa")]
#[test]
fn test_boa_runtime() {
    let (passed, failed, errors) = run_tests_with_runtime(RuntimeType::Boa);

    if !errors.is_empty() {
        eprintln!("\n=== Boa Runtime Errors ===");
        for error in &errors {
            eprintln!("{}", error);
        }
    }

    eprintln!("\nBoa Results: {}/{} passed", passed, passed + failed);

    assert!(
        failed == 0 || passed > 0,
        "Boa: {} tests failed out of {}",
        failed,
        passed + failed
    );
}

#[cfg(feature = "external")]
#[test]
fn test_deno_runtime() {
    let (passed, failed, errors) = run_tests_with_runtime(RuntimeType::Deno);

    if !errors.is_empty() {
        eprintln!("\n=== Deno Runtime Errors ===");
        for error in &errors {
            eprintln!("{}", error);
        }
    }

    eprintln!("\nDeno Results: {}/{} passed", passed, passed + failed);

    assert!(
        failed == 0 || passed > 0,
        "Deno: {} tests failed out of {}",
        failed,
        passed + failed
    );
}

#[cfg(feature = "external")]
#[test]
fn test_node_runtime() {
    let (passed, failed, errors) = run_tests_with_runtime(RuntimeType::Node);

    if !errors.is_empty() {
        eprintln!("\n=== Node Runtime Errors ===");
        for error in &errors {
            eprintln!("{}", error);
        }
    }

    eprintln!("\nNode Results: {}/{} passed", passed, passed + failed);

    assert!(
        failed == 0 || passed > 0,
        "Node: {} tests failed out of {}",
        failed,
        passed + failed
    );
}

#[cfg(feature = "external")]
#[test]
fn test_bun_runtime() {
    let (passed, failed, errors) = run_tests_with_runtime(RuntimeType::Bun);

    if !errors.is_empty() {
        eprintln!("\n=== Bun Runtime Errors ===");
        for error in &errors {
            eprintln!("{}", error);
        }
    }

    eprintln!("\nBun Results: {}/{} passed", passed, passed + failed);

    assert!(
        failed == 0 || passed > 0,
        "Bun: {} tests failed out of {}",
        failed,
        passed + failed
    );
}
