use std::env;
use std::fs;
use std::process;

use ejs::{RuntimeType, run};

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

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        print_usage(&args[0]);
        process::exit(1);
    }

    let mut player_path: Option<String> = None;
    let mut requests_args = vec![];
    let mut runtime_type = RuntimeType::default();

    let mut i = 1;
    while i < args.len() {
        let arg = &args[i];
        if arg == "--runtime" {
            i += 1;
            if i >= args.len() {
                eprintln!("ERROR: --runtime requires an argument");
                process::exit(1);
            }
            runtime_type = match RuntimeType::parse(&args[i]) {
                Some(rt) => rt,
                None => {
                    eprintln!(
                        "ERROR: Unknown runtime '{}'. Available: {}",
                        args[i],
                        RuntimeType::available_runtimes().join(", ")
                    );
                    process::exit(1);
                }
            };
        } else if arg == "--help" || arg == "-h" {
            print_usage(&args[0]);
            return;
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
            process::exit(1);
        }
    };

    let player = match fs::read_to_string(&player_path) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("ERROR: Failed to read player file: {}", e);
            process::exit(1);
        }
    };

    if requests_args.is_empty() {
        eprintln!("ERROR: At least one request is required");
        print_usage(&args[0]);
        process::exit(1);
    }

    match run(player, runtime_type, requests_args) {
        Ok(output) => match serde_json::to_string(&output) {
            Ok(json) => {
                println!("{}", json);
            }
            Err(e) => {
                eprintln!("ERROR: Failed to serialize output: {}", e);
            }
        },
        Err(e) => {
            eprintln!("ERROR: {}", e);
        }
    }
}
