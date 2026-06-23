use std::env;
use std::fs;
use std::process;

use ytdlp_ejs::trace::{debug, info};
use ytdlp_ejs::{JsChallengeInput, RuntimeType, run};

#[cfg(feature = "snmalloc")]
#[global_allocator]
static ALLOC: snmalloc_rs::SnMalloc = snmalloc_rs::SnMalloc;

fn print_usage(program: &str) {
    // yt-dlp detects QuickJS-compatible runtimes by parsing `--help` output.
    // It runs `{exe} --help` and matches: r'^QuickJS(?:-ng)?\s+version\s+(\S+)'
    // We report as quickjs-ng v0.12.0 (the minimum recommended) so yt-dlp
    // does not emit a "versions older than ..." warning.
    eprintln!(
        "QuickJS-ng version 0.12.0  (ejs {})",
        env!("CARGO_PKG_VERSION")
    );
    eprintln!();
    eprintln!(
        "Usage: {} [OPTIONS] <player> [<type>:<request> ...]",
        program
    );
    eprintln!();
    eprintln!("Options:");
    eprintln!("  --runtime <runtime>  JavaScript runtime to use");
    eprintln!(
        "                       Available: {}",
        RuntimeType::available_runtimes().join(", ")
    );
    eprintln!("  --script <file>      Execute JS file via embedded QuickJS (qjs compat)");
    eprintln!("  --help, -h           Show this help message");
    eprintln!("  --version, -V        Print version");
    eprintln!();
    eprintln!("Examples:");
    eprintln!("  {} player.js n:ZdZIqFPQK-Ty8wId", program);
    eprintln!(
        "  {} --runtime deno player.js sig:gN7a-hudCuAuPH6f...",
        program
    );
    eprintln!("  {} --script solver_program.js", program);
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

    // Use a larger stack (8 MB) to handle deeply nested AST processing
    std::thread::Builder::new()
        .stack_size(32 * 1024 * 1024)
        .spawn(|| {
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
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        print_usage(&args[0]);
        process::exit(1);
    }

    // ── Script mode (bare JS interpreter, used by yt-dlp as qjs replacement) ─
    // yt-dlp invokes: ejs --script <temp_file.js>
    // The temp file contains lib.js + core.js + console.log(JSON.stringify(jsc(...)))
    // We execute it via embedded QuickJS and output the console.log result.
    //
    // IMPORTANT: yt-dlp checks `if proc.returncode or stderr` — any stderr
    // output causes the request to fail. We must NOT write to stderr in the
    // success path. All diagnostic logging goes through the tracing subscriber
    // (controlled by RUST_LOG), which writes to stderr only when enabled.
    #[cfg(feature = "qjs")]
    if args.len() >= 3 && args[1] == "--script" {
        let script_path = &args[2];
        let code = fs::read_to_string(script_path)?;
        info!(script_len = code.len(), path = %script_path, "Executing script file");

        // ── Optimized path: extract jsc() JSON input, solve with Rust ─────
        // yt-dlp's script is: lib.js + core.js + console.log(JSON.stringify(jsc({...})))
        // Instead of running the entire JS program (which uses meriyah for
        // preprocessing), we extract the JSON argument from the jsc() call
        // and feed it directly into our native Rust preprocessing pipeline.
        match extract_jsc_input(&code) {
            Some(input) => {
                info!("Extracted jsc input from script — using native Rust solver");
                let output = ytdlp_ejs::process_input(input, ytdlp_ejs::RuntimeType::QuickJS);
                let json = serde_json::to_string(&output)?;
                println!("{}", json);
                return Ok(());
            }
            None => {
                // Fallback: run the full script via embedded QuickJS
                info!("Could not extract jsc input, falling back to QuickJS interpreter");
                match ytdlp_ejs::run_script(&code) {
                    Ok(output) => {
                        print!("{}", output);
                        return Ok(());
                    }
                    Err(e) => {
                        eprintln!("{}", e);
                        process::exit(1);
                    }
                }
            }
        }
    }
    #[cfg(not(feature = "qjs"))]
    if args.len() >= 3 && args[1] == "--script" {
        return Err("--script mode requires the 'qjs' feature".into());
    }

    let mut player_path: Option<String> = None;
    let mut requests_args = vec![];
    let mut runtime_type = RuntimeType::QuickJS;

    let mut i = 1;
    while i < args.len() {
        let arg = &args[i];
        match arg.as_str() {
            "--runtime" => {
                i += 1;
                if i >= args.len() {
                    return Err("--runtime requires an argument".into());
                }
                runtime_type = RuntimeType::parse(&args[i]).ok_or_else(|| {
                    format!(
                        "Unknown runtime '{}'. Available: {}",
                        args[i],
                        RuntimeType::available_runtimes().join(", ")
                    )
                })?;
                debug!(?runtime_type, "Runtime selected");
            }
            "--help" | "-h" => {
                print_usage(&args[0]);
                return Ok(());
            }
            "--version" | "-V" => {
                println!("ejs {}", env!("CARGO_PKG_VERSION"));
                return Ok(());
            }
            _ if player_path.is_none() => player_path = Some(arg.clone()),
            _ => requests_args.push(arg.clone()),
        }
        i += 1;
    }

    let player_path = player_path.ok_or("Missing player file argument")?;
    debug!(path = %player_path, "Loading player file");
    let player = fs::read_to_string(&player_path)?;
    info!(size = player.len(), path = %player_path, "Loaded player file");

    if requests_args.is_empty() {
        return Err("At least one request is required".into());
    }

    debug!(?requests_args, ?runtime_type, "Processing requests");
    let output = run(player, runtime_type, requests_args)?;
    let json = serde_json::to_string(&output)?;
    println!("{}", json);

    info!("Done");
    Ok(())
}

// ── jsc() JSON extraction ───────────────────────────────────────────────────
//
// yt-dlp's script format is:
//   {lib.js} + Object.assign(globalThis, lib); + {core.js}
//   + console.log(JSON.stringify(jsc({...json input...})));
//
// This function extracts the JSON input from the jsc() call, avoiding the
// need to run the meriyah-based JS solver inside QuickJS.

/// Extracts the `JsChallengeInput` from a yt-dlp jsc() call script.
/// Returns `None` if the extraction fails (e.g. unexpected format).
///
/// The script ends with:
///   console.log(JSON.stringify(jsc({"type":"player","player":"...",...})));
///
/// We extract the JSON object between the last `jsc(` and its matching `)`.
/// Braces inside JSON string values (e.g. the player source code) are safely
/// skipped by tracking `"` delimiters and escape sequences.
fn extract_jsc_input(code: &str) -> Option<JsChallengeInput> {
    // Use the full marker for robustness — the player code could theoretically
    // contain the literal text "jsc(" which would confuse a naive rfind.
    let marker = "console.log(JSON.stringify(jsc(";
    let jsc_pos = code.rfind(marker)?;
    let start = jsc_pos + marker.len(); // position after "jsc("

    // Find the JSON object by tracking braces
    let bytes = code.as_bytes();
    let mut pos = start;
    let mut brace_depth: i32 = 0;
    let mut json_start = start;

    // Find the opening '{' of the JSON object
    while pos < bytes.len() {
        match bytes[pos] {
            b'{' => {
                brace_depth = 1;
                json_start = pos;
                pos += 1;
                break;
            }
            _ => {
                pos += 1;
            }
        }
    }

    // Track braces to find the matching closing '}'.
    // JSON string values (like the player source) are skipped so braces
    // inside them don't affect the count.
    while pos < bytes.len() && brace_depth > 0 {
        match bytes[pos] {
            b'{' => brace_depth += 1,
            b'}' => brace_depth -= 1,
            b'"' => {
                pos += 1;
                while pos < bytes.len() && bytes[pos] != b'"' {
                    if bytes[pos] == b'\\' {
                        pos += 1; // skip escaped char (handles \\, \", \n, etc.)
                    }
                    pos += 1;
                }
            }
            _ => {}
        }
        pos += 1;
    }

    if brace_depth != 0 {
        debug!(
            "jsc JSON extraction failed: unmatched braces (depth={})",
            brace_depth
        );
        return None;
    }

    let json_end = pos; // position after the closing '}'
    let json_str = &code[json_start..json_end];

    debug!(json_len = json_str.len(), "Extracted jsc JSON input");
    serde_json::from_str::<JsChallengeInput>(json_str).ok()
}
