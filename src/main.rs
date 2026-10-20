use std::fs;

use clap::Parser;
use ytdlp_ejs::trace::{debug, info};
use ytdlp_ejs::{JsChallengeInput, RuntimeType, process_input, run};

#[cfg(feature = "snmalloc")]
#[global_allocator]
static ALLOC: snmalloc_rs::SnMalloc = snmalloc_rs::SnMalloc;

/// Stack reserved for the evaluation thread. SWC preprocessing and the
/// embedded JS engines both recurse; quickjs-oxide also uses this size to
/// set its native stack budget.
const EVALUATION_STACK_SIZE: usize = 32 * 1024 * 1024;

/// yt-dlp detects QuickJS-compatible runtimes by parsing `--help` output and
/// matches `^QuickJS(?:-ng)?\s+version\s+(\S+)`. Report quickjs-ng v0.12.0
/// (the minimum recommended) so it does not emit an outdated-version warning.
const QUICKJS_HELP: &str = concat!(
    "QuickJS-ng version 0.12.0  (ejs ",
    env!("CARGO_PKG_VERSION"),
    ")"
);

/// YouTube player signature solver.
#[derive(Parser)]
#[command(
    name = "ejs",
    version,
    about = "YouTube player signature solver",
    before_help = QUICKJS_HELP,
    arg_required_else_help = true,
    after_help = "Examples:\n  \
                  ejs player.js n:ZdZIqFPQK-Ty8wId\n  \
                  ejs --runtime quickjs-oxide player.js sig:...\n  \
                  ejs --script solver_program.js"
)]
struct Cli {
    /// JavaScript runtime to use.
    #[arg(
        long,
        value_name = "RUNTIME",
        value_parser = parse_runtime,
        help = runtime_help()
    )]
    runtime: Option<RuntimeType>,

    /// yt-dlp compatibility shim: solve a generated jsc() script.
    #[arg(long, value_name = "FILE")]
    script: Option<String>,

    /// Player JavaScript file.
    #[arg(
        value_name = "PLAYER",
        required_unless_present = "script",
        conflicts_with = "script"
    )]
    player: Option<String>,

    /// Challenge requests, e.g. n:... or sig:...
    #[arg(
        value_name = "REQUEST",
        num_args = 1..,
        required_unless_present = "script",
        conflicts_with = "script"
    )]
    requests: Vec<String>,
}

fn parse_runtime(value: &str) -> Result<RuntimeType, String> {
    RuntimeType::parse(value).ok_or_else(|| {
        format!(
            "unknown runtime '{value}'; available: {}",
            RuntimeType::available_runtimes().join(", ")
        )
    })
}

fn runtime_help() -> String {
    format!(
        "JavaScript runtime to use (available: {})",
        RuntimeType::available_runtimes().join(", ")
    )
}

fn main() {
    // Initialize tracing subscriber when the "tracing" feature is enabled.
    // Controlled by RUST_LOG env var. Default: show warnings and errors only.
    // Enable: RUST_LOG=info or RUST_LOG=ytdlp_ejs=debug
    #[cfg(feature = "tracing")]
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("warn")),
        )
        .with_target(false)
        .init();

    // Use a larger stack to handle deeply nested AST processing and JS
    // evaluation.
    std::thread::Builder::new()
        .stack_size(EVALUATION_STACK_SIZE)
        .spawn(|| {
            #[cfg(feature = "quickjs-oxide")]
            quickjs_oxide::engine::api::core::declare_thread_stack(EVALUATION_STACK_SIZE);

            if let Err(e) = run_main() {
                eprintln!("ERROR: {}", e);
                std::process::exit(1);
            }
        })
        .unwrap()
        .join()
        .unwrap();
}

fn run_main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    let runtime_type = match cli.runtime {
        Some(runtime) => runtime,
        None => RuntimeType::default_runtime().ok_or(
            "No JavaScript runtime feature enabled (enable qjs, quickjs-oxide, boa, or external)",
        )?,
    };

    if let Some(script_path) = cli.script {
        let code = fs::read_to_string(&script_path)?;
        let input = extract_jsc_input(&code)
            .ok_or("unsupported --script input: expected console.log(JSON.stringify(jsc(...)))")?;
        let output = process_input(input, runtime_type);
        println!("{}", serde_json::to_string(&output)?);
        return Ok(());
    }

    let player_path = cli.player.ok_or("Missing player file argument")?;
    debug!(path = %player_path, "Loading player file");
    let player = fs::read_to_string(&player_path)?;
    info!(size = player.len(), path = %player_path, "Loaded player file");

    debug!(requests = ?cli.requests, ?runtime_type, "Processing requests");
    let output = run(player, runtime_type, cli.requests)?;
    println!("{}", serde_json::to_string(&output)?);

    info!("Done");
    Ok(())
}

/// Extracts the `jsc({...})` JSON argument from a yt-dlp generated script.
fn extract_jsc_input(code: &str) -> Option<JsChallengeInput> {
    const MARKER: &str = "console.log(JSON.stringify(jsc(";
    let start = code.rfind(MARKER)? + MARKER.len();
    let bytes = code.as_bytes();
    let json_start = start + bytes[start..].iter().position(|&byte| byte == b'{')?;

    let mut depth = 0i32;
    let mut in_string = false;
    let mut escaped = false;
    for (offset, &byte) in bytes[json_start..].iter().enumerate() {
        if in_string {
            if escaped {
                escaped = false;
            } else if byte == b'\\' {
                escaped = true;
            } else if byte == b'"' {
                in_string = false;
            }
            continue;
        }
        match byte {
            b'"' => in_string = true,
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    let json = &code[json_start..json_start + offset + 1];
                    return serde_json::from_str(json).ok();
                }
            }
            _ => {}
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_jsc_input() {
        let script = r#"console.log(JSON.stringify(jsc({"type":"preprocessed","preprocessed_player":"_result.n = () => 1;","requests":[]})));"#;
        assert!(matches!(
            extract_jsc_input(script),
            Some(JsChallengeInput::Preprocessed { .. })
        ));
    }

    #[test]
    fn ignores_braces_in_strings() {
        let script = r#"console.log(JSON.stringify(jsc({"type":"preprocessed","preprocessed_player":"const x = {a: 1};","requests":[{"type":"n","challenges":["x"]}]})));"#;
        assert!(extract_jsc_input(script).is_some());
    }
}
