use std::env;
use std::fs;
use std::process;

use ejs::{RuntimeType, run};

#[cfg(feature = "snmalloc")]
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
    if let Err(e) = run_main() {
        eprintln!("ERROR: {}", e);
        process::exit(1);
    }
}

fn run_main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        print_usage(&args[0]);
        process::exit(1);
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
            }
            "--help" | "-h" => {
                print_usage(&args[0]);
                return Ok(());
            }
            _ if player_path.is_none() => player_path = Some(arg.clone()),
            _ => requests_args.push(arg.clone()),
        }
        i += 1;
    }

    let player_path = player_path.ok_or("Missing player file argument")?;
    let player = fs::read_to_string(&player_path)?;

    if requests_args.is_empty() {
        return Err("At least one request is required".into());
    }

    let output = run(player, runtime_type, requests_args)?;
    let json = serde_json::to_string(&output)?;
    println!("{}", json);

    Ok(())
}
