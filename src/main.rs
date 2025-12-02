use std::env;
use std::fs;
use std::process;
use std::thread;

use ejs::types::{Input, Request, RequestType};
use ejs::{process_input_with_runtime, RuntimeType};

// Stack size for parsing large JavaScript files (16MB)
const STACK_SIZE: usize = 16 * 1024 * 1024;

#[cfg(all(
    not(target_os = "windows"),
    not(target_os = "android"),
    not(target_env = "musl")
))]
#[global_allocator]
static ALLOC: snmalloc_rs::SnMalloc = snmalloc_rs::SnMalloc;

fn print_usage(program: &str) {
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
    eprintln!();
    eprintln!("Examples:");
    eprintln!("  {} player.js n:ZdZIqFPQK-Ty8wId", program);
    eprintln!(
        "  {} --runtime deno player.js sig:gN7a-hudCuAuPH6f...",
        program
    );
}

fn run() -> i32 {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        print_usage(&args[0]);
        return 1;
    }

    let mut runtime_type = RuntimeType::default();
    let mut player_path: Option<String> = None;
    let mut requests_args: Vec<String> = Vec::new();

    let mut i = 1;
    while i < args.len() {
        let arg = &args[i];
        if arg == "--runtime" {
            i += 1;
            if i >= args.len() {
                eprintln!("ERROR: --runtime requires an argument");
                return 1;
            }
            runtime_type = match RuntimeType::parse(&args[i]) {
                Some(rt) => rt,
                None => {
                    eprintln!(
                        "ERROR: Unknown runtime '{}'. Available: {}",
                        args[i],
                        RuntimeType::available_runtimes().join(", ")
                    );
                    return 1;
                }
            };
        } else if arg == "--help" || arg == "-h" {
            print_usage(&args[0]);
            return 0;
        } else if player_path.is_none() {
            player_path = Some(arg.clone());
        } else {
            requests_args.push(arg.clone());
        }
        i += 1;
    }

    let player_path = match player_path {
        Some(p) => p,
        None => {
            eprintln!("ERROR: Missing player file argument");
            print_usage(&args[0]);
            return 1;
        }
    };

    if requests_args.is_empty() {
        eprintln!("ERROR: At least one request is required");
        print_usage(&args[0]);
        return 1;
    }

    let player = match fs::read_to_string(&player_path) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("ERROR: Failed to read player file: {}", e);
            return 1;
        }
    };

    let mut n_challenges = Vec::new();
    let mut sig_challenges = Vec::new();

    for request in &requests_args {
        let parts: Vec<&str> = request.splitn(2, ':').collect();
        if parts.len() != 2 {
            eprintln!("ERROR: Invalid request format: {}", request);
            return 1;
        }

        let req_type = parts[0];
        let challenge = parts[1].to_string();

        match req_type {
            "n" => n_challenges.push(challenge),
            "sig" => sig_challenges.push(challenge),
            _ => {
                eprintln!("ERROR: Unsupported request type: {}", req_type);
                return 1;
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

    let output = process_input_with_runtime(input, runtime_type);

    match serde_json::to_string(&output) {
        Ok(json) => println!("{}", json),
        Err(e) => {
            eprintln!("ERROR: Failed to serialize output: {}", e);
            return 1;
        }
    }

    0
}

fn main() {
    // Spawn a thread with larger stack size to handle large JS files
    let child = thread::Builder::new()
        .stack_size(STACK_SIZE)
        .spawn(run)
        .expect("Failed to spawn thread");

    let exit_code = child.join().expect("Thread panicked");
    process::exit(exit_code);
}
